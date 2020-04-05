#[derive(ibuilder::ibuilder)]
struct Foo {
    #[ibuilder(wibble(monster))]
    field: i64,
}

#[derive(ibuilder::ibuilder)]
#[ibuilder(wibble(monster))]
struct Bar {
    field: i64,
}

#[derive(ibuilder::ibuilder)]
#[ibuilder(wibble(monster))]
enum Baz {
    Var,
}

#[derive(ibuilder::ibuilder)]
enum Bim {
    #[ibuilder(wibble(monster))]
    Var,
}

fn main() {}
