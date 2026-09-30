// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Calculate ELF32 and ELF64 section-header layouts without reading or writing a file.

#[path = "support/elf_section_header.rs"]
mod section_header;

use binlayout::LayoutError;
use section_header::{ElfClass, section_header};

fn main() -> Result<(), LayoutError> {
    for class in [ElfClass::Elf32, ElfClass::Elf64] {
        let header = section_header(class)?;
        println!(
            "{class:?}: size={}, align={}",
            header.size(),
            header.align()
        );
        for field in header.fields() {
            println!(
                "  {:<13} offset={:>2}, size={}",
                field.name(),
                field.offset(),
                field.layout().size()
            );
        }
    }
    Ok(())
}
