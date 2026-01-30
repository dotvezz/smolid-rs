# smolid

`smolid` is a 64-bit (8-byte) ID scheme for Rust that is URL-friendly, temporally sortable, and optimized for database locality. It is designed for use cases where 128-bit UUIDs are unnecessarily large, but a standard auto-incrementing integer is insufficient.

- **URL-Friendly**: Encoded as short, unpadded base32 strings (e.g., `acpje64aeyez6`).
- **Temporally Sortable**: Most significant bits contain a millisecond-precision timestamp.
- **Compact**: Fits into a standard 64-bit integer (`u64` in Rust, `bigint` in PostgreSQL).
- **Type-Aware**: Supports embedding an optional 7-bit type identifier directly into the ID.

## Related Links

- [dotvezz/smolid](https://github.com/dotvezz/smolid): The original Go implementation.
- [pg_smolid](https://github.com/dotvezz/pg_smolid): A pgrx extension for using `smolid` natively in PostgreSQL.
- [mirorac/smolid-js](https://github.com/mirorac/smolid-js) is a reimplementation of `smolid` in Javascript and Typescript by [@mirorac](https://github.com/mirorac)
    - NPM Link: https://www.npmjs.com/package/smolid

## ID Structure

A `smolid` consists of 64 bits partitioned as follows:

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          time_high                            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|    time_low     |ver|t| rand  | type or rand|       rand      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- **Timestamp (41 bits)**: Millisecond-precision timestamp with a custom epoch (2025-01-01). Valid until 2094.
- **Version (2 bits)**: Reserved for versioning (v1 is `01`).
- **Type Flag (1 bit)**: Boolean flag indicating if the Type field is used.
- **Random/Type (20 bits)**:
    - If Type Flag is unset: 20 bits of pseudo-random data.
    - If Type Flag is set: 4 bits of random data, a 7-bit Type ID, and 9 bits of random data.

## Usage

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
smolid = "1.0.0"
```


### Basic Example

```rust
use smolid::Smolid;
use std::str::FromStr;

// Generate a new ID
let id = Smolid::new();
println!("{}", id); // e.g., "acpje64aeyez6"

// Get raw u64
let n = id.as_u64();

// Parse from string
let parsed = Smolid::from_str("acpje64aeyez6").unwrap();
```

### Using Embedded Types

Embedded types allow you to identify the resource type (e.g., User, Post, Comment) directly from the ID itself.

```rust
use smolid::Smolid;

const TYPE_USER: u16 = 1;
const TYPE_POST: u16 = 2;

// Create an ID with a type
let id = Smolid::new_with_type(TYPE_USER).unwrap();

// Check type later
if let Some(t) = id.get_type() {
    match t {
        TYPE_USER => println!("This is a User ID"),
        _ => println!("Other type"),
    }
}

// Helper method
assert!(id.is_of_type(TYPE_USER));
```

## Considerations

### Uniqueness and Collisions

`smolid` provides 13 to 20 bits of entropy per millisecond. This is "unique-enough" for many applications but is not a replacement for UUIDs in high-concurrency environments with massive write volumes (e.g., >10,000 IDs per second).
