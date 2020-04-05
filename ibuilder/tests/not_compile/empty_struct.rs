use ibuilder::*;

#[derive(Debug, IBuilder)]
struct Empty1;

#[derive(Debug, IBuilder)]
struct Empty2();

#[derive(Debug, IBuilder)]
struct Empty3 {}

fn main() {
    Empty1::builder();
    Empty2::builder();
    Empty3::builder();
}
