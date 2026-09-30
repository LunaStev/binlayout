// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Independently tabulated offsets for the ELF section-header example.

#[path = "../examples/support/elf_section_header.rs"]
mod section_header;

use section_header::{ElfClass, section_header};

#[test]
fn elf_section_headers_match_gabi_field_tables() {
    // gABI Listing 3.1 defines the field sequence and ELF-class-specific types.
    // The expected values below are literal fixtures, not computed by binlayout.
    let fields = [
        "sh_name",
        "sh_type",
        "sh_flags",
        "sh_addr",
        "sh_offset",
        "sh_size",
        "sh_link",
        "sh_info",
        "sh_addralign",
        "sh_entsize",
    ];
    for (class, size, align, offsets) in [
        (
            ElfClass::Elf32,
            40,
            4,
            [0, 4, 8, 12, 16, 20, 24, 28, 32, 36],
        ),
        (
            ElfClass::Elf64,
            64,
            8,
            [0, 4, 8, 16, 24, 32, 40, 44, 48, 56],
        ),
    ] {
        let header = section_header(class).unwrap();
        assert_eq!((header.size(), header.align()), (size, align));
        assert_eq!(header.fields().len(), fields.len());
        for (name, offset) in fields.into_iter().zip(offsets) {
            assert_eq!(header.offset(name), Some(offset), "{class:?}: {name}");
        }
        assert_eq!(header.tail_padding(), 0);
    }
}
