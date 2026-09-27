// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Host-independent target rules and recursive type layouts.

use binlayout::{Layout, LayoutError, Scalar, TargetLayout, Type};

#[test]
fn same_type_has_different_target_layouts() {
    let foo = Type::Struct(vec![Type::U8, Type::Pointer]);
    for (target, size, align) in [
        (TargetLayout::wasm32(), 8, 4),
        (TargetLayout::x86_64(), 16, 8),
    ] {
        assert_eq!(
            target.layout_of(&foo).unwrap(),
            Layout::new(size, align).unwrap()
        );
        let named = target
            .struct_layout([("tag", Type::U8), ("data", Type::Pointer)])
            .unwrap();
        assert_eq!(named.layout(), target.layout_of(&foo).unwrap());
        assert_eq!(named.offset("data"), Some(align));
        assert_eq!(target.pointer_layout(), Layout::new(align, align).unwrap());
    }
}

#[test]
fn every_scalar_in_both_presets() {
    for target in [TargetLayout::wasm32(), TargetLayout::x86_64()] {
        for (ty, scalar, size) in [
            (Type::U8, Scalar::U8, 1),
            (Type::U16, Scalar::U16, 2),
            (Type::U32, Scalar::U32, 4),
            (Type::U64, Scalar::U64, 8),
            (Type::I8, Scalar::I8, 1),
            (Type::I16, Scalar::I16, 2),
            (Type::I32, Scalar::I32, 4),
            (Type::I64, Scalar::I64, 8),
            (Type::F32, Scalar::F32, 4),
            (Type::F64, Scalar::F64, 8),
        ] {
            assert_eq!(
                target.layout_of(&ty).unwrap(),
                Layout::new(size, size).unwrap()
            );
            assert_eq!(target.scalar_layout(scalar), target.layout_of(&ty).unwrap());
            assert_eq!(scalar.size(), size);
        }
    }
}

#[test]
fn nested_structures_and_arrays_use_complete_element_size() {
    let header = Type::Struct(vec![Type::U8, Type::U32, Type::U16]);
    let outer = Type::Struct(vec![Type::U8, Type::array(header.clone(), 3), Type::U16]);
    let target = TargetLayout::x86_64();
    assert_eq!(target.layout_of(&header).unwrap().size(), 12);
    let named = target
        .struct_layout([
            ("tag", Type::U8),
            ("headers", Type::array(header, 3)),
            ("flags", Type::U16),
        ])
        .unwrap();
    assert_eq!(named.layout(), target.layout_of(&outer).unwrap());
    assert_eq!((named.size(), named.align()), (44, 4));
    assert_eq!(named.offset("headers"), Some(4));
    assert_eq!(named.offset("flags"), Some(40));
    assert_eq!(
        target
            .layout_of(&Type::array(Type::array(Type::U16, 3), 5))
            .unwrap()
            .size(),
        30
    );
}

#[test]
fn custom_scalar_alignment_changes_struct_and_array_layout() {
    let target = TargetLayout::builder()
        .pointer_size(4)
        .pointer_align(4)
        .scalar_alignment(Scalar::U64, 4)
        .scalar_alignment(Scalar::I64, 4)
        .scalar_alignment(Scalar::F64, 4)
        .build()
        .unwrap();
    for ty in [Type::U64, Type::I64, Type::F64] {
        let record = Type::Struct(vec![Type::U8, ty]);
        assert_eq!(
            target.layout_of(&record).unwrap(),
            Layout::new(12, 4).unwrap()
        );
        assert_eq!(
            target.layout_of(&Type::array(record, 2)).unwrap().size(),
            24
        );
    }
    let separate = TargetLayout::builder()
        .pointer_size(8)
        .pointer_align(8)
        .scalar_alignment(Scalar::U64, 4)
        .build()
        .unwrap();
    assert_eq!(separate.layout_of(&Type::I64).unwrap().align(), 8);
}

#[test]
fn pointer_size_and_alignment_are_independent() {
    let target = TargetLayout::builder()
        .pointer_size(8)
        .pointer_align(4)
        .build()
        .unwrap();
    let structure = target
        .struct_layout([("tag", Type::U8), ("ptr", Type::Pointer)])
        .unwrap();
    assert_eq!(structure.offset("ptr"), Some(4));
    assert_eq!((structure.size(), structure.align()), (12, 4));
}

#[test]
fn target_builder_requires_explicit_valid_settings() {
    for builder in [
        TargetLayout::builder(),
        TargetLayout::builder().pointer_size(4),
        TargetLayout::builder().pointer_align(4),
    ] {
        assert_eq!(builder.build(), Err(LayoutError::MissingPointerLayout));
    }
    for align in [0, 3] {
        assert_eq!(
            TargetLayout::builder()
                .pointer_size(8)
                .pointer_align(align)
                .build(),
            Err(LayoutError::InvalidAlignment(align))
        );
    }
    for (size, align) in [(0, 1), (3, 2), (4, 8)] {
        assert_eq!(
            TargetLayout::builder()
                .pointer_size(size)
                .pointer_align(align)
                .build(),
            Err(LayoutError::InvalidTypeSize { size, align })
        );
    }
    assert_eq!(
        TargetLayout::builder()
            .pointer_size(4)
            .pointer_align(4)
            .scalar_alignment(Scalar::F64, 3)
            .build(),
        Err(LayoutError::InvalidAlignment(3))
    );
    assert_eq!(
        TargetLayout::builder()
            .pointer_size(4)
            .pointer_align(4)
            .scalar_alignment(Scalar::U8, 2)
            .build(),
        Err(LayoutError::InvalidTypeSize { size: 1, align: 2 })
    );
}

#[test]
fn empty_types_and_zero_length_arrays() {
    let target = TargetLayout::wasm32();
    assert_eq!(
        target.layout_of(&Type::Struct(vec![])).unwrap(),
        Layout::EMPTY
    );
    assert_eq!(
        target.layout_of(&Type::array(Type::U64, 0)).unwrap(),
        Layout::new(0, 8).unwrap()
    );
    assert_eq!(
        target
            .layout_of(&Type::array(Type::Struct(vec![]), u64::MAX))
            .unwrap(),
        Layout::EMPTY
    );
    let record = Type::Struct(vec![Type::U8, Type::array(Type::U64, 0)]);
    assert_eq!(
        target.layout_of(&record).unwrap(),
        Layout::new(8, 8).unwrap()
    );
}

#[test]
fn type_layout_overflow_propagates() {
    let target = TargetLayout::wasm32();
    let too_big = Type::array(Type::U64, u64::MAX);
    assert_eq!(target.layout_of(&too_big), Err(LayoutError::Overflow));
    assert_eq!(
        target.layout_of(&Type::array(too_big.clone(), 0)),
        Err(LayoutError::Overflow)
    );
    assert_eq!(
        target.layout_of(&Type::Struct(vec![too_big])),
        Err(LayoutError::Overflow)
    );
    let overflow_on_append = Type::Struct(vec![Type::array(Type::U8, u64::MAX), Type::U8]);
    assert_eq!(
        target.layout_of(&overflow_on_append),
        Err(LayoutError::Overflow)
    );
    let overflow_on_tail = Type::Struct(vec![Type::U64, Type::array(Type::U8, u64::MAX - 8)]);
    assert_eq!(
        target.layout_of(&overflow_on_tail),
        Err(LayoutError::Overflow)
    );
}

#[test]
fn target_address_space_does_not_limit_geometry() {
    let layout = TargetLayout::wasm32()
        .layout_of(&Type::array(Type::U8, 1 << 40))
        .unwrap();
    assert_eq!(layout.size(), 1 << 40);
}

#[test]
fn named_target_structs_reject_duplicate_fields() {
    assert_eq!(
        TargetLayout::x86_64().struct_layout([("x", Type::U8), ("x", Type::U32)]),
        Err(LayoutError::DuplicateField("x".into()))
    );
}
