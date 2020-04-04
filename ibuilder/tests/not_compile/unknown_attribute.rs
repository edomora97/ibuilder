use ibuilder::*;

#[derive(ibuilder)]
struct Foo {
    #[ibuilder(wibble_monster = 42)]
    field: i64,
}

fn main() {
    Foo::builder();
}
