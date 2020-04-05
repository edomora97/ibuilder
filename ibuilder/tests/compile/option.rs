use ibuilder::*;

#[derive(IBuilder)]
pub struct Foo {
    bar: Option<i32>,
    baz: Option<Bim>,
}

#[derive(Debug, IBuilder)]
pub struct Bim {
    lol: i32,
}

fn main() {
    let _builder = Foo::builder();
}
