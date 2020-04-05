#![allow(dead_code)]

use ibuilder::nodes::{Field, FieldKind, Node};
use ibuilder::*;

#[derive(IBuilder)]
#[ibuilder(rename = "Renamed struct")]
struct Struct {
    #[ibuilder(rename = "renamed field")]
    field: i32,
    #[ibuilder(rename = "enum")]
    enm: Enum,
}

#[derive(IBuilder)]
enum Enum {
    #[ibuilder(rename = "renamed variant")]
    Var1,
    #[ibuilder(rename = "renamed variant 2")]
    Var2 {
        #[ibuilder(rename = "renamed inner field")]
        field: i32,
    },
    #[ibuilder(rename = "renamed variant 3")]
    Var3(i32),
}

#[test]
fn test_struct() {
    let builder = Struct::builder();

    let options = builder.get_options();
    let choices: Vec<_> = options.choices.iter().map(|c| c.text.as_str()).collect();
    assert!(choices.contains(&"Edit renamed field"));
    assert!(choices.contains(&"Edit enum"));

    let nodes = builder.to_node();
    match nodes {
        Node::Leaf(_) => panic!("Expecting a composite"),
        Node::Composite(name, fields) => {
            assert_eq!(name, "Renamed struct");
            match &fields[0] {
                FieldKind::Named(name, _) => {
                    assert_eq!(name, "renamed field");
                }
                FieldKind::Unnamed(_) => panic!("Expecting a named field"),
            }
            match &fields[1] {
                FieldKind::Named(name, _) => {
                    assert_eq!(name, "enum");
                }
                FieldKind::Unnamed(_) => panic!("Expecting a named field"),
            }
        }
    }
}

#[test]
fn test_enum_options() {
    let builder = Enum::builder();
    let options = builder.get_options();
    let choices: Vec<_> = options.choices.iter().map(|c| c.text.as_str()).collect();
    assert!(choices.contains(&"renamed variant"));
    assert!(choices.contains(&"renamed variant 2"));
    assert!(choices.contains(&"renamed variant 3"));
}

#[test]
fn test_enum_empty() {
    let mut builder = Enum::builder();

    builder.choose(Input::choice("Var1")).unwrap();

    let nodes = builder.to_node();
    match nodes {
        Node::Leaf(field) => match field {
            Field::String(name) => assert_eq!(name, "renamed variant"),
            Field::Missing => panic!("Expecting a string"),
        },
        Node::Composite(_, _) => panic!("Expecting a leaf"),
    }
}

#[test]
fn test_enum_named() {
    let mut builder = Enum::builder();

    builder.choose(Input::choice("Var2")).unwrap();

    let options = builder.get_options();
    let choices: Vec<_> = options.choices.iter().map(|c| c.text.as_str()).collect();
    assert!(choices.contains(&"Edit renamed inner field"));

    let nodes = builder.to_node();
    match nodes {
        Node::Leaf(_) => panic!("Expecting a composite"),
        Node::Composite(name, fields) => {
            assert_eq!(name, "renamed variant 2");
            match &fields[0] {
                FieldKind::Named(name, _) => {
                    assert_eq!(name, "renamed inner field");
                }
                FieldKind::Unnamed(_) => panic!("Expecting a named"),
            }
        }
    }
}

#[test]
fn test_enum_unnamed() {
    let mut builder = Enum::builder();

    builder.choose(Input::choice("Var3")).unwrap();

    let nodes = builder.to_node();
    match nodes {
        Node::Leaf(_) => panic!("expecting a composite"),
        Node::Composite(name, _) => {
            assert_eq!(name, "renamed variant 3");
        }
    }
}
