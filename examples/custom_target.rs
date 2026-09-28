// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Compare the same record under two explicit custom target models.

use binlayout::{LayoutError, Scalar, TargetLayout, Type};

fn main() -> Result<(), LayoutError> {
    let u64_align_4 = TargetLayout::builder()
        .pointer_size(4)
        .pointer_align(4)
        .scalar_alignment(Scalar::U64, 4)
        .build()?;
    let u64_align_8 = TargetLayout::builder()
        .pointer_size(4)
        .pointer_align(4)
        .scalar_alignment(Scalar::U64, 8)
        .build()?;
    let fields = || [("tag", Type::U8), ("value", Type::U64)];

    for (name, target, expected_offset, expected_size) in [
        ("U64 alignment 4", u64_align_4, 4, 12),
        ("U64 alignment 8", u64_align_8, 8, 16),
    ] {
        let record = target.struct_layout(fields())?;
        let value_offset = record.offset("value").expect("value field exists");
        let pointer = target.pointer_layout();
        let i64_align = target.scalar_layout(Scalar::I64).align();
        let f64_align = target.scalar_layout(Scalar::F64).align();

        assert_eq!(
            (value_offset, record.size()),
            (expected_offset, expected_size)
        );
        assert_eq!((pointer.size(), pointer.align()), (4, 4));
        assert_eq!((i64_align, f64_align), (8, 8));

        println!(
            "{name}: value offset={value_offset}, record size={}, pointer size/alignment={}/{}, I64/F64 alignment={i64_align}/{f64_align}",
            record.size(),
            pointer.size(),
            pointer.align(),
        );
    }

    Ok(())
}
