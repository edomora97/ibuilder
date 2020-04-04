use ibuilder::nodes::Node;
use ibuilder::*;

#[derive(ibuilder)]
pub struct Foo {
    bar: i32,
}

fn main() {
    let mut builder: Builder<Foo> = Foo::builder();
    let _: Options = builder.get_options();
    let _: Result<Option<Foo>, ChooseError> = builder.choose(Input::text("foo"));
    let _: Result<Option<Foo>, ChooseError> = builder.choose(Input::choice("foo"));
    let _: Result<Foo, FinalizeError> = builder.finalize();
    let _: bool = builder.is_done();
    let _: Node = builder.to_node();
}
