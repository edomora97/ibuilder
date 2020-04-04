use ibuilder::*;

#[derive(Debug, ibuilder, Eq, PartialEq)]
struct Base {
    integer: i32,
    #[ibuilder(default = 42)]
    defaulted: i32,
    inner: Inner,
}

#[derive(Debug, ibuilder, Eq, PartialEq)]
struct Inner {
    string: String,
    #[ibuilder(default = "lol")]
    defaulted: String,
}

#[test]
fn test_interaction() {
    let mut builder: Builder<Base> = Base::builder();
    assert!(!builder.is_done());
    assert!(builder.finalize().is_err());

    let options = builder.get_options();
    assert!(!options.text_input);
    assert!(has_choice("integer", &options));
    assert!(has_choice("defaulted", &options));
    assert!(has_choice("inner", &options));
    assert!(!has_choice(BACK_ID, &options));
    assert!(!has_choice(FINALIZE_ID, &options));

    let res = builder.choose(Input::Choice("nope".into()));
    assert_eq!(res, Err(ChooseError::UnexpectedChoice));
    let res = builder.choose(Input::Choice(BACK_ID.into()));
    assert_eq!(res, Err(ChooseError::UnexpectedChoice));
    let res = builder.choose(Input::Choice("integer".into()));
    assert_eq!(res, Ok(None));

    let options = builder.get_options();
    assert!(options.text_input);
    assert!(has_choice(BACK_ID, &options));

    let res = builder.choose(Input::Text("nope".into()));
    if let Err(ChooseError::InvalidText { .. }) = res {
    } else {
        panic!("expecting ChooseError::InvalidText");
    }
    let res = builder.choose(Input::Text("123".into()));
    assert_eq!(res, Ok(None));

    let options = builder.get_options();
    assert!(!options.text_input);
    assert!(has_choice("integer", &options));
    assert!(has_choice("defaulted", &options));
    assert!(has_choice("inner", &options));
    assert!(!has_choice(BACK_ID, &options));
    assert!(!has_choice(FINALIZE_ID, &options));

    let res = builder.choose(Input::Choice("inner".into()));
    assert_eq!(res, Ok(None));

    let options = builder.get_options();
    assert!(!options.text_input);
    assert!(has_choice("string", &options));
    assert!(has_choice("defaulted", &options));
    assert!(has_choice(BACK_ID, &options));
    assert!(!has_choice(FINALIZE_ID, &options));

    let res = builder.choose(Input::Choice("string".into()));
    assert_eq!(res, Ok(None));

    let options = builder.get_options();
    assert!(options.text_input);
    assert!(has_choice(BACK_ID, &options));
    assert!(!has_choice(FINALIZE_ID, &options));

    let res = builder.choose(Input::Text("lallabalalla".into()));
    assert_eq!(res, Ok(None));

    let options = builder.get_options();
    assert!(!options.text_input);
    assert!(has_choice("string", &options));
    assert!(has_choice("defaulted", &options));
    assert!(has_choice(BACK_ID, &options));
    assert!(!has_choice(FINALIZE_ID, &options));

    let res = builder.choose(Input::Choice(BACK_ID.into()));
    assert_eq!(res, Ok(None));

    let options = builder.get_options();
    assert!(!options.text_input);
    assert!(has_choice("integer", &options));
    assert!(has_choice("defaulted", &options));
    assert!(has_choice("inner", &options));
    assert!(!has_choice(BACK_ID, &options));
    assert!(has_choice(FINALIZE_ID, &options));

    let res = builder.choose(Input::Choice(FINALIZE_ID.into()));
    assert_eq!(
        res,
        Ok(Some(Base {
            integer: 123,
            defaulted: 42,
            inner: Inner {
                string: "lallabalalla".to_string(),
                defaulted: "lol".to_string()
            }
        }))
    );
}

fn has_choice<S: AsRef<str>>(id: S, options: &Options) -> bool {
    for opt in &options.choices {
        if opt.choice_id == id.as_ref() {
            return true;
        }
    }
    return false;
}
