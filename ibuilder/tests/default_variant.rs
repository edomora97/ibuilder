use ibuilder::*;

#[derive(IBuilder)]
pub enum NotDefaulted {
    Var1,
    Var2,
}

#[derive(IBuilder, Eq, PartialEq, Debug)]
pub enum Defaulted {
    #[ibuilder(default)]
    Var1,
    Var2,
}

#[test]
fn test_no_default() {
    let builder = NotDefaulted::builder();
    assert!(!builder.is_done());
}

#[test]
fn test_default() {
    let builder = Defaulted::builder();
    assert!(builder.is_done());
    let res = builder.finalize().unwrap();
    assert_eq!(res, Defaulted::Var1);
}
