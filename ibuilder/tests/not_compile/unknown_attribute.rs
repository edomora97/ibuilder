#[derive(ibuilder::ibuilder)]
struct Foo {
    #[ibuilder(wibble_monster = 42)]
    field: i64,
}

#[derive(ibuilder::ibuilder)]
#[ibuilder(wibble_monster = 42)]
struct Bar {
    field: i64,
}

#[derive(ibuilder::ibuilder)]
#[ibuilder(wibble_monster = 42)]
enum Baz {
    Var,
}

#[derive(ibuilder::ibuilder)]
enum Bim {
    #[ibuilder(wibble_monster = 42)]
    Var,
}

fn main() {}
