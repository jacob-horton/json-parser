# JSON Parser

This is a JSON parser written in Rust as a learning exercise. It works similarly to [Serde](https://serde.rs/)

There are two ways to parse JSON with this library:
1. Parse any value with `JsonValue`
2. Parse into a struct, with type checking deriving `JsonDeserialise`

For an example using `JsonDeserialise`, check out `main/src/main.rs`


## Base Supported Types

These types are supported out-the-box, but by implementing the `json_parser::Parse` trait, you can add your own types. For structs, you can derive `JsonDeserialise` to do this automatically.

| Category | Types |
| --- | --- |
| Signed integers | `i128`, `i64`, `i32`, `i16`, `i8` |
| Unsigned integers | `u128`, `u64`, `u32`, `u16`, `u8` |
| Floats | `f64`, `f32` |
| Booleans | `bool` |
| Strings | `String` |
| Lists | `Vec<T: Parse>` |
| Objects | `HashMap<String, T: Parse>` |
| Optionals | `Option<T: Parse>` |
| Generic JSON value | `JsonValue` |


## JSON Value

The `JsonValue` enum contains all possible data types of JSON. It is useful when you do not know the JSON structure, or it can dynamically change. However, it is often inconvenient to use (especially for nested data) as you will have to perform lots of `match` statements to get the data out of the enum

Usage:
```rust
use json_parser::*;

fn main() {
    let source = "[1, 2, 3]";
    let result = Parser::parse::<JsonValue>(source);
    println!("{result:?}");
}
```


## JSON Deserialise Derive

This derive macro works similarly to [Serde](https://serde.rs/) - you can apply `#[derive(JsonDeserialise)]` to a struct, then you will be able to parse directly into that struct. For example

Usage:
```rust
use json_parser::*;
use json_parser_macros::JsonDeserialise;

#[derive(Debug, JsonDeserialise)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

fn main() {
    let result = Parser::parse::<Person>(r#"{"name": "John Smith", "age": 42}"#);
    println!("{result:#?}");
}
```

This is extremely useful for two main reasons:
1. It will type check - the parser will fail if the JSON string does not match the provided types
2. It puts the data into an easy-to-use struct (that you define!). This is much easier to work with than the `JsonValue` enum

It also works with nested data structures, and supports using any type that implements `json_parser::Parse` (i.e. the primitives, vectors, any other struct with `JsonDeserialise`, etc.)
