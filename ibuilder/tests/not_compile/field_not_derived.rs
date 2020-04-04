use ibuilder::*;

#[derive(ibuilder)]
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
