use ibuilder::*;

#[derive(IBuilder)]
pub struct Foo {
    bar: Bar,
}

#[derive(IBuilder)]
pub struct Bar {
    baz: Baz,
}

#[derive(IBuilder)]
pub struct Baz {
    bim: Bim,
}

#[derive(IBuilder)]
pub struct Bim {
    val: String,
}

fn main() {
    Foo::builder();
}
