# ibuilder_derive

See the documentation of the [`ibuilder`](https://crates.io/ibuilder) create for the details,
you probably are looking for that.

### ibuilder derive macro

Usage:
```rust
#[derive(ibuilder)]
struct Example {
    /// The help message for field1
    field1: i64,
    /// The help message for field2
    #[ibuilder(default = "something")]
    field2: String,
}
```

Will implement the function `Example::builder()` that returns a `Builder<Example>` for
interactively building instances of the `Example` struct.

It will also implement a private struct for keeping the state of the builder and implement the
`NewBuildableValue` trait for `Example`.

License: MIT
