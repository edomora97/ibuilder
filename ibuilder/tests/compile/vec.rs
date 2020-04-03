use ibuilder_derive::ibuilder;

#[derive(ibuilder)]
pub struct Foo {
    bar: Vec<i32>,
    baz: Vec<Bim>,
}

#[derive(Debug, ibuilder)]
pub struct Bim {
    lol: i32,
}

fn main() {
    let _builder = Foo::builder();
}
