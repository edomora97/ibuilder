use ibuilder::*;

#[derive(ibuilder)]
pub struct Foo {
    bar: Bar,
}

#[derive(ibuilder)]
pub struct Bar {
    baz: Baz,
}

#[derive(ibuilder)]
pub struct Baz {
    bim: Bim,
}

#[derive(ibuilder)]
pub struct Bim {
    val: String,
}

fn main() {
    let _builder = Foo::builder();
}
