use ibuilder::*;

#[derive(Debug, IBuilder)]
struct Test(i64);

#[test]
fn test() {
    Test::builder();
}
