#![crate_type = "proc-macro"]
extern crate proc_macro;
use proc_macro::TokenStream;
use sgml::dtd::{read_dtd, DocumentTypeDefinition, DocumentTypeDefinitionElement};
use sgml::element::{ContentModelTokenValue, Element};
use sgml::entity::Entity;
use std::fs::File;
use std::io::Read;

#[macro_use]
extern crate quote;

fn generate_element(
    r: &DocumentTypeDefinitionElement,
    entities: &[Entity],
    e: &DocumentTypeDefinition,
    mut already_generated: &mut Vec<String>,
) -> proc_macro2::TokenStream {
    let r_name = r.get_name();

    let rc = r.get_children();
    let r_children = rc
        .iter()
        .filter(|c| !already_generated.contains(&c.get_name()))
        .collect::<Vec<_>>();
    println!("children = {:?}", r_children);
    let children_name = format!("{}Children", r_name);

    let childre_props = r_children
        .iter()
        .map(|f| {
            let child_name = f.get_name();
            let child_name_ident = format_ident!("{}", child_name);
            quote! {
                #child_name_ident(#child_name_ident)
            }
        })
        .collect::<Vec<_>>();

    let r_name_ident = format_ident!("{}", r_name);
    let r_children_name_ident = format_ident!("{}Children", r_name);

    //Must add everything new to the already genned list before recursing
    for n in &r_children {
        already_generated.push(n.get_name());
    }

    // Recurse for children
    let childre_structs = r_children
        .iter()
        .map(|c| generate_element(c, entities, e, &mut already_generated));

    let x = quote! {
        #[derive(Debug, Clone)]
        pub struct #r_name_ident {
            pub children: Vec<#r_children_name_ident>
        }

        #[derive(Debug, Clone)]
        pub enum #r_children_name_ident {
            #(
            #childre_props,
            )*
        }

        #(#childre_structs)*
    };
    x
}

#[proc_macro]
pub fn dtd(_item: TokenStream) -> TokenStream {
    let mut f = File::open("/mnt/Media/software/sgml/dtd_gen_example/dtd/html.dtd").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let x = read_dtd(&s);
    let (i, e) = x.unwrap();

    let entities = &e.entities;
    let roots = e.get_roots();
    let root_names = roots.iter().map(|r| format_ident!("{}", r.get_name()));

    let mut already_generated = roots.iter().map(|r| r.get_name()).collect::<Vec<_>>();

    let roots = roots
        .iter()
        .map(|r| generate_element(r, &entities, &e, &mut already_generated))
        .collect::<Vec<_>>();

    let root_struct = quote! {
        #[derive(Debug, Clone)]
        pub enum Root {
            #(
            #root_names(#root_names),
            )*
        }
    };

    let a = quote! {
        #(#roots)*
        #root_struct
    };

    a.into()
}
