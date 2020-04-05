use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(wibble_monster)]
    field: i64,
}

#[derive(IBuilder)]
#[ibuilder(wibble_monster)]
struct Bar {
    field: i64,
}

#[derive(IBuilder)]
#[ibuilder(wibble_monster)]
enum Baz {
    Var,
}

#[derive(IBuilder)]
enum Bim {
    #[ibuilder(wibble_monster)]
    Var,
}

fn main() {}
