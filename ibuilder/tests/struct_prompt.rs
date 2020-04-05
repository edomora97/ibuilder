#![allow(dead_code)]

use ibuilder::*;

#[derive(IBuilder)]
struct DefaultPrompt {
    field: i32,
}

#[derive(IBuilder)]
#[ibuilder(prompt = "lol or lel?")]
struct UnnamedStruct(String);

#[derive(IBuilder)]
#[ibuilder(prompt = "lol or lel?")]
struct CustomPrompt {
    #[ibuilder(prompt = "plain field prompt")]
    field: i32,
    #[ibuilder(prompt = "override enum")]
    var: Enum,
    #[ibuilder(prompt = "override struct")]
    nest: Nested,
}

#[derive(IBuilder)]
#[ibuilder(prompt = "base prompt")]
struct Nested {
    field: i32,
}

#[derive(IBuilder)]
#[ibuilder(prompt = "base prompt")]
enum Enum {
    Var,
}

#[test]
fn default_prompt() {
    let mut builder = DefaultPrompt::builder();

    let options = builder.get_options();
    assert!(!options.query.is_empty());

    builder.choose(Input::choice("field")).unwrap();
    let options = builder.get_options();
    assert!(!options.query.is_empty());
}

#[test]
fn unnamed_prompt() {
    let builder = UnnamedStruct::builder();

    let options = builder.get_options();
    assert_eq!(options.query, "lol or lel?");
}

#[test]
fn custom_prompt() {
    let mut builder = CustomPrompt::builder();

    let options = builder.get_options();
    assert_eq!(options.query, "lol or lel?");

    builder.choose(Input::choice("field")).unwrap();
    let options = builder.get_options();
    assert_eq!(options.query, "plain field prompt");

    builder.choose(Input::choice(BACK_ID)).unwrap();
    builder.choose(Input::choice("var")).unwrap();
    let options = builder.get_options();
    assert_eq!(options.query, "override enum");

    builder.choose(Input::choice(BACK_ID)).unwrap();
    builder.choose(Input::choice("nest")).unwrap();
    let options = builder.get_options();
    assert_eq!(options.query, "override struct");
}
