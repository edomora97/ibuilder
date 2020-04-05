#[derive(ibuilder::ibuilder)]
struct Foo {
    #[ibuilder(wibble_monster)]
    field: i64,
}

#[derive(ibuilder::ibuilder)]
#[ibuilder(wibble_monster)]
struct Bar {
    field: i64,
}

#[derive(ibuilder::ibuilder)]
#[ibuilder(wibble_monster)]
enum Baz {
    Var,
}

#[derive(ibuilder::ibuilder)]
enum Bim {
    #[ibuilder(wibble_monster)]
    Var,
}

fn main() {}
