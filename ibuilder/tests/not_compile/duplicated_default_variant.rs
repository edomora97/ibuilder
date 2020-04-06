use ibuilder::*;

#[derive(IBuilder)]
pub enum Enum {
    #[ibuilder(default)]
    Var1,
    #[ibuilder(default)]
    Var2,
    Var3,
}

fn main() {}
