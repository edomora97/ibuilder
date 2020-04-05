use ibuilder::*;

#[derive(IBuilder)]
pub struct Foo {
    bar: Vec<i32>,
    baz: Vec<Bim>,
}

#[derive(Debug, IBuilder)]
pub struct Bim {
    lol: i32,
}

fn main() {
    let _builder = Foo::builder();
}
