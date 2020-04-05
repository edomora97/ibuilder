use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    field: i64,
    ups: Bar,
}

struct Bar {
    baz: i32,
}

fn main() {
    Foo::builder();
}
