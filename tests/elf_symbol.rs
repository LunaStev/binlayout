// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Independently tabulated offsets for ELF symbol table entry layouts.

#[path = "../examples/support/elf_symbol.rs"]
mod elf_symbol;

use elf_symbol::{ElfClass, elf_symbol_entry};

#[test]
fn elf_symbol_entries_match_gabi_field_tables() {
    // gABI symbol table entries, Listing 5.1. Expected values are literal
    // specification fixtures, not calculated with the library being tested.
    let fields = [
        "st_name", "st_value", "st_size", "st_info", "st_other", "st_shndx",
    ];
    for (class, size, align, offsets) in [
        (ElfClass::Elf32, 16, 4, [0, 4, 8, 12, 13, 14]),
        (ElfClass::Elf64, 24, 8, [0, 8, 16, 4, 5, 6]),
    ] {
        let symbol = elf_symbol_entry(class).unwrap();
        assert_eq!((symbol.size(), symbol.align()), (size, align));
        assert_eq!(symbol.fields().len(), fields.len());
        for (name, offset) in fields.into_iter().zip(offsets) {
            assert_eq!(symbol.offset(name), Some(offset), "{class:?}: {name}");
        }
        assert_eq!(symbol.tail_padding(), 0);
    }
}
