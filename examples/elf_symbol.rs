// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Calculate ELF32 and ELF64 symbol-table entry layouts without parsing a file.

#[path = "support/elf_symbol.rs"]
mod support;

use binlayout::LayoutError;
use support::{ElfClass, elf_symbol_entry};

fn main() -> Result<(), LayoutError> {
    for class in [ElfClass::Elf32, ElfClass::Elf64] {
        let symbol = elf_symbol_entry(class)?;
        println!(
            "{class:?} symbol: size={}, align={}",
            symbol.size(),
            symbol.align()
        );
        for field in symbol.fields() {
            println!(
                "  {:<12} offset={:>2}, size={}",
                field.name(),
                field.offset(),
                field.layout().size()
            );
        }
    }
    Ok(())
}
