use ibuilder::ibuilder;

#[derive(ibuilder)]
struct Foo {
    #[ibuilder(default = 42, default = 42)]
    field: i64,
}

fn main() {
    Foo::builder();
}
