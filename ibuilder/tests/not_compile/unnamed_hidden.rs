use ibuilder::*;

#[derive(IBuilder)]
struct Foo(#[ibuilder(hidden)] i64);

fn main() {
    Foo::builder();
}
