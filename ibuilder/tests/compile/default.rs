use ibuilder::*;

#[derive(IBuilder)]
pub struct Foo {
    #[ibuilder(default = 42)]
    bar: u8,
}

fn main() {
    let _builder = Foo::builder();
}
