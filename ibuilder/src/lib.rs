//! [![Rust](https://github.com/edomora97/ibuilder/workflows/Rust/badge.svg?branch=master)](https://github.com/edomora97/ibuilder/actions?query=workflow%3ARust)
//! [![crates.io](https://img.shields.io/crates/v/ibuilder.svg)](https://crates.io/crates/ibuilder)
//! [![Docs](https://docs.rs/ibuilder/badge.svg)](https://docs.rs/ibuilder)
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
//! The API of this crate is very simple:
//! - Derive a struct (or an enum) from `IBuilder`, including all the structs/enums that it depends
//!   upon;
//! - Call the `builder()` method (from the `Buildable` trait) to get an instance of `Builder<T>`;
//! - By calling `get_options()` on the builder you'll get an object that contains a message to show
//!   the user, a list of possible _choices_ (i.e. buttons to press) and eventually the possibility
//!   to enter some text (i.e. a text box);
//! - By calling `to_node()` on the builder you'll get a tree-like structure with the state of the
//!   builder, highlighting the fields that still need actions;
//! - You choose how to show to the user the options and when the user made the decision you call
//!   `choose(input)` on the builder. This will apply the choice to the state of the structure if
//!   it's valid, or return an error;
//! - When the state is complete (all the required fields are present) a new option is present in
//!   the list: _Done_. If the user selects it `choose` will return an instance of `T`.
//!
//! ## Rationale
//! When building an interactive application (e.g. a Telegram bot or a console application) which
//! needs many configurations it can be pretty cumbersome to come out with a decent interface
//! without spending loads of time writing the logic for handling the parsing and the validation of
//! the input.
//!
//! This crates provides a useful abstraction that allows an easy connection between the data and
//! the user interface. Just by deriving the struct (or enum) that defines your data you can get a
//! safe interface over a possible UI.
//!
//! The derive API is inspired by the great [`structopt`](https://docs.rs/structopt) crate.
//!
//! ## Supported features
//! - Deriving any struct with named fields (or with one unnamed field like `struct Foo(i64)`)
//! - Enums (also with variants with field, but only one if unnamed)
//! - Default values for the fields and default variant for enums
//! - Custom message prompt for fields, structs, enums and variants
//! - Renaming fields, structs and variants for better looking options
//! - Hidden fields (that takes the value only from the default)
//! - Nested structures (i.e. custom types)
//! - Supported field types: all numeric types from rust, `bool`, `String`, `char`, `Box<T>`,
//!   `Vec<T>` and `Option<T>`
//! - Any field type that implementes the `NewBuildableValue` trait
//!
//! ## Example of usage
//!
//! In this example the data is stored inside a struct named `Person` which has 3 fields, one of
//! which has a default value. Deriving from `IBuilder` gives access to the `builder()` method that
//! returns a `Builder<Person>`.
//!
//! ![Figure 1](https://raw.githubusercontent.com/edomora97/ibuilder/8a4ad5d26fb508b8488b26c63fc9e5c80d51467c/docs/example1.png) |  ![Figure 2](https://raw.githubusercontent.com/edomora97/ibuilder/8a4ad5d26fb508b8488b26c63fc9e5c80d51467c/docs/example2.png) | ![Figure 3](https://raw.githubusercontent.com/edomora97/ibuilder/8a4ad5d26fb508b8488b26c63fc9e5c80d51467c/docs/example3.png)
//! :-------------------------:|:-------------------------:|:-------------------------:
//! **Figure 1**: main menu    |  **Figure 2**: `AgeRange` menu | **Figure 3**: main menu again
//!
//! ```
//! use ibuilder::*;
//!
//! #[derive(IBuilder)]
//! pub struct Person {
//!     #[ibuilder(rename = "full name")]
//!     full_name: String,
//!     age: AgeRange,
//!     #[ibuilder(default = 2, rename = "number of hands")]
//!     num_hands: u64,
//! }
//!
//! #[derive(IBuilder, Debug, Eq, PartialEq)]
//! #[ibuilder(prompt = "How old are you?")]
//! pub enum AgeRange {
//!     #[ibuilder(rename = "Less than 13 years old")]
//!     Child,
//!     #[ibuilder(rename = "From 13 to 19 years old")]
//!     Teen,
//!     #[ibuilder(rename = "20 years or more")]
//!     Adult,
//!     #[ibuilder(rename = "I don't want to tell")]
//!     Unknown,
//! }
//!
//! let mut builder = Person::builder();
//!
//! // * figure 1 *
//! let options = builder.get_options(); // main menu: select the field to edit
//! builder.choose(Input::choice("age")).unwrap(); // select the field
//!
//! // * figure 2 *
//! let options = builder.get_options(); // age menu
//! builder.choose(Input::choice("Adult")).unwrap(); // insert the value
//!
//! let options = builder.get_options(); // back to the main menu
//! builder.choose(Input::choice("full_name")).unwrap(); // select the field
//!
//! let options = builder.get_options(); // full_name menu
//! assert!(options.text_input); // for inserting the string value
//! builder.choose(Input::text("edomora97")).unwrap(); // insert the value
//!
//! // * figure 3 *
//! assert!(builder.is_done());
//! let options = builder.get_options(); // main menu again, but the "Done" option is available
//! // chose the "Done" option, the return value is Ok(Some(Person))
//! let value = builder.choose(Input::Choice(FINALIZE_ID.to_string())).unwrap().unwrap();
//!
//! assert_eq!(value.full_name, "edomora97");
//! assert_eq!(value.age, AgeRange::Adult);
//! assert_eq!(value.num_hands, 2);
//! ```

#[cfg(feature = "derive")]
pub use ibuilder_derive::IBuilder;

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
/// new `Builder` for the type `T`, make `T` derive from `IBuilder` and call `builder()` on it from
/// the `Buildable` trait.
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

/// A type that supports being built using a `Builder`. Deriving `IBuilder` an auto-generated
/// implementation for this trait is provided.
pub trait Buildable<T> {
    /// Create a new `Builder<T>` for the current type.
    fn builder() -> Builder<T>;
}

impl<T> Buildable<T> for T
where
    T: NewBuildableValue + 'static,
{
    fn builder() -> Builder<T> {
        Builder::<T>::from_buildable_value(T::new_buildable_value(Default::default()))
    }
}

/// The interactive builder for a base type.
pub trait BuildableValue: std::fmt::Debug {
    /// Try to change the inner value using the provided input.
    fn apply(&mut self, data: Input, current_fields: &[String]) -> Result<(), ChooseError>;

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
    /// Construct a new `BuildableValue` using the provided configuration. Note that using this
    /// constructor instead of the `new` method of the actual builder opaques the inner type.
    fn new_buildable_value(config: BuildableValueConfig<()>) -> Box<dyn BuildableValue>;
}

/// The configuration for customizing the aspect of a `BuildableValue` that produces a value of type
/// `T`.
pub struct BuildableValueConfig<T> {
    /// The default value to use, if `None` there is no default value and the field must be
    /// provided.
    pub default: Option<T>,
    /// The prompt message to show to the user, if `None` a default message is shown.
    pub prompt: Option<String>,
}

impl<T> Default for BuildableValueConfig<T> {
    fn default() -> Self {
        Self {
            default: None,
            prompt: None,
        }
    }
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
            if let Input::Choice(data) = &input {
                if data == FINALIZE_ID && self.is_done() {
                    return Ok(Some(self.finalize().expect("Finalize failed")));
                }
            }

        // field menu
        } else {
            match &input {
                Input::Choice(data) if data == BACK_ID => {
                    self.current_fields.pop();
                    return Ok(None);
                }
                _ => {}
            }
        };
        let subfields = self.builder.get_subfields(&self.current_fields);
        for subfield in subfields {
            match &input {
                Input::Choice(data) => {
                    if subfield == data.as_str() {
                        self.builder.apply(input, &self.current_fields)?;
                        self.current_fields.push(subfield);
                        return Ok(None);
                    }
                }
                Input::Text(_) => {}
            }
        }
        self.builder.apply(input, &self.current_fields)?;
        self.current_fields.pop();
        Ok(None)
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
#[derive(Debug, Eq, PartialEq)]
pub struct Options {
    /// A textual message with the query to show to the user.
    pub query: String,
    /// Whether the user can insert raw textual inputs (i.e. `Input::Text`).
    pub text_input: bool,
    /// The list of all the choices the user can use.
    pub choices: Vec<Choice>,
}

/// A single choice that the user can select.
#[derive(Debug, Eq, PartialEq)]
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
#[derive(Debug, Eq, PartialEq)]
pub enum Input {
    /// The user inserted some raw textual content. Can be used only if the `text_input` field of
    /// the last `Options` was set to `true`.
    Text(String),
    /// The user selected one of the multiple choices in the `Options`. The value should be one of
    /// the `choice_id` inside the list of `Choice`s of the last `Options`.
    Choice(String),
}

impl Input {
    /// The user inserted some raw textual content. Can be used only if the `text_input` field of
    /// the last `Options` was set to `true`.
    pub fn text<S: Into<String>>(text: S) -> Input {
        Input::Text(text.into())
    }
    /// The user selected one of the multiple choices in the `Options`. The value should be one of
    /// the `choice_id` inside the list of `Choice`s of the last `Options`.
    pub fn choice<S: Into<String>>(choice: S) -> Input {
        Input::Choice(choice.into())
    }
}

/// The `Input` provided to `Builder::choose` was is invalid.
#[derive(Debug, Fail, Eq, PartialEq)]
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
#[derive(Debug, Fail, Eq, PartialEq)]
pub enum FinalizeError {
    /// One or more fields were still missing.
    #[fail(display = "There is at least a missing field")]
    MissingField,
}
