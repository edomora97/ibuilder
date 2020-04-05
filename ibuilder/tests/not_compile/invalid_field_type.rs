use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    field: i64,
    ups: &'static str,
}

fn main() {
    Foo::builder();
}
