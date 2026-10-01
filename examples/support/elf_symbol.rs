// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ELF symbol-table entry layouts, kept separate from other ELF examples.

use binlayout::{Layout, LayoutError, StructLayout};

/// The ELF file class, independent of host pointer width.
#[derive(Clone, Copy, Debug)]
pub enum ElfClass {
    Elf32,
    Elf64,
}

/// Calculates an ELF32 or ELF64 symbol-table entry layout from its specification.
pub fn elf_symbol_entry(class: ElfClass) -> Result<StructLayout, LayoutError> {
    let byte = Layout::new(1, 1)?;
    let half = Layout::new(2, 2)?;
    let word = Layout::new(4, 4)?;
    let address = match class {
        ElfClass::Elf32 => Layout::new(4, 4)?,
        ElfClass::Elf64 => Layout::new(8, 8)?,
    };
    let symbol_size = match class {
        ElfClass::Elf32 => word,
        ElfClass::Elf64 => Layout::new(8, 8)?,
    };

    match class {
        ElfClass::Elf32 => StructLayout::builder()
            .field("st_name", word)
            .field("st_value", address)
            .field("st_size", symbol_size)
            .field("st_info", byte)
            .field("st_other", byte)
            .field("st_shndx", half)
            .build(),
        ElfClass::Elf64 => StructLayout::builder()
            .field("st_name", word)
            .field("st_info", byte)
            .field("st_other", byte)
            .field("st_shndx", half)
            .field("st_value", address)
            .field("st_size", symbol_size)
            .build(),
    }
}
