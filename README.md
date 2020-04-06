# ibuilder

[![Rust](https://github.com/edomora97/ibuilder/workflows/Rust/badge.svg?branch=master)](https://github.com/edomora97/ibuilder/actions?query=workflow%3ARust)
[![crates.io](https://img.shields.io/crates/v/ibuilder.svg)](https://crates.io/crates/ibuilder)
[![Docs](https://docs.rs/ibuilder/badge.svg)](https://docs.rs/ibuilder)

Interactive builders for structs.

This crate provides a way to construct structs interactively, starting from an "empty" state
and filling the values of the fields of the struct prompting the user with multiple choices
and text inputs. After each choice the internal state of the builder changes.

The builder provides the user with interactive menu-like interfaces, keeping the UI abstract
and rust type-safeness.

The API of this crate is very simple:
- Derive a struct (or an enum) from `IBuilder`, including all the structs/enums that it depends
  upon;
- Call the `builder()` method (from the `Buildable` trait) to get an instance of `Builder<T>`;
- By calling `get_options()` on the builder you'll get an object that contains a message to show
  the user, a list of possible _choices_ (i.e. buttons to press) and eventually the possibility
  to enter some text (i.e. a text box);
- By calling `to_node()` on the builder you'll get a tree-like structure with the state of the
  builder, highlighting the fields that still need actions;
- You choose how to show to the user the options and when the user made the decision you call
  `choose(input)` on the builder. This will apply the choice to the state of the structure if
  it's valid, or return an error;
- When the state is complete (all the required fields are present) a new option is present in
  the list: _Done_. If the user selects it `choose` will return an instance of `T`.

### Rationale
When building an interactive application (e.g. a Telegram bot or a console application) which
needs many configurations it can be pretty cumbersome to come out with a decent interface
without spending loads of time writing the logic for handling the parsing and the validation of
the input.

This crates provides a useful abstraction that allows an easy connection between the data and
the user interface. Just by deriving the struct (or enum) that defines your data you can get a
safe interface over a possible UI.

The derive API is inspired by the great [`structopt`](https://docs.rs/structopt) crate.

### Supported features
- Deriving any struct with named fields (or with one unnamed field like `struct Foo(i64)`)
- Enums (also with variants with field, but only one if unnamed)
- Default values for the fields and default variant for enums
- Custom message prompt for fields, structs, enums and variants
- Renaming fields, structs and variants for better looking options
- Hidden fields (that takes the value only from the default)
- Nested structures (i.e. custom types)
- Supported field types: all numeric types from rust, `bool`, `String`, `char`, `Box<T>`,
  `Vec<T>` and `Option<T>`
- Any field type that implementes the `NewBuildableValue` trait

### Example of usage

In this example the data is stored inside a struct named `Person` which has 3 fields, one of
which has a default value. Deriving from `IBuilder` gives access to the `builder()` method that
returns a `Builder<Person>`.

![Figure 1](https://raw.githubusercontent.com/edomora97/ibuilder/8a4ad5d26fb508b8488b26c63fc9e5c80d51467c/docs/example1.png) |  ![Figure 2](https://raw.githubusercontent.com/edomora97/ibuilder/8a4ad5d26fb508b8488b26c63fc9e5c80d51467c/docs/example2.png) | ![Figure 3](https://raw.githubusercontent.com/edomora97/ibuilder/8a4ad5d26fb508b8488b26c63fc9e5c80d51467c/docs/example3.png)
:-------------------------:|:-------------------------:|:-------------------------:
**Figure 1**: main menu    |  **Figure 2**: `AgeRange` menu | **Figure 3**: main menu again

```rust
use ibuilder::*;

#[derive(IBuilder)]
pub struct Person {
    #[ibuilder(rename = "full name")]
    full_name: String,
    age: AgeRange,
    #[ibuilder(default = 2, rename = "number of hands")]
    num_hands: u64,
}

#[derive(IBuilder, Debug, Eq, PartialEq)]
#[ibuilder(prompt = "How old are you?")]
pub enum AgeRange {
    #[ibuilder(rename = "Less than 13 years old")]
    Child,
    #[ibuilder(rename = "From 13 to 19 years old")]
    Teen,
    #[ibuilder(rename = "20 years or more")]
    Adult,
    #[ibuilder(rename = "I don't want to tell")]
    Unknown,
}

let mut builder = Person::builder();

// * figure 1 *
let options = builder.get_options(); // main menu: select the field to edit
builder.choose(Input::choice("age")).unwrap(); // select the field

// * figure 2 *
let options = builder.get_options(); // age menu
builder.choose(Input::choice("Adult")).unwrap(); // insert the value

let options = builder.get_options(); // back to the main menu
builder.choose(Input::choice("full_name")).unwrap(); // select the field

let options = builder.get_options(); // full_name menu
assert!(options.text_input); // for inserting the string value
builder.choose(Input::text("edomora97")).unwrap(); // insert the value

// * figure 3 *
assert!(builder.is_done());
let options = builder.get_options(); // main menu again, but the "Done" option is available
// chose the "Done" option, the return value is Ok(Some(Person))
let value = builder.choose(Input::Choice(FINALIZE_ID.to_string())).unwrap().unwrap();

assert_eq!(value.full_name, "edomora97");
assert_eq!(value.age, AgeRange::Adult);
assert_eq!(value.num_hands, 2);
```

License: MIT
