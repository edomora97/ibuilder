use ibuilder::*;

#[derive(Debug, ibuilder)]
struct Empty1;

#[derive(Debug, ibuilder)]
struct Empty2();

#[derive(Debug, ibuilder)]
struct Empty3 {}

fn main() {
    Empty1::builder();
    Empty2::builder();
    Empty3::builder();
}
