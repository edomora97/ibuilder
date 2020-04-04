use ibuilder::*;

#[derive(ibuilder)]
struct Foo {
    #[ibuilder(wibble(monster))]
    field: i64,
}

fn main() {
    Foo::builder();
}
