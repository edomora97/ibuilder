use ibuilder::*;

#[derive(IBuilder)]
#[ibuilder(rename = "it doesn't make sense since this name is not shown")]
enum Baz {
    Var,
}

fn main() {}
