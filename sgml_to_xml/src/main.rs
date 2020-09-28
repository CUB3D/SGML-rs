use crate::dtd::read_dtd;
use crate::element::{parse_content_model, ContentModelTokenValue};
use std::fs::File;
use std::io::Read;

fn main() {
    let mut f = File::open("../sgml/dtd/html.dtd").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let x = sgml::read_dtd(&s);
    println!("{:?}", x);
    let (i, e) = x.unwrap();

    // find the root, will be the only element which is not a child of another
    println!("Root = {:#?}", e.get_roots());
}
