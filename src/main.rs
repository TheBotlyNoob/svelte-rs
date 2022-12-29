use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use svelte_rs::parse::{Rule, Svelte};

fn main() {
    let pairs = match Svelte::parse(Rule::Program, "<a><a>") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
    };
    println!("{pairs:#?}");
}
