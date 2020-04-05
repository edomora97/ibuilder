use ibuilder::*;

#[derive(IBuilder)]
pub struct Foo {
    bar: i32,
}

fn main() {
    let _builder = Foo::builder();
}
