use ibuilder::*;

#[derive(ibuilder)]
struct Foo {
    #[ibuilder(default = 42)]
    field: Bar,
}

#[derive(ibuilder)]
struct Bar {
    #[ibuilder(default = 42)]
    field: i32,
}

fn main() {
    Foo::builder();
}
