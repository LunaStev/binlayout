// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Independently tabulated offsets for the ELF header example.

#[path = "../examples/support/mod.rs"]
mod support;

use support::{ElfClass, elf_header};

#[test]
fn elf_headers_match_gabi_field_tables() {
    // gABI ELF Header, Listing 2.1; type widths/alignments from Introduction,
    // Tables 1.1 and 1.2. Expected offsets are literal fixtures, not calculated
    // with the library being tested.
    let fields = [
        "e_ident",
        "e_type",
        "e_machine",
        "e_version",
        "e_entry",
        "e_phoff",
        "e_shoff",
        "e_flags",
        "e_ehsize",
        "e_phentsize",
        "e_phnum",
        "e_shentsize",
        "e_shnum",
        "e_shstrndx",
    ];
    for (class, size, align, offsets) in [
        (
            ElfClass::Elf32,
            52,
            4,
            [0, 16, 18, 20, 24, 28, 32, 36, 40, 42, 44, 46, 48, 50],
        ),
        (
            ElfClass::Elf64,
            64,
            8,
            [0, 16, 18, 20, 24, 32, 40, 48, 52, 54, 56, 58, 60, 62],
        ),
    ] {
        let header = elf_header(class).unwrap();
        assert_eq!((header.size(), header.align()), (size, align));
        assert_eq!(header.fields().len(), fields.len());
        for (name, offset) in fields.into_iter().zip(offsets) {
            assert_eq!(header.offset(name), Some(offset), "{class:?}: {name}");
        }
        assert_eq!(header.tail_padding(), 0);
    }
}
