use ibuilder::ibuilder;

#[derive(ibuilder)]
struct Foo(i64);

fn main() {
    Foo::builder();
}
