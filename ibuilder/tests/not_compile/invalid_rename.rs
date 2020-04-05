use ibuilder::IBuilder;

#[derive(IBuilder)]
enum Enum {
    #[ibuilder(rename = 42)]
    Var1,
}

fn main() {}
