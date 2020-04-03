//! Test inter-mod builders.

#![allow(dead_code)]

mod mod1 {
    use ibuilder_derive::ibuilder;

    #[derive(ibuilder)]
    pub struct Foo {
        bar: super::mod2::Bar,
    }
}

mod mod2 {
    use ibuilder_derive::ibuilder;

    #[derive(ibuilder)]
    pub struct Bar {
        field: i64,
    }
}

#[test]
fn test_mods() {
    let _builder = mod1::Foo::builder();
}
