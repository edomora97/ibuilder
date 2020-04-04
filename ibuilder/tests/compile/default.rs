use ibuilder::*;

#[derive(ibuilder)]
pub struct Foo {
    #[ibuilder(default = 42)]
    bar: i32,
}

fn main() {
    let _builder = Foo::builder();
}
