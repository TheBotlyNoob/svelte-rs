use anyhow::Result;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use swc::{
    config::{Config, JscConfig, Options},
    Compiler,
};
use swc_bundler::{Bundle, Bundler, Hook, ModuleRecord};
use swc_common::{FileName, FilePathMapping, Globals, SourceMap, Span, GLOBALS};
use swc_ecma_ast::KeyValueProp;
use swc_ecma_codegen::{
    text_writer::{omit_trailing_semi, JsWriter, WriteJs},
    Emitter,
};
use swc_ecma_loader::{resolvers::node::NodeModulesResolver, TargetEnv};
use swc_ecma_parser::{Syntax, TsConfig};
use swc_node_bundler::loaders::swc::SwcLoader;

const MINIFY: bool = !cfg!(debug_assertions);

fn _main() -> Result<()> {
    let options = Options {
        config: Config {
            #[cfg(not(debug_assertions))]
            minify: MINIFY.into(), // TODO(@TheBotlyNoob): turn this into a CLI flag
            jsc: JscConfig {
                syntax: Some(Syntax::Typescript(TsConfig {
                    tsx: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let c = Arc::new(Compiler::new(Arc::new(SourceMap::new(
        FilePathMapping::empty(),
    ))));

    let loader = SwcLoader::new(c.clone(), options);

    let globals = Globals::new();
    let mut bundler = Bundler::new(
        &globals,
        c.cm.clone(),
        loader,
        NodeModulesResolver::new(TargetEnv::default(), Default::default(), true),
        swc_bundler::Config {
            require: false,
            disable_inliner: false,
            disable_hygiene: MINIFY,
            disable_fixer: MINIFY,
            disable_dce: false,
            ..Default::default()
        },
        Box::new(Noop),
    );

    let mut entries = HashMap::new();

    entries.insert(
        String::from("main"),
        FileName::Real(PathBuf::from("test/index.svelte")),
    );

    let out = bundler.bundle(entries)?;

    print_bundles(c.cm.clone(), out, MINIFY);

    Ok(())
}

fn print_bundles(cm: Arc<SourceMap>, modules: Vec<Bundle>, minify: bool) {
    for bundled in modules {
        let code = {
            let mut buf = vec![];

            {
                let wr = JsWriter::new(cm.clone(), "\n", &mut buf, None);
                let mut emitter = Emitter {
                    cfg: swc_ecma_codegen::Config {
                        minify,
                        ..Default::default()
                    },
                    cm: cm.clone(),
                    comments: None,
                    wr: if minify {
                        Box::new(omit_trailing_semi(wr)) as Box<dyn WriteJs>
                    } else {
                        Box::new(wr) as Box<dyn WriteJs>
                    },
                };

                emitter.emit_module(&bundled.module).unwrap();
            }

            String::from_utf8_lossy(&buf).to_string()
        };

        println!("Created output.js ({}kb)", code.len() / 1024);
        std::fs::write("output.js", &code).unwrap();
    }
}

fn main() -> Result<()> {
    GLOBALS.set(&Globals::new(), _main)
}

struct Noop;
impl Hook for Noop {
    fn get_import_meta_props(
        &self,
        _span: Span,
        _module_record: &ModuleRecord,
    ) -> Result<Vec<KeyValueProp>, anyhow::Error> {
        todo!()
    }
}
