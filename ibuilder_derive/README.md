# ibuilder_derive

[![crates.io](https://img.shields.io/crates/v/ibuilder_derive.svg)](https://crates.io/crates/ibuilder_derive)
[![Docs](https://docs.rs/ibuilder_derive/badge.svg)](https://docs.rs/ibuilder_derive)

See the documentation of the [`ibuilder`](https://crates.io/crates/ibuilder) create for the details,
you probably are looking for that.

### ibuilder derive macro

Usage:
```rust
#[derive(IBuilder)]
struct Example {
    field1: i64,
    #[ibuilder(default = "something")]
    field2: String,
}
```

Will implement the trait `ibuilder::Buildable` for `Example`, prodiding the `builder()` method
for getting a `ibuilder::Builder`.

It will also implement a private struct for keeping the state of the builder and implement the
`NewBuildableValue` trait for `Example`, allowing it to be inside a fields of other derived
types.

License: MIT
