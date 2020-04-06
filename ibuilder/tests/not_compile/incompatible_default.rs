use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(default = "not an integer")]
    field: i64,
}

#[derive(IBuilder)]
struct Bar {
    #[ibuilder(default = -123)]
    field: u64,
}

fn main() {
    Foo::builder();
}
