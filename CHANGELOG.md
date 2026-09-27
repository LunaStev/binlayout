# Changelog

## 0.1.0

Initial release contents (publication pending).

- Host-independent `u64` layouts with checked extension, alignment, padding,
  and array stride calculations.
- Declaration-order named structures and overlapping named unions, including
  packing caps, aggregate alignment, and padding inspection.
- Fixed-width integer/float, pointer, array, structure, and union type descriptions.
- Explicit target builders, scalar alignment overrides, and wasm32/x86-64 presets.
- Zero-size conventions, invalid-alignment/duplicate-name validation, and
  overflow errors without imposing the host's allocator limits.
- `no_std + alloc`, no dependencies, and forbidden unsafe code.
- Header, ELF32/ELF64, and packed tagged-payload examples.
- Rust 1.85 minimum supported version; invariant, reference, and specification tests.
- MPL-2.0 license and version-tag release workflow.
