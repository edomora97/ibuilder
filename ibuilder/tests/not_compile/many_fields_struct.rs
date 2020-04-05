use ibuilder::*;

#[derive(IBuilder)]
struct Foo(i64, String);

fn main() {
    Foo::builder();
}
