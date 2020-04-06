use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(default = -123)]
    field: u64,
}

fn main() {
    Foo::builder();
}
