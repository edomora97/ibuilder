use ibuilder::ibuilder;

#[derive(ibuilder)]
enum Foo {
    Var,
}

fn main() {
    Foo::builder();
}
