use ibuilder::*;

#[derive(IBuilder)]
struct NonDefault {
    field: String,
}

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(hidden)]
    field: NonDefault,
}

fn main() {
    Foo::builder();
}
