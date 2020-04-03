//! ![Rust](https://github.com/edomora97/ibuilder/workflows/Rust/badge.svg?branch=master)
//!
//! Interactive builders for structs.
//!
//! This crate provides a way to construct structs interactively, starting from an "empty" state
//! and filling the values of the fields of the struct prompting the user with multiple choices
//! and text inputs. After each choice the internal state of the builder changes.
//!
//! The builder provides the user with interactive menu-like interfaces, keeping the UI abstract
//! and rust type-safeness.
//!
//! The API of this crate is very simple, you start with a struct derived from `ibuilder` and call
//! the auto-generated function `builder()`. This will construct a new custom-built `Builder` to
//! use for the communication. The `Builder` provides two main functions: `get_options()` for
//! getting the current state of the builder and the list of possible options the user can choose,
//! and `choose(input)` that validates and inserts the choice of the user.
//!
//! ## Rationale
//! When building an interactive application (e.g. a Telegram Bot, a Console application) which
//! needs loads of configurations it can be pretty hard to come out with a decent interface without
//! writing loads of code for handling all the corner cases.
//!
//! This crates provides a useful abstraction that takes care of the management of the abstract
//! interface while keeping the API clean. The struct where you needs the data is the actual output
//! of this crate, keeping all the type-safeness.
//!
//! The derive API is inspired by the great [`structopt`](https://docs.rs/structopt) crate.
//!
//! ## Supported features
//! - Deriving any struct with named fields (not `struct Foo(i64)`)
//! - Help messages for the fields from the _first line_ of the rustdoc
//! - Default values for the fields
//! - Nested structures (i.e. custom types)
//! - Supported field types: all numeric types from rust, `bool`, `String`, `char` and `Vec<T>`
//! - Any field type that implementes the `NewBuildableValue` trait.
//!
//! ### Not yet supported, but planned
//! - Hidden fields (that takes the value only from the default)
//! - Enums
//! - Field types: `Option<T>`
//!
//! ## Example of usage
//! ```
//! extern crate ibuilder_derive;
//! use ibuilder_derive::ibuilder;
//! use ibuilder::{FINALIZE_ID, Input};
//!
//! #[derive(ibuilder)]
//! struct Example {
//!     /// This message is used as the help message of the field.
//!     int_field: i64,
//!     string_field: String,
//!     #[ibuilder(default = 123)]
//!     defaulted: i64,
//! }
//!
//! # fn main() {
//! let mut builder = Example::builder();
//!
//! let options = builder.get_options(); // main menu: select the field to edit
//! builder.choose(Input::Choice("int_field".into())).unwrap(); // select the field
//!
//! let options = builder.get_options(); // int_field menu
//! assert!(options.text_input); // for inserting the integer value
//! builder.choose(Input::Text("42".into())).unwrap(); // insert the value
//!
//! let options = builder.get_options(); // back to the main menu
//! builder.choose(Input::Choice("string_field".into())).unwrap(); // select the second field
//!
//! let options = builder.get_options(); // string_field menu
//! assert!(options.text_input); // for inserting the string value
//! builder.choose(Input::Text("hello world!".into())).unwrap(); // insert the value
//!
//! assert!(builder.is_done());
//! let options = builder.get_options(); // main menu again, but the "Done" option is available
//! // chose the "Done" option, the return value is Ok(Some(Example))
//! let value = builder.choose(Input::Choice(FINALIZE_ID.to_string())).unwrap().unwrap();
//!
//! assert_eq!(value.int_field, 42);
//! assert_eq!(value.string_field, "hello world!");
//! assert_eq!(value.defaulted, 123);
//! # }
//! ```

#[cfg(feature = "derive")]
pub use ibuilder_derive::ibuilder;

use std::any::Any;
use std::marker::PhantomData;

use failure::Fail;

use crate::nodes::Node;

pub mod builders;
pub mod nodes;

/// The identifier of the "Done" choice.
pub const FINALIZE_ID: &str = "__finalize";
/// The identifier of the "Back" choice.
pub const BACK_ID: &str = "__back";

/// Interactive builder for creating instances of the struct `T` by communicating. To instantiate a
/// new `Builder` for the type `T`, make `T` derive from `ibuilder` and call `::builder()` on it.
///
/// ## Communication
/// After having instantiated a new `Builder<T>` you can call the `get_options()` method for
/// fetching the list of possible actions that can be done to update the builder. Those options are
/// like menu entries used to move between menus and set the value of the fields.
///
/// The `Options` struct contains a list of possible `Choice`s (like buttons to press) and
/// eventually allow raw text input (like a textbox). For example while editing an integer field
/// the user can insert the new value of the number _as a text_ or can choose to go back to the
/// previous menu by pressing on "back".
///
/// The user's input is communicated to the `Builder` via the `choose` method. It takes an `Input`,
/// a container with the choice of the user, which can be either some `Text` (if the `Options`
/// allowed it), or a `Choice` (whose content is the identifier of the selected option between the
/// ones in the `Options`).
///
/// When the user has filled all the fields of the builder, he can select the "done" options, which
/// will make the `choose` method return `Ok(Some(T))`, signaling the end of the communication.
#[derive(Debug)]
pub struct Builder<T> {
    builder: Box<dyn BuildableValue>,
    current_fields: Vec<String>,
    inner_type: PhantomData<T>,
}

/// The interactive builder for a base type.
pub trait BuildableValue: std::fmt::Debug {
    /// The help message for this value. Will be shown in the interactive menus.
    fn get_help(&self) -> &str;

    /// Try to change the inner value using the provided string.
    fn apply(&mut self, data: &str, current_fields: &[String]) -> Result<(), ChooseError>;

    /// The options to show to the user for setting this value.
    fn get_options(&self, current_fields: &[String]) -> Options;

    /// Whether this value contains itself other values (i.e. it's a struct).
    fn get_subfields(&self, current_fields: &[String]) -> Vec<String>;

    /// Create the tree structure of this value.
    fn to_node(&self) -> Node;

    /// Get the inner value, if present, as an `Any`.
    ///
    /// It's **very important** that the returned `Any` internal type matches the type that this
    /// builder is used for. The `Builder` will downcast this `Any` to the types it's expecting,
    /// panicking in case of mismatched type.
    fn get_value_any(&self) -> Option<Box<dyn Any>>;
}

/// A type that can be built with a `BuildableValue` inside a `Builder`. Keep in mind that the
/// semantics of the generated builder must be compatible with this type, especially looking at the
/// `get_value_any` method.
pub trait NewBuildableValue {
    /// Construct a new `BuildableValue`.
    fn new_builder(help: String) -> Box<dyn BuildableValue>;
}

impl<T: 'static> Builder<T> {
    /// Create a new builder from a `BuildableValue`. Note that the inner type of the
    /// `BuildableValue` must match `T`, otherwise a panic is very likely.
    pub fn from_buildable_value(inner: Box<dyn BuildableValue>) -> Builder<T> {
        Self {
            builder: inner,
            current_fields: vec![],
            inner_type: Default::default(),
        }
    }

    /// Return all the valid options that this builder accepts in the current state.
    pub fn get_options(&self) -> Options {
        // main menu
        if self.current_fields.is_empty() {
            let mut options = self.builder.get_options(&self.current_fields);
            if self.is_done() {
                options.choices.push(Choice {
                    choice_id: FINALIZE_ID.to_string(),
                    text: "Done".to_string(),
                    needs_action: false,
                });
            }
            options
        // field menu
        } else {
            let mut options = self.builder.get_options(&self.current_fields);
            options.choices.push(Choice {
                choice_id: BACK_ID.to_string(),
                text: "Go back".to_string(),
                needs_action: false,
            });
            options
        }
    }

    /// Apply an input to the builder, making it change state. Call again `get_options()` for the
    /// new options.
    ///
    /// Returns `Ok(None)` if the process is not done yet, `Ok(Some(T))` when the user choose to
    /// finish the builder.
    pub fn choose(&mut self, input: Input) -> Result<Option<T>, ChooseError> {
        // main menu
        if self.current_fields.is_empty() {
            match input {
                Input::Text(_) => Err(ChooseError::UnexpectedText),
                Input::Choice(id) => {
                    if id == FINALIZE_ID && self.is_done() {
                        return Ok(Some(self.finalize().expect("Finalize failed")));
                    }
                    for field in self.builder.get_subfields(&self.current_fields) {
                        if field == id {
                            self.current_fields.push(field);
                            return Ok(None);
                        }
                    }
                    Err(ChooseError::UnexpectedChoice)
                }
            }

        // field menu
        } else {
            let data = match input {
                Input::Choice(data) if data == BACK_ID => {
                    self.current_fields.pop();
                    return Ok(None);
                }
                Input::Choice(data) => data,
                Input::Text(data) => data,
            };
            let subfields = self.builder.get_subfields(&self.current_fields);
            if subfields.is_empty() {
                self.builder.apply(&data, &self.current_fields)?;
                self.current_fields.pop();
            } else {
                for subfield in subfields {
                    if subfield == data {
                        self.builder.apply(&data, &self.current_fields)?;
                        self.current_fields.push(subfield);
                        return Ok(None);
                    }
                }
                return Err(ChooseError::UnexpectedChoice);
            }
            Ok(None)
        }
    }

    /// If the process is done try to finalize the process, even if the user hasn't completed the
    /// the selection yet.
    pub fn finalize(&self) -> Result<T, FinalizeError> {
        self.builder
            .get_value_any()
            .ok_or_else(|| FinalizeError::MissingField)
            .map(|r| *r.downcast::<T>().unwrap())
    }

    /// Check if all the fields have been set and the call to `finalize()` will be successful.
    pub fn is_done(&self) -> bool {
        self.builder.get_value_any().is_some()
    }

    /// Return the tree structure of the `Builder` internal state.
    pub fn to_node(&self) -> Node {
        self.builder.to_node()
    }
}

/// The options that the user has for the next choice in the `Builder`.
#[derive(Debug)]
pub struct Options {
    /// A textual message with the query to show to the user.
    pub query: String,
    /// Whether the user can insert raw textual inputs (i.e. `Input::Text`).
    pub text_input: bool,
    /// The list of all the choices the user can use.
    pub choices: Vec<Choice>,
}

/// A single choice that the user can select.
#[derive(Debug)]
pub struct Choice {
    /// Identifier of the choice, may not be shown to the user. Its value has to be used as the
    /// value in `Input::Choice`.
    pub choice_id: String,
    /// Textual message to show to the user about this choice.
    pub text: String,
    /// This choice probably needs to be selected sooner or later because there is a field inside
    /// that is missing.
    pub needs_action: bool,
}

/// An input of the user to the `Builder`.
#[derive(Debug)]
pub enum Input {
    /// The user inserted some raw textual content. Can be used only if the `text_input` field of
    /// the last `Options` was set to `true`.
    Text(String),
    /// The user selected one of the multiple choices in the `Options`. The value should be one of
    /// the `choice_id` inside the list of `Choice`s of the last `Options`.
    Choice(String),
}

/// The `Input` provided to `Builder::choose` was is invalid.
#[derive(Debug, Fail)]
pub enum ChooseError {
    /// The textual input is not valid.
    #[fail(display = "Invalid input: {}", error)]
    InvalidText { error: String },
    /// Provided `Input::Text` even though `Options::text_input` was set to `false`.
    #[fail(display = "Unexpected text")]
    UnexpectedText,
    /// Provided an `Input::Choice` with an invalid id.
    #[fail(display = "Unexpected choice")]
    UnexpectedChoice,
}

/// The finalization of the result failed.
#[derive(Debug, Fail)]
pub enum FinalizeError {
    /// One or more fields were still missing.
    #[fail(display = "There is at least a missing field")]
    MissingField,
}

impl<T: BuildableValue + ?Sized> BuildableValue for Box<T> {
    fn get_help(&self) -> &str {
        self.as_ref().get_help()
    }

    fn apply(&mut self, data: &str, current_fields: &[String]) -> Result<(), ChooseError> {
        self.as_mut().apply(data, current_fields)
    }

    fn get_options(&self, current_fields: &[String]) -> Options {
        self.as_ref().get_options(current_fields)
    }

    fn get_subfields(&self, current_fields: &[String]) -> Vec<String> {
        self.as_ref().get_subfields(current_fields)
    }

    fn to_node(&self) -> Node {
        self.as_ref().to_node()
    }

    fn get_value_any(&self) -> Option<Box<dyn Any>> {
        self.as_ref().get_value_any()
    }
}
