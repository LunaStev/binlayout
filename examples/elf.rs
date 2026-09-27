// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Calculate ELF32 and ELF64 header layouts without reading or writing a file.

mod support;

use binlayout::LayoutError;
use support::{ElfClass, elf_header};

fn main() -> Result<(), LayoutError> {
    for class in [ElfClass::Elf32, ElfClass::Elf64] {
        let header = elf_header(class)?;
        println!(
            "{class:?}: size={}, align={}",
            header.size(),
            header.align()
        );
        for field in header.fields() {
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
