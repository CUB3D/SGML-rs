#[macro_use]
extern crate dtd_gen;
use dtd_gen::dtd;

dtd!();

fn main() {
    let html = LINK { children: vec![] };
    println!("{:?}", html);
}
