// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ELF32 and ELF64 section-header layouts from gABI Listing 3.1.
//!
//! Field widths and alignments are defined by the ELF class, not the host ABI.

use binlayout::{Layout, LayoutError, StructLayout};

/// The ELF class, independent of the host or a guest's pointer type.
#[derive(Clone, Copy, Debug)]
pub enum ElfClass {
    /// ELFCLASS32.
    Elf32,
    /// ELFCLASS64.
    Elf64,
}

/// Calculates a section-header layout using the widths specified by its class.
pub fn section_header(class: ElfClass) -> Result<StructLayout, LayoutError> {
    let word = Layout::new(4, 4)?;
    let (extended_word, address, offset) = match class {
        ElfClass::Elf32 => (word, word, word),
        ElfClass::Elf64 => (Layout::new(8, 8)?, Layout::new(8, 8)?, Layout::new(8, 8)?),
    };

    StructLayout::builder()
        .field("sh_name", word)
        .field("sh_type", word)
        .field("sh_flags", extended_word)
        .field("sh_addr", address)
        .field("sh_offset", offset)
        .field("sh_size", extended_word)
        .field("sh_link", word)
        .field("sh_info", word)
        .field("sh_addralign", extended_word)
        .field("sh_entsize", extended_word)
        .build()
}
