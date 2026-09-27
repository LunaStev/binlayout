# binlayout

Calculate the size, alignment, field offsets, and padding of binary structures
without reading or writing their data. Intended for binary format and protocol
implementations, compiler tooling, and guest data models in emulators and VMs.

**Host layout and target layout are independent.** All byte sizes, offsets,
alignments, and array counts use `u64`, including on a 32-bit host. There are no
dependencies and no unsafe code. Rust 1.85 or newer is required.

## Installation

```toml
[dependencies]
binlayout = "0.1"
```

The API reference is available with `cargo doc --open`.

## Basic layouts

`Layout` describes byte geometry. Construction validates alignment, which must
be a nonzero power of two. Size can be zero or an unpadded intermediate size.

```rust
use binlayout::Layout;

let a = Layout::new(1, 1)?;
let b = Layout::new(4, 4)?;
let (combined, offset) = a.extend(b)?;
assert_eq!(offset, 4);
assert_eq!(combined.size(), 8);
assert_eq!(combined.align(), 4);
# Ok::<(), binlayout::LayoutError>(())
```

`extend` inserts padding before the next field. `pad_to_align` adds trailing
padding when a record is finished. `repeat(count)` returns an array layout and
element stride, including trailing padding for every element.

Calculations that exceed `u64` return `LayoutError::Overflow`.

## Named structures

```rust
use binlayout::{Layout, StructLayout};

let header = StructLayout::builder()
    .field("tag", Layout::new(1, 1)?)
    .field("length", Layout::new(4, 4)?)
    .field("flags", Layout::new(2, 2)?)
    .build()?;

assert_eq!((header.size(), header.align()), (12, 4));
assert_eq!(header.offset("tag"), Some(0));
assert_eq!(header.offset("length"), Some(4));
assert_eq!(header.offset("flags"), Some(8));
assert_eq!(header.fields()[1].padding_before(), 3);
assert_eq!(header.tail_padding(), 2);
# Ok::<(), binlayout::LayoutError>(())
```

| Byte range | Contents |
| --- | --- |
| 0 | tag |
| 1–3 | padding |
| 4–7 | length |
| 8–9 | flags |
| 10–11 | tail padding |

Fields retain declaration order. Duplicate names are errors; unknown names
return `None`. `header.layout()` can be nested in another structure or repeated
as an array.

For byte-packed wire formats, use `.packed(1)` on the builder: this header then
has offsets `0, 1, 5`, size `7`, and alignment `1`. A larger power-of-two packing
value caps field alignment at that value. `.align(n)` imposes a minimum aggregate
alignment. Packing does not alter padding already inside a nested layout.
`FieldLayout::layout()` preserves the original layout while `FieldLayout::align()`
reports the effective placement alignment.

## Types and targets

`Type` describes fixed-width unsigned/signed integers, floats, pointers, arrays,
ordered structures, and unions. A target supplies scalar and pointer alignment rules.

```rust
use binlayout::{TargetLayout, Type};

let foo = Type::Struct(vec![Type::U8, Type::Pointer]);
let wasm32 = TargetLayout::wasm32();
let x86_64 = TargetLayout::x86_64();

let a = wasm32.layout_of(&foo)?;
let b = x86_64.layout_of(&foo)?;
assert_eq!((a.size(), a.align()), (8, 4));
assert_eq!((b.size(), b.align()), (16, 8));

// Use named types when you also need field offsets.
let named = x86_64.struct_layout([
    ("tag", Type::U8),
    ("data", Type::Pointer),
])?;
assert_eq!(named.offset("data"), Some(8));

let records = Type::array(foo, 3);
assert_eq!(wasm32.layout_of(&records)?.size(), 24);
assert_eq!(x86_64.layout_of(&records)?.size(), 48);
# Ok::<(), binlayout::LayoutError>(())
```

Custom targets require explicit pointer size and alignment. Scalar alignments
default to their byte sizes, and can be overridden individually:

```rust
use binlayout::{Scalar, TargetLayout, Type};

let target = TargetLayout::builder()
    .pointer_size(4)
    .pointer_align(4)
    .scalar_alignment(Scalar::U64, 4)
    .scalar_alignment(Scalar::I64, 4)
    .scalar_alignment(Scalar::F64, 4)
    .build()?;

let record = Type::Struct(vec![Type::U8, Type::U64]);
assert_eq!(target.layout_of(&record)?.size(), 12);
# Ok::<(), binlayout::LayoutError>(())
```

Scalar and pointer sizes must be nonzero multiples of alignment. Signed and
unsigned scalar overrides are independent. For a packed target-specific record,
pass `target.layout_of(&ty)?` into `StructLayout::builder().field(...)`.

The wasm32 and x86-64 presets cover scalar, pointer, and aggregate layouts;
they do not implement complete calling conventions or bit-field rules.

## Unions

Unions place every member at offset zero and round the largest member size up
to the union alignment. They do not add a tag or discriminant.

```rust
use binlayout::{Layout, UnionLayout};

let payload = UnionLayout::builder()
    .field("bytes", Layout::new(9, 1)?)
    .field("number", Layout::new(8, 8)?)
    .build()?;
assert_eq!((payload.size(), payload.align()), (16, 8));
assert_eq!(payload.offset("bytes"), Some(0));
assert_eq!(payload.offset("number"), Some(0));
assert_eq!(payload.tail_padding(), 7);
# Ok::<(), binlayout::LayoutError>(())
```

`UnionLayout` supports the same `.packed(n)` and `.align(n)` builder options as
structures. `Type::Union(vec![...])` and `target.union_layout([("name", ty), ...])`
provide target-dependent union calculations. `tail_padding()` counts bytes past
the largest member, not unused bytes after a smaller alternative.

## Format examples

- `cargo run --example header`: padding and 32/64-bit pointer layouts.
- `cargo run --example elf`: ELF32 and ELF64 headers, with every field offset.
- `cargo run --example protocol`: a byte-packed tagged union.

## `no_std`

Disable the default `std` feature:

```toml
[dependencies]
binlayout = { version = "0.1", default-features = false }
```

`alloc` is still required for names and type/field collections. Basic `Layout`
arithmetic does not allocate.

## License

This Source Code Form is subject to the terms of the Mozilla Public License,
v. 2.0. See [LICENSE](https://github.com/LunaStev/binlayout/blob/master/LICENSE)
or <https://mozilla.org/MPL/2.0/>.
