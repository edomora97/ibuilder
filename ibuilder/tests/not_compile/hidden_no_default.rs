use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(hidden)]
    field: i64,
}

fn main() {
    Foo::builder();
}
