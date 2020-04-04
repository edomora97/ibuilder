use ibuilder::{nodes::Node, Builder, ChooseError, FinalizeError, Input, Options};
use ibuilder_derive::ibuilder;

#[derive(ibuilder)]
pub struct Foo {
    bar: i32,
}

fn main() {
    let mut builder: Builder<Foo> = Foo::builder();
    let _: Options = builder.get_options();
    let _: Result<Option<Foo>, ChooseError> = builder.choose(Input::Text("foo".to_string()));
    let _: Result<Option<Foo>, ChooseError> = builder.choose(Input::Choice("foo".to_string()));
    let _: Result<Foo, FinalizeError> = builder.finalize();
    let _: bool = builder.is_done();
    let _: Node = builder.to_node();
}
