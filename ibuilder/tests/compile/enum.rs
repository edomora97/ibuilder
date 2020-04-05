use ibuilder::*;

#[derive(IBuilder)]
enum Foo {
    Var,
    WithFields {
        lol: i32,
        #[ibuilder(default = "ciao")]
        baz: String,
    },
}

fn main() {
    Foo::builder();
}
