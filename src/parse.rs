use html5ever::{namespace_url, ns, parse_fragment, tendril::TendrilSink, Attribute, QualName};
use itertools::Itertools;
use markup5ever::local_name;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    io::Read,
};

#[derive(Clone, Debug)]
struct Parser {
    in_script: bool,
    in_style: bool,
    script: String,
    style: String,
    create_fn: String,
    mount_fn: String,
    internal_imports: HashSet<String>,
    elem_vars: HashMap<String, /* how many times the var occurs */ u32>,
    depth: u32,
}

impl Parser {
    pub fn new(doc: &Handle) -> Self {
        let mut this = Self {
            in_script: false,
            in_style: false,
            script: String::new(),
            style: String::new(),
            create_fn: String::new(),
            mount_fn: String::new(),
            internal_imports: HashSet::new(),
            elem_vars: HashMap::new(),
            depth: 0,
        };
        this.add_imports(vec!["SvelteComponent", "init", "safe_not_equal"]);
        this.walk(&doc);
        this
    }
    fn walk(&mut self, node: &Handle) {
        let Self {
            in_script,
            in_style,
            ..
        } = *self;

        match &node.data {
            NodeData::Text { contents } => {
                let contents = contents.borrow();
                let t = contents.trim();
                if self.in_script {
                    self.script.push_str(t);
                } else if self.in_style {
                    self.style.push_str(t)
                } else {
                }
            }

            NodeData::Element { name, attrs, .. } => {
                match name.local {
                    local_name!("script") => self.in_script = true,
                    local_name!("style") => self.in_style = true,
                    _ => self.create_elem(&*name.local, &attrs.borrow()),
                };
            }

            NodeData::ProcessingInstruction { .. } => unreachable!(),

            _ => {}
        }

        self.depth += 1;

        for e in &*node.children.borrow() {
            self.walk(e);
        }

        // We don't want parallel children to be counted as a script/style
        self.in_script = in_script;
        self.in_style = in_style;
    }

    fn create_elem(&mut self, name: &str, attrs: &Vec<Attribute>) {
        self.add_imports(vec!["attr", "detach", "element", "init", "insert"]);

        let var = self.create_elem_var(name);

        let set_attrs = attrs.iter().format_with(";", |a, f| {
            f(&format_args!(
                "
                attr(
                    {var}, 
                    \"{}\", 
                    \"{}\"
                )
                ",
                a.name.local, a.value
            ))
        });
        self.create_fn = format!(
            "
            {}

            {var} = element(\"{name}\");
            {set_attrs}
            ",
            self.create_fn
        );
        self.mount_fn = format!(
            "
                insert(target, {var}, anchor);
            "
        );
    }

    fn create_elem_var(&mut self, var: impl Into<String>) -> String {
        let var = var.into();
        let idx = self.elem_vars.entry(var.clone()).or_default();
        *idx += 1;
        format!("{var}{idx}")
    }

    fn add_imports(&mut self, imports: Vec<impl Into<String>>) {
        for i in imports {
            self.internal_imports.insert(i.into());
        }
    }
}

impl Display for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            create_fn,
            mount_fn,
            elem_vars,
            internal_imports,
            script,
            style,
            ..
        } = self;

        let internal_imports = internal_imports.iter().format(",");
        let detach = elem_vars
            .iter()
            .format_with(";", |(v, i), f| f(&format_args!("detach({v}{i})")));
        let elem_vars = elem_vars
            .iter()
            .format_with(";", |(v, i), f| f(&format_args!("let {v}{i}")));

        write!(
            f,
            r#"
import {{
    {internal_imports}
}} from "svelte/internal";

function create_fragment(ctx) {{
    {elem_vars};

    return {{
        c() {{
            {create_fn};
        }},
        m(target, anchor) {{
            {mount_fn};
        }},
        p: noop,
        i: noop,
        o: noop,
        d(detaching) {{
            if (detaching) {{
                {detach};
            }}
        }}
    }};
}}

class App extends SvelteComponent {{
    constructor(options) {{
        super();
        init(this, options, null, create_fragment, safe_not_equal, {{   }});
    }}
}}

export default App;
        "#
        )
    }
}

pub fn parse(f: &mut impl Read) {
    let dom = parse_fragment(
        RcDom::default(),
        Default::default(),
        QualName::new(None, ns!(html), local_name!("")),
        vec![],
    )
    .from_utf8()
    .read_from(f)
    .unwrap();
    let parser = Parser::new(&dom.document);

    println!("{parser}");
}
