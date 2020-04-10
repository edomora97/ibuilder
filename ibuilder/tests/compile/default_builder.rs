use ibuilder::*;

#[derive(IBuilder)]
pub struct Foo {
    bar: u8,
}

fn main() {
    Builder::<Foo>::default();
}
