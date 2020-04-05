#![allow(dead_code)]

#[derive(ibuilder::ibuilder)]
enum DefaultPrompt {
    Var,
}

#[derive(ibuilder::ibuilder)]
#[ibuilder(prompt = "lol or lel?")]
enum CustomPrompt {
    Var1,
    #[ibuilder(prompt = "override unnamed")]
    Var2(Nested),
    #[ibuilder(prompt = "custom prompt")]
    Var3 {
        #[ibuilder(prompt = "override field")]
        field: Nested,
    },
}

#[derive(ibuilder::ibuilder)]
#[ibuilder(prompt = "base prompt")]
struct Nested {
    field: i32,
}

use ibuilder::{Buildable, Input, BACK_ID};

#[test]
fn default_prompt() {
    let mut builder = DefaultPrompt::builder();

    let options = builder.get_options();
    assert!(!options.query.is_empty());

    builder.choose(Input::choice("Var")).unwrap();
    let options = builder.get_options();
    assert!(!options.query.is_empty());
}

#[test]
fn custom_prompt() {
    let mut builder = CustomPrompt::builder();

    let options = builder.get_options();
    assert_eq!(options.query, "lol or lel?");

    builder.choose(Input::choice("Var2")).unwrap();
    let options = builder.get_options();
    assert_eq!(options.query, "override unnamed");

    builder.choose(Input::choice(BACK_ID)).unwrap();
    builder.choose(Input::choice("Var3")).unwrap();
    let options = builder.get_options();
    assert_eq!(options.query, "custom prompt");

    builder.choose(Input::choice("field")).unwrap();
    let options = builder.get_options();
    assert_eq!(options.query, "override field");
}
