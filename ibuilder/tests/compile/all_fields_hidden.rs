use ibuilder::IBuilder;

#[derive(IBuilder)]
struct Struct {
    #[ibuilder(hidden, default = 42)]
    field: i32,
}

fn main() {}
