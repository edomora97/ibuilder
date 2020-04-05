use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(default = "not an integer")]
    field: i64,
}

fn main() {
    Foo::builder();
}
