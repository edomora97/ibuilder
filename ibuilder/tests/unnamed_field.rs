use ibuilder::*;

#[derive(Debug, ibuilder)]
struct Test(i64);

#[test]
fn test() {
    Test::builder();
}
