use html5ever::{namespace_url, ns, parse_fragment, tendril::TendrilSink, QualName};
use markup5ever::local_name;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::io::Read;

#[derive(Clone, Debug, Default)]
struct Parser {
    in_script: bool,
    in_style: bool,
    script: String,
    style: String,
    depth: u32,
}
impl Parser {
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
                }
            }

            NodeData::Element { name, attrs, .. } => {
                assert!(name.ns == ns!(html));
                match name.local {
                    local_name!("script") => self.in_script = true,
                    local_name!("style") => self.in_style = true,
                    _ => {}
                };
                print!("<{}", name.local);
                for attr in attrs.borrow().iter() {
                    assert!(attr.name.ns == ns!());
                    print!(" {}=\"{}\"", attr.name.local, attr.value);
                }
                println!(">");
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
    let nodes = dom.document.children.borrow();
    let html_elem = nodes.get(0).expect("HTML tag");
    let mut parser = Parser::default();
    for e in &*html_elem.children.borrow() {
        parser.walk(e);
    }

    println!();

    println!("SCRIPT:");
    println!("{}", parser.script);
    println!(":SCRIPT");
    println!("STYLE:");
    println!("{}", parser.style);
    println!(":STYLE");

    println!();

    println!("ERRORS:");

    let errs = dom.errors.into_iter().filter(|s| {
        ![
            // gets emitted unintentionally for components
            // e.g. <slot />, <Component />
            "Unacknowledged self-closing tag",
            // gets emitted because of the above error
            "Unexpected open tag at end of body",
        ]
        .contains(&&**s)
    });
    for err in errs {
        println!("\t{}", err);
    }

    println!(":ERRORS")
}
