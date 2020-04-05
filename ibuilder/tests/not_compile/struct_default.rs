use ibuilder::*;

#[derive(IBuilder)]
struct Foo {
    #[ibuilder(default = 42)]
    field: Bar,
}

#[derive(IBuilder)]
struct Bar {
    #[ibuilder(default = 42)]
    field: i32,
}

fn main() {
    Foo::builder();
}
