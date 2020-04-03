use ibuilder_derive::ibuilder;

#[derive(ibuilder)]
pub struct Foo {
    bar: i32,
}

fn main() {
    let _builder = Foo::builder();
}
