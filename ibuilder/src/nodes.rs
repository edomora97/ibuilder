//! Allow generic display of the structures using a tree representation.
//!
//! The `Builder` exposes the `to_node()` method that returns a tree-like structures with all the
//! visible fields of the builder. This structure can be used for pretty-printing the internal
//! builder state is a customized manner.

/// A `Node` of the tree, it represents an item that can be interacted with.
#[derive(Debug)]
pub enum Node {
    /// The `Node` is a leaf node of the tree, i.e. it doesn't contains subfields, just a value.
    Leaf(Field),
    /// The `Node` is actually composed by inner fields, for example a `Vec` is composed by items
    /// and a `struct` by fields.
    Composite(String, Vec<FieldKind>),
}

/// A field of a composite structure. The field may be named (like in `struct`s), or be unnamed
/// (like in `Vec`).
#[derive(Debug)]
pub enum FieldKind {
    /// The field is named, the first item is the name of the field, the second is the inner node of
    /// it.
    Named(String, Node),
    /// The field does not have a name, the value is the inner node of the item.
    Unnamed(Node),
}

/// A leaf field of the tree structure.
#[derive(Debug)]
pub enum Field {
    /// The field is valid and the textual representation of it is provided.
    String(String),
    /// The field is not present yet.
    Missing,
}
