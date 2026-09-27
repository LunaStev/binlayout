// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Print field placement and compare pointer layouts across targets.

use binlayout::{Layout, LayoutError, StructLayout, TargetLayout, Type};

fn main() -> Result<(), LayoutError> {
    let header = StructLayout::builder()
        .field("tag", Layout::new(1, 1)?)
        .field("length", Layout::new(4, 4)?)
        .field("flags", Layout::new(2, 2)?)
        .build()?;

    println!("Header: size={}, align={}", header.size(), header.align());
    for field in header.fields() {
        println!(
            "  {:<8} offset={}, size={}, align={}, padding_before={}",
            field.name(),
            field.offset(),
            field.layout().size(),
            field.align(),
            field.padding_before(),
        );
    }
    println!("  tail padding={}", header.tail_padding());

    for (name, target) in [
        ("wasm32", TargetLayout::wasm32()),
        ("x86_64", TargetLayout::x86_64()),
    ] {
        let pointer_record = target.struct_layout([("tag", Type::U8), ("data", Type::Pointer)])?;
        println!(
            "{name}: Foo size={}, align={}, data offset={}",
            pointer_record.size(),
            pointer_record.align(),
            pointer_record.offset("data").expect("data field exists"),
        );
    }
    Ok(())
}
