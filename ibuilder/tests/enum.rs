use ibuilder::*;

#[derive(Debug, IBuilder)]
enum Enum {
    Var1,
    Var2 {
        field: i32,
        #[ibuilder(default = "ciao")]
        lol: String,
    },
    Var3(i32),
}

#[test]
fn test() {
    let _ = Enum::builder();
}
