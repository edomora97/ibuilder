//! Module with the implementors of `BuildableValue` for the various standard types.

use std::any::Any;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::str::FromStr;

use crate::nodes::{Field, FieldKind, Node};
use crate::{
    BuildableValue, BuildableValueConfig, Choice, ChooseError, Input, NewBuildableValue, Options,
};

macro_rules! type_builder_boilerplate {
    (normal) => {
        fn get_subfields(&self, _: &[String]) -> Vec<String> {
            vec![]
        }

        fn to_node(&self) -> Node {
            if let Some(value) = &self.value {
                Node::Leaf(Field::String(value.to_string()))
            } else {
                Node::Leaf(Field::Missing)
            }
        }
    };
    (path) => {
        fn get_subfields(&self, _: &[String]) -> Vec<String> {
            vec![]
        }

        fn to_node(&self) -> Node {
            if let Some(value) = &self.value {
                Node::Leaf(Field::String(
                    value.as_os_str().to_string_lossy().to_string(),
                ))
            } else {
                Node::Leaf(Field::Missing)
            }
        }
    };
}

macro_rules! type_builder_struct {
    ($base:ty, $name:ident, $query:expr) => {
        type_builder_struct!(
            $base,
            $name,
            $query,
            concat!("Builder for the type `", stringify!($base), "`")
        );
    };
    ($base:ty, $name:ident, $query:expr, $docstring:expr) => {
        #[doc = $docstring]
        #[derive(Debug)]
        pub struct $name {
            /// The current value.
            pub value: Option<$base>,
            /// The message to show to the user.
            pub prompt: String,
        }

        impl $name {
            /// Make a new instance of the builder.
            pub fn new(config: BuildableValueConfig<$base>) -> Self {
                Self {
                    value: config.default,
                    prompt: config.prompt.unwrap_or_else(|| $query.to_string()),
                }
            }
        }
    };
}

macro_rules! type_builder {
    ($base:ty, $name:ident, $query:expr) => {
        type_builder!(
            $base,
            $name,
            $query,
            normal
        );
    };
    ($base:ty, $name:ident, $query:expr, $variant:tt) => {
        type_builder!(
            @,
            $base,
            $name,
            $query,
            concat!("Builder for the type `", stringify!($base), "`"),
            $variant
        );
    };
    (@, $base:ty, $name:ident, $query:expr, $docstring:expr, $variant:tt) => {
        type_builder_struct!($base, $name, $query, $docstring);

        impl BuildableValue for $name {
            type_builder_boilerplate!($variant);

            fn apply(&mut self, data: Input, current_fields: &[String]) -> Result<(), ChooseError> {
                if !current_fields.is_empty() {
                    panic!(
                        "{}.apply() called with non empty fields: {:?}",
                        stringify!($name),
                        current_fields
                    );
                }
                match data {
                    Input::Text(data) => {
                        self.value = Some(<$base>::from_str(&data).map_err(|e| {
                            ChooseError::InvalidText {
                                error: e.to_string(),
                            }
                        })?);
                    }
                    _ => return Err(ChooseError::UnexpectedChoice),
                }
                Ok(())
            }

            fn get_options(&self, current_fields: &[String]) -> Options {
                if !current_fields.is_empty() {
                    panic!(
                        "{}.get_options() called with non empty fields: {:?}",
                        stringify!($name),
                        current_fields
                    );
                }
                Options {
                    query: self.prompt.clone(),
                    text_input: true,
                    choices: vec![],
                }
            }

            fn get_value_any(&self) -> Option<Box<dyn Any>> {
                self.value.clone().map(|x| Box::new(x) as Box<dyn Any>)
            }
        }

        impl NewBuildableValue for $base {
            fn new_buildable_value(config: BuildableValueConfig<()>) -> Box<dyn BuildableValue> {
                Box::new($name::new(BuildableValueConfig {
                    default: None,
                    prompt: config.prompt,
                }))
            }
        }
    };
}

type_builder!(i8, I8Builder, "Type an integer");
type_builder!(i16, I16Builder, "Type an integer");
type_builder!(i32, I32Builder, "Type an integer");
type_builder!(i64, I64Builder, "Type an integer");
type_builder!(u8, U8Builder, "Type an integer");
type_builder!(u16, U16Builder, "Type an integer");
type_builder!(u32, U32Builder, "Type an integer");
type_builder!(u64, U64Builder, "Type an integer");
type_builder!(isize, IsizeBuilder, "Type an integer");
type_builder!(usize, UsizeBuilder, "Type an integer");
type_builder!(f32, F32Builder, "Type an integer");
type_builder!(f64, F64Builder, "Type an integer");
type_builder!(String, StringBuilder, "Type a string");
type_builder!(char, CharBuilder, "Type a char");
type_builder!(PathBuf, PathBufBuilder, "Type a path", path);

type_builder_struct!(bool, BoolBuilder, "True or false?");

impl BuildableValue for BoolBuilder {
    type_builder_boilerplate!(normal);

    fn apply(&mut self, data: Input, current_fields: &[String]) -> Result<(), ChooseError> {
        if !current_fields.is_empty() {
            panic!(
                "BoolBuilder.apply() called with non empty fields: {:?}",
                current_fields
            );
        }
        match data {
            Input::Choice(data) => match data.as_str() {
                "true" => self.value = Some(true),
                "false" => self.value = Some(false),
                _ => return Err(ChooseError::UnexpectedChoice),
            },
            Input::Text(_) => return Err(ChooseError::UnexpectedText),
        }
        Ok(())
    }

    fn get_options(&self, current_fields: &[String]) -> Options {
        if !current_fields.is_empty() {
            panic!(
                "BoolBuilder.get_options() called with non empty fields: {:?}",
                current_fields
            );
        }
        Options {
            query: self.prompt.clone(),
            text_input: false,
            choices: vec![
                Choice {
                    choice_id: "true".to_string(),
                    text: "true".to_string(),
                    needs_action: false,
                },
                Choice {
                    choice_id: "false".to_string(),
                    text: "false".to_string(),
                    needs_action: false,
                },
            ],
        }
    }

    fn get_value_any(&self) -> Option<Box<dyn Any>> {
        self.value.map(|x| Box::new(x) as Box<dyn Any>)
    }
}

/// Builder for the type `Vec<T>`.
///
/// The type parameters are:
/// - `B`: the type of the builder that produces `T`
/// - `T`: the type of the items of the final `Vec`
///
/// The state machine that this builder implements is a bit complex since it has to handle
/// insertions, deletions and updates and it looks like this:
///
/// ```text
///                 |
///                 v
///            +-------------+    __new    +-------------+  <B> specific
///  +-------> |  empty      | ----------> | not empty   | ------>>>
///  |         |  main       |             | edit        |
///  |         +-------------+             +-------------+
///  |                                         ^    |
///  |                                         |    |
///  |         +-------------+  index / __new  |    | __back
///  +-------> |  not empty  | ----------------+    |
///  |         |  main       | <--------------------+
///  | index   +-------------+
///  |            ^    |
///  |     __back |    | __remove
///  |            |    v
///  |         +-------------+
///  +-------- |  not empty  |
///            |  remove     |
///            +-------------+
/// ```
///
/// When `__new` is applied a new item is pushed at the back of the `Vec` and when `__new` is to
/// be considered as an index it refers to the last element of the `Vec`.
pub struct VecBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    items: Vec<Box<dyn BuildableValue>>,
    inner_type: PhantomData<T>,
    prompt: String,
}

impl<T> std::fmt::Debug for VecBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecBuilder")
            .field("items", &self.items)
            .finish()
    }
}

impl<T> NewBuildableValue for Vec<T>
where
    T: NewBuildableValue + 'static,
{
    fn new_buildable_value(config: BuildableValueConfig<()>) -> Box<dyn BuildableValue> {
        Box::new(VecBuilder::<T> {
            items: Vec::new(),
            inner_type: Default::default(),
            prompt: config
                .prompt
                .unwrap_or_else(|| "Select an action".to_string()),
        })
    }
}

impl<T> BuildableValue for VecBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    fn apply(&mut self, data: Input, current_fields: &[String]) -> Result<(), ChooseError> {
        // vec main menu
        if current_fields.is_empty() {
            match data {
                Input::Choice(data) if data == "__new" => {
                    self.items.push(T::new_buildable_value(Default::default()));
                }
                Input::Choice(data) => {
                    if data != "__remove" {
                        // check that the inserted index is valid
                        let index =
                            usize::from_str(&data).map_err(|_| ChooseError::UnexpectedChoice)?;
                        if index >= self.items.len() {
                            return Err(ChooseError::UnexpectedChoice);
                        }
                    }
                }
                _ => return Err(ChooseError::UnexpectedText),
            }
        // remove item or apply to element
        } else {
            let field = &current_fields[0];
            let rest = &current_fields[1..];
            match field.as_str() {
                "__remove" => match data {
                    Input::Choice(choice) => {
                        let index =
                            usize::from_str(&choice).map_err(|_| ChooseError::UnexpectedChoice)?;
                        if index >= self.items.len() {
                            return Err(ChooseError::UnexpectedChoice);
                        }
                        self.items.remove(index);
                    }
                    Input::Text(_) => return Err(ChooseError::UnexpectedText),
                },
                "__new" => {
                    self.items
                        .last_mut()
                        .expect("Vec __new didn't push")
                        .apply(data, rest)?;
                }
                index => {
                    let index = usize::from_str(index)
                        .unwrap_or_else(|_| panic!("Invalid index for vec: {}", index));
                    self.items[index].apply(data, rest)?;
                }
            }
        }
        Ok(())
    }

    fn get_options(&self, current_fields: &[String]) -> Options {
        // vec main manu
        if current_fields.is_empty() {
            let mut choices = vec![Choice {
                choice_id: "__new".to_string(),
                text: "New element".to_string(),
                needs_action: false,
            }];
            if !self.items.is_empty() {
                choices.push(Choice {
                    choice_id: "__remove".to_string(),
                    text: "Remove element".to_string(),
                    needs_action: false,
                });
                for i in 0..self.items.len() {
                    choices.push(Choice {
                        choice_id: i.to_string(),
                        text: format!("Edit item {}", i),
                        needs_action: self.items[i].get_value_any().is_none(),
                    });
                }
            }
            Options {
                query: self.prompt.clone(),
                text_input: false,
                choices,
            }
        // item menu
        } else {
            let field = &current_fields[0];
            let rest = &current_fields[1..];
            match field.as_str() {
                // select the item to remove
                "__remove" => {
                    let mut choices = Vec::new();
                    for i in 0..self.items.len() {
                        choices.push(Choice {
                            choice_id: i.to_string(),
                            text: format!("Remove item {}", i),
                            needs_action: false,
                        });
                    }
                    Options {
                        query: "Select the item to remove".to_string(),
                        text_input: false,
                        choices,
                    }
                }
                // last action was __new, now inside the last item menu
                "__new" => self
                    .items
                    .last()
                    .expect("Vec __new didn't push")
                    .get_options(rest),
                // edit one of the items
                index => {
                    let index = usize::from_str(index)
                        .unwrap_or_else(|_| panic!("Invalid index for vec: {}", index));
                    self.items[index].get_options(rest)
                }
            }
        }
    }

    fn get_subfields(&self, current_fields: &[String]) -> Vec<String> {
        // main manu
        if current_fields.is_empty() {
            if self.items.is_empty() {
                vec!["__new".into()]
            } else {
                let mut res = vec!["__new".into(), "__remove".into()];
                for i in 0..self.items.len() {
                    res.push(i.to_string());
                }
                res
            }
        } else {
            let field = &current_fields[0];
            let rest = &current_fields[1..];
            match field.as_str() {
                // just select the item to remove
                "__remove" => vec![],
                "__new" => self
                    .items
                    .last()
                    .expect("Vec __new didn't push")
                    .get_subfields(rest),
                index => {
                    let index = usize::from_str(index)
                        .unwrap_or_else(|_| panic!("Invalid index for vec: {}", index));
                    self.items[index].get_subfields(rest)
                }
            }
        }
    }

    fn to_node(&self) -> Node {
        let items = self
            .items
            .iter()
            .map(|i| FieldKind::Unnamed(i.to_node()))
            .collect();
        // Vec has no name
        Node::Composite("".into(), items)
    }

    fn get_value_any(&self) -> Option<Box<dyn Any>> {
        let mut results: Vec<T> = Vec::with_capacity(self.items.len());
        for item in &self.items {
            results.push(*item.get_value_any()?.downcast::<T>().unwrap());
        }
        Some(Box::new(results))
    }
}

/// Builder for the type `Box<T>`.
pub struct BoxBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    value: Box<dyn BuildableValue>,
    inner_type: PhantomData<T>,
}

impl<T> std::fmt::Debug for BoxBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxBuilder")
            .field("value", &self.value)
            .finish()
    }
}

impl<T> NewBuildableValue for Box<T>
where
    T: NewBuildableValue + 'static,
{
    fn new_buildable_value(config: BuildableValueConfig<()>) -> Box<dyn BuildableValue> {
        Box::new(BoxBuilder::<T> {
            value: T::new_buildable_value(config),
            inner_type: Default::default(),
        })
    }
}

impl<T> BuildableValue for BoxBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    fn apply(&mut self, data: Input, current_fields: &[String]) -> Result<(), ChooseError> {
        self.value.apply(data, current_fields)
    }

    fn get_options(&self, current_fields: &[String]) -> Options {
        self.value.get_options(current_fields)
    }

    fn get_subfields(&self, current_fields: &[String]) -> Vec<String> {
        self.value.get_subfields(current_fields)
    }

    fn to_node(&self) -> Node {
        self.value.to_node()
    }

    fn get_value_any(&self) -> Option<Box<dyn Any>> {
        Some(Box::new(Box::new(
            *self.value.get_value_any()?.downcast::<T>().unwrap(),
        )))
    }
}

/// Builder for the type `Option<T>`.
pub struct OptionBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    value: Option<Box<dyn BuildableValue>>,
    inner_type: PhantomData<T>,
    prompt: String,
}

impl<T> std::fmt::Debug for OptionBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptionBuilder")
            .field("value", &self.value)
            .finish()
    }
}

impl<T> NewBuildableValue for Option<T>
where
    T: NewBuildableValue + 'static,
{
    fn new_buildable_value(config: BuildableValueConfig<()>) -> Box<dyn BuildableValue> {
        Box::new(OptionBuilder::<T> {
            value: None,
            inner_type: Default::default(),
            prompt: config
                .prompt
                .unwrap_or_else(|| "Choose an option".to_string()),
        })
    }
}

impl<T> BuildableValue for OptionBuilder<T>
where
    T: NewBuildableValue + 'static,
{
    fn apply(&mut self, data: Input, current_fields: &[String]) -> Result<(), ChooseError> {
        if current_fields.is_empty() {
            match data {
                Input::Choice(data) => match data.as_str() {
                    "__remove" => self.value = None,
                    "__edit" => {}
                    "__set" => self.value = Some(T::new_buildable_value(Default::default())),
                    _ => return Err(ChooseError::UnexpectedChoice),
                },
                Input::Text(_) => return Err(ChooseError::UnexpectedText),
            }
            Ok(())
        } else {
            let field = &current_fields[0];
            let rest = &current_fields[1..];
            if field == "__edit" || field == "__set" {
                self.value.as_mut().unwrap().apply(data, rest)
            } else {
                unreachable!("Unexpected field: {}", field);
            }
        }
    }

    fn get_options(&self, current_fields: &[String]) -> Options {
        if current_fields.is_empty() {
            let choices = match self.value {
                Some(_) => vec![
                    Choice {
                        choice_id: "__remove".to_string(),
                        text: "Remove value".to_string(),
                        needs_action: false,
                    },
                    Choice {
                        choice_id: "__edit".to_string(),
                        text: "Edit value".to_string(),
                        needs_action: false,
                    },
                ],
                None => vec![Choice {
                    choice_id: "__set".to_string(),
                    text: "Set value".to_string(),
                    needs_action: false,
                }],
            };
            Options {
                query: self.prompt.clone(),
                text_input: false,
                choices,
            }
        } else {
            let field = &current_fields[0];
            let rest = &current_fields[1..];
            if field == "__edit" || field == "__set" {
                self.value.as_ref().unwrap().get_options(rest)
            } else {
                unreachable!("Unexpected field: {}", field);
            }
        }
    }

    fn get_subfields(&self, current_fields: &[String]) -> Vec<String> {
        if current_fields.is_empty() {
            match self.value {
                Some(_) => vec!["__edit".to_string()],
                None => vec!["__set".to_string()],
            }
        } else {
            let field = &current_fields[0];
            let rest = &current_fields[1..];
            if field == "__edit" || field == "__set" {
                self.value.as_ref().unwrap().get_subfields(rest)
            } else {
                unreachable!("Unexpected field: {}", field);
            }
        }
    }

    fn to_node(&self) -> Node {
        match &self.value {
            Some(inner) => inner.to_node(),
            None => Node::Leaf(Field::String("None".into())),
        }
    }

    fn get_value_any(&self) -> Option<Box<dyn Any>> {
        match &self.value {
            Some(inner) => Some(Box::new(Some(
                *inner.get_value_any()?.downcast::<T>().unwrap(),
            ))),
            None => Some(Box::new(None::<T>)),
        }
    }
}
