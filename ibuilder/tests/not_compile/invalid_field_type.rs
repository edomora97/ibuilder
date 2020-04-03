use ibuilder::ibuilder;

#[derive(ibuilder)]
struct Foo {
    field: i64,
    ups: &'static str,
}

fn main() {
    Foo::builder();
}
