use ibuilder::*;
use rand::prelude::*;

#[derive(Debug, IBuilder)]
struct Base {
    integer: i32,
    #[ibuilder(default = 42)]
    defaulted: i32,
    inner: Inner,
    #[ibuilder(rename = "enum")]
    en: Enum,
}

#[derive(Debug, IBuilder)]
#[ibuilder(rename = "inner inner inner")]
struct Inner {
    string: Option<String>,
    #[ibuilder(default = "lol")]
    defaulted: String,
}

#[derive(Debug, IBuilder)]
#[ibuilder(prompt = "WHAAT??!")]
enum Enum {
    #[ibuilder(rename = "hello")]
    Var1,
    Var2 {
        #[ibuilder(hidden, default = "nope")]
        field: String,
        #[ibuilder(rename = "baz")]
        field2: Inner,
    },
    Var3(Inner),
    #[ibuilder(rename = "man! this field is strange!")]
    Var4(Vec<Vec<Option<Box<Vec<Box<Box<Base>>>>>>>),
}

#[test]
fn random() {
    let mut builder = Base::builder();
    const N_ITER: usize = 10_000;
    let mut rng = rand::thread_rng();

    for _ in 0..N_ITER {
        let options = builder.get_options();
        if options.text_input && rand::random() {
            let input = rand::random::<i32>().to_string();
            builder.choose(Input::text(&input)).unwrap_or_else(|e| {
                panic!(
                    "Failed to choose text '{}': {}\nBuilder: {:#?}",
                    input, e, builder
                )
            });
        } else {
            if rand::random() {
                let _res = builder.choose(Input::choice("totally not a valid choice"));
            // TODO: this actually triggers a bug. Need to update the signature of `apply`
            // if res.as_ref().unwrap_err() != &ChooseError::UnexpectedChoice {
            //     panic!(
            //         "Expecting ChooseError::UnexpectedChoice, but got: {:?}",
            //         res
            //     );
            // }
            } else {
                let choice = options.choices.choose(&mut rng).expect("Empty choices");
                builder
                    .choose(Input::choice(&choice.choice_id))
                    .unwrap_or_else(|e| {
                        panic!(
                            "Failed to choose option {:?}: {}\nBuilder: {:#?}",
                            choice, e, builder
                        )
                    });
            }
        }
    }
}
