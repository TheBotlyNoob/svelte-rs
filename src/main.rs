static SVELTE: &[u8] = include_bytes!("../test/index.svelte");

fn main() {
    svelte_rs::parse::parse(&mut &*SVELTE);
}
