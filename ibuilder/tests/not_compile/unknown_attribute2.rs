use ibuilder::*;

#[derive(ibuilder)]
struct Foo {
    #[ibuilder(wibble_monster)]
    field: i64,
}

fn main() {
    Foo::builder();
}
