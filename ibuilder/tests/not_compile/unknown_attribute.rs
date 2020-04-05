use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(wibble_monster = 42)]
    field: i64,
}

#[derive(IBuilder)]
#[ibuilder(wibble_monster = 42)]
struct Bar {
    field: i64,
}

#[derive(IBuilder)]
#[ibuilder(wibble_monster = 42)]
enum Baz {
    Var,
}

#[derive(IBuilder)]
enum Bim {
    #[ibuilder(wibble_monster = 42)]
    Var,
}

fn main() {}
