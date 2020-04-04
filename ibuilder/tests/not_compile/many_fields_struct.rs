use ibuilder::*;

#[derive(ibuilder)]
struct Foo(i64, String);

fn main() {
    Foo::builder();
}
