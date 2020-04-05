use ibuilder::*;

#[derive(IBuilder)]
struct Foo(i64);

fn main() {
    Foo::builder();
}
