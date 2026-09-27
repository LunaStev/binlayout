// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Shared format definitions used by the ELF example and its specification tests.
//!
//! Sources: https://gabi.xinuos.com/elf/01-intro.html#data-representation
//! and https://gabi.xinuos.com/elf/02-eheader.html#contents-of-the-elf-header

use binlayout::{Layout, LayoutError, StructLayout};

/// The ELF file class, independent of the host or a guest's pointer type.
#[derive(Clone, Copy, Debug)]
pub enum ElfClass {
    /// ELFCLASS32: 32-bit addresses and file offsets.
    Elf32,
    /// ELFCLASS64: 64-bit addresses and file offsets.
    Elf64,
}

/// Calculates the ELF header from the format's explicit size/alignment rules.
///
/// Addresses and file offsets use the ELF class's width. They are not native
/// Rust pointers, and changing the host ABI must not change this definition.
pub fn elf_header(class: ElfClass) -> Result<StructLayout, LayoutError> {
    let address_size = match class {
        ElfClass::Elf32 => 4,
        ElfClass::Elf64 => 8,
    };
    let address = Layout::new(address_size, address_size)?;
    let half = Layout::new(2, 2)?;
    let word = Layout::new(4, 4)?;
    StructLayout::builder()
        .field("e_ident", Layout::new(1, 1)?.repeat(16)?.0)
        .field("e_type", half)
        .field("e_machine", half)
        .field("e_version", word)
        .field("e_entry", address)
        .field("e_phoff", address)
        .field("e_shoff", address)
        .field("e_flags", word)
        .field("e_ehsize", half)
        .field("e_phentsize", half)
        .field("e_phnum", half)
        .field("e_shentsize", half)
        .field("e_shnum", half)
        .field("e_shstrndx", half)
        .build()
}
