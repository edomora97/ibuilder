use ibuilder::IBuilder;

#[derive(IBuilder)]
enum Enum {
    #[ibuilder(hidden)]
    Var1,
}

fn main() {}
