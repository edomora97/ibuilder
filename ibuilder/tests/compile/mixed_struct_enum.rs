use ibuilder::*;

#[derive(IBuilder)]
pub struct Foo {
    bar: Bar,
    bim: Box<Bim>,
}

#[derive(IBuilder)]
pub struct Bar {
    baz: Baz,
}

#[derive(IBuilder)]
pub struct Baz(Bim);

#[derive(IBuilder)]
pub struct Bim {
    val: Vec<Enum>,
}

#[derive(IBuilder)]
pub enum Enum {
    Var1,
    Var2(String),
    Var3(Bar),
    Var4 { foo: Foo, lol: i32 },
    Var5(Enumeration),
}

#[derive(IBuilder)]
pub enum Enumeration {
    Var1,
    Var2(Vec<Vec<Enum>>),
}

fn main() {
    Foo::builder();
}
