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

The API of this crate is very simple, you start with a struct derived from `ibuilder` and call
the auto-generated function `builder()` from the `Buildable` trait. This will construct a new
custom-built `Builder` to use for the communication. The `Builder` provides two main functions:
`get_options()` for getting the current state of the builder and the list of possible options
the user can choose, and `choose(input)` that validates and inserts the choice of the user.

### Rationale
When building an interactive application (e.g. a Telegram Bot, a Console application) which
needs loads of configurations it can be pretty hard to come out with a decent interface without
writing loads of code for handling all the corner cases.

This crates provides a useful abstraction that takes care of the management of the abstract
interface while keeping the API clean. The struct where you needs the data is the actual output
of this crate, keeping all the type-safeness.

The derive API is inspired by the great [`structopt`](https://docs.rs/structopt) crate.

### Supported features
- Deriving any struct with named fields (not `struct Foo(i64)`)
- Default values for the fields
- Nested structures (i.e. custom types)
- Supported field types: all numeric types from rust, `bool`, `String`, `char` and `Vec<T>`
- Any field type that implementes the `NewBuildableValue` trait.

#### Not yet supported, but planned
- Hidden fields (that takes the value only from the default)
- Enums
- Field types: `Option<T>`

### Example of usage
```rust
use ibuilder::*;

#[derive(ibuilder)]
struct Example {
    int_field: i64,
    string_field: String,
    #[ibuilder(default = 123)]
    defaulted: i64,
}

let mut builder = Example::builder();

let options = builder.get_options(); // main menu: select the field to edit
builder.choose(Input::choice("int_field")).unwrap(); // select the field

let options = builder.get_options(); // int_field menu
assert!(options.text_input); // for inserting the integer value
builder.choose(Input::text("42")).unwrap(); // insert the value

let options = builder.get_options(); // back to the main menu
builder.choose(Input::choice("string_field")).unwrap(); // select the second field

let options = builder.get_options(); // string_field menu
assert!(options.text_input); // for inserting the string value
builder.choose(Input::text("hello world!")).unwrap(); // insert the value

assert!(builder.is_done());
let options = builder.get_options(); // main menu again, but the "Done" option is available
// chose the "Done" option, the return value is Ok(Some(Example))
let value = builder.choose(Input::Choice(FINALIZE_ID.to_string())).unwrap().unwrap();

assert_eq!(value.int_field, 42);
assert_eq!(value.string_field, "hello world!");
assert_eq!(value.defaulted, 123);
```

License: MIT
