use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(wibble(monster))]
    field: i64,
}

#[derive(IBuilder)]
#[ibuilder(wibble(monster))]
struct Bar {
    field: i64,
}

#[derive(IBuilder)]
#[ibuilder(wibble(monster))]
enum Baz {
    Var,
}

#[derive(IBuilder)]
enum Bim {
    #[ibuilder(wibble(monster))]
    Var,
}

fn main() {}
