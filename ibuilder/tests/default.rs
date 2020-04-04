use ibuilder::*;

#[derive(ibuilder)]
pub struct NestedFoo {
    foo: Foo,
}

#[derive(ibuilder)]
pub struct Foo {
    #[ibuilder(default = 42)]
    bar: i32,
}

#[test]
fn test_default() {
    let builder = Foo::builder();
    assert!(builder.is_done());
    let fooo = builder.finalize().unwrap();
    assert_eq!(fooo.bar, 42);
}

#[test]
fn test_nested_default() {
    let builder = NestedFoo::builder();
    assert!(builder.is_done());
    let fooo = builder.finalize().unwrap();
    assert_eq!(fooo.foo.bar, 42);
}
