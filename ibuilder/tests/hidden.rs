#![allow(dead_code)]

use ibuilder::nodes::{FieldKind, Node};
use ibuilder::*;

#[derive(IBuilder, Eq, PartialEq, Debug)]
struct Struct {
    #[ibuilder(hidden, default = 42)]
    field: i32,
    enm: Enum,
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct Defaultable {
    field: String,
}

impl Default for Defaultable {
    fn default() -> Self {
        Defaultable {
            field: "success".into(),
        }
    }
}

#[derive(IBuilder, Eq, PartialEq, Debug)]
struct StructWithoutDefault {
    #[ibuilder(hidden)]
    field: Defaultable,
    field2: i32,
}

#[derive(IBuilder, Eq, PartialEq, Debug)]
enum Enum {
    #[ibuilder(hidden)]
    Var1,
    Var2 {
        field: i32,
    },
    Var3(i32),
}

#[test]
fn hidden_variant() {
    let mut builder = Enum::builder();

    let options = builder.get_options();
    let choices: Vec<_> = options.choices.iter().map(|c| c.text.as_str()).collect();
    assert!(!choices.contains(&"Var1"));

    assert_eq!(
        builder.choose(Input::choice("Var1")),
        Err(ChooseError::UnexpectedChoice)
    );
}

#[test]
fn hidden_field() {
    let mut builder = Struct::builder();

    let options = builder.get_options();
    let choices: Vec<_> = options.choices.iter().map(|c| c.text.as_str()).collect();
    assert!(!choices.contains(&"field"));

    assert_eq!(
        builder.choose(Input::choice("field")),
        Err(ChooseError::UnexpectedChoice)
    );

    let node = builder.to_node();
    match node {
        Node::Leaf(_) => panic!("expecting a composite"),
        Node::Composite(_, fields) => {
            assert_eq!(fields.len(), 1);
            match &fields[0] {
                FieldKind::Named(name, _) => {
                    assert_ne!(name, "field");
                }
                FieldKind::Unnamed(_) => panic!("expecting named"),
            }
        }
    }
}

#[test]
fn hidden_field_without_default() {
    let mut builder = StructWithoutDefault::builder();

    let options = builder.get_options();
    let choices: Vec<_> = options.choices.iter().map(|c| c.text.as_str()).collect();
    assert!(!choices.contains(&"field"));

    assert_eq!(
        builder.choose(Input::choice("field")),
        Err(ChooseError::UnexpectedChoice)
    );

    let node = builder.to_node();
    match node {
        Node::Leaf(_) => panic!("expecting a composite"),
        Node::Composite(_, fields) => {
            assert_eq!(fields.len(), 1);
            match &fields[0] {
                FieldKind::Named(name, _) => {
                    assert_ne!(name, "field");
                }
                FieldKind::Unnamed(_) => panic!("expecting named"),
            }
        }
    }

    builder.choose(Input::choice("field2")).unwrap();
    builder.choose(Input::text("42")).unwrap();

    let res = builder.finalize().unwrap();
    assert_eq!(res.field.field, "success");
}
