use ibuilder::*;

#[derive(ibuilder)]
pub struct Foo {
    bar: Bar,
    bim: Box<Bim>,
}

#[derive(ibuilder)]
pub struct Bar {
    baz: Baz,
}

#[derive(ibuilder)]
pub struct Baz(Bim);

#[derive(ibuilder)]
pub struct Bim {
    val: Vec<Enum>,
}

#[derive(ibuilder)]
pub enum Enum {
    Var1,
    Var2(String),
    Var3(Bar),
    Var4 { foo: Foo, lol: i32 },
    Var5(Enumeration),
}

#[derive(ibuilder)]
pub enum Enumeration {
    Var1,
    Var2(Vec<Vec<Enum>>),
}

fn main() {
    Foo::builder();
}
