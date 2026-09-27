// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Union layout, native representation, nesting, and overflow checks.

use binlayout::{Layout, LayoutError, StructLayout, TargetLayout, Type, UnionLayout};

fn payload_builder() -> binlayout::UnionLayoutBuilder {
    UnionLayout::builder()
        .field("bytes", Layout::new(9, 1).unwrap())
        .field("number", Layout::new(8, 8).unwrap())
}

#[test]
fn union_rounds_largest_size_to_largest_alignment() {
    let payload = payload_builder().build().unwrap();
    assert_eq!(
        (payload.size(), payload.align(), payload.tail_padding()),
        (16, 8, 7)
    );
    assert_eq!(payload.fields().len(), 2);
    assert_eq!(payload.fields()[0].name(), "bytes");
    assert_eq!(payload.offset("bytes"), Some(0));
    assert_eq!(payload.offset("number"), Some(0));
    assert_eq!(payload.offset("unknown"), None);
    assert_eq!(payload.field("unknown"), None);
    for field in payload.fields() {
        assert_eq!(field.padding_before(), 0);
        assert_eq!(field.layout().align(), field.align());
    }
}

#[test]
fn packing_caps_alignment_but_retains_original_layouts() {
    for (pack, size, align) in [(1, 9, 1), (2, 10, 2), (4, 12, 4), (16, 16, 8)] {
        let payload = payload_builder().packed(pack).build().unwrap();
        assert_eq!((payload.size(), payload.align()), (size, align));
        assert_eq!(payload.field("number").unwrap().layout().align(), 8);
        assert_eq!(payload.field("number").unwrap().align(), align);
    }
    let aligned = payload_builder().packed(1).align(32).build().unwrap();
    assert_eq!(
        (aligned.size(), aligned.align(), aligned.tail_padding()),
        (32, 32, 23)
    );
}

#[test]
fn empty_and_zero_sized_unions() {
    let empty = UnionLayout::builder().build().unwrap();
    assert_eq!(empty.layout(), Layout::EMPTY);
    assert_eq!(empty.tail_padding(), 0);
    assert!(empty.fields().is_empty());
    assert_eq!(
        UnionLayout::builder().align(16).build().unwrap().layout(),
        Layout::new(0, 16).unwrap()
    );
    let zst = UnionLayout::builder()
        .field("zero", Layout::new(0, 8).unwrap())
        .build()
        .unwrap();
    assert_eq!(zst.layout(), Layout::new(0, 8).unwrap());
    assert_eq!(zst.offset("zero"), Some(0));
}

#[test]
fn union_errors_and_packing_at_the_u64_boundary() {
    assert_eq!(
        UnionLayout::builder().packed(0).build(),
        Err(LayoutError::InvalidAlignment(0))
    );
    assert_eq!(
        UnionLayout::builder().align(3).build(),
        Err(LayoutError::InvalidAlignment(3))
    );
    assert_eq!(
        UnionLayout::builder()
            .field("x", Layout::EMPTY)
            .field("x", Layout::EMPTY)
            .build(),
        Err(LayoutError::DuplicateField("x".into()))
    );
    let huge = UnionLayout::builder()
        .field("bytes", Layout::new(u64::MAX, 1).unwrap())
        .field("aligned", Layout::new(8, 8).unwrap());
    assert_eq!(huge.clone().build(), Err(LayoutError::Overflow));
    assert_eq!(huge.packed(1).build().unwrap().size(), u64::MAX);
}

#[test]
fn union_nests_in_structs_and_repeats_at_its_padded_stride() {
    let payload = payload_builder().build().unwrap();
    let packet = StructLayout::builder()
        .field("tag", Layout::new(1, 1).unwrap())
        .field("payload", payload.layout())
        .build()
        .unwrap();
    assert_eq!(packet.offset("payload"), Some(8));
    assert_eq!(packet.size(), 24);
    assert_eq!(
        payload.layout().repeat(3).unwrap(),
        (Layout::new(48, 8).unwrap(), 16)
    );
}

#[test]
fn target_types_match_named_unions_including_pointer_rules() {
    for target in [TargetLayout::wasm32(), TargetLayout::x86_64()] {
        let ty = Type::Union(vec![Type::array(Type::U8, 9), Type::Pointer]);
        let named = target
            .union_layout([("bytes", Type::array(Type::U8, 9)), ("ptr", Type::Pointer)])
            .unwrap();
        assert_eq!(named.layout(), target.layout_of(&ty).unwrap());
        let expected = if target.pointer_layout().size() == 4 {
            12
        } else {
            16
        };
        assert_eq!(named.size(), expected);
        assert_eq!(named.offset("ptr"), Some(0));
        assert_eq!(
            target
                .layout_of(&Type::array(ty.clone(), 3))
                .unwrap()
                .size(),
            expected * 3
        );
        let wrapper = Type::Struct(vec![Type::U8, ty]);
        assert_eq!(
            target.layout_of(&wrapper).unwrap().size(),
            expected + target.pointer_layout().size()
        );
    }
}

#[test]
fn target_union_empty_nested_and_error_cases() {
    let target = TargetLayout::x86_64();
    assert_eq!(
        target.layout_of(&Type::Union(vec![])).unwrap(),
        Layout::EMPTY
    );
    let nested = Type::Union(vec![Type::Union(vec![Type::U8, Type::U32]), Type::U64]);
    assert_eq!(
        target.layout_of(&nested).unwrap(),
        Layout::new(8, 8).unwrap()
    );
    let huge = Type::Union(vec![Type::array(Type::U8, u64::MAX), Type::U64]);
    assert_eq!(target.layout_of(&huge), Err(LayoutError::Overflow));
    let inner_overflow = Type::Union(vec![Type::array(Type::U64, u64::MAX)]);
    assert_eq!(
        target.layout_of(&inner_overflow),
        Err(LayoutError::Overflow)
    );
    assert_eq!(
        target.union_layout([("x", Type::U8), ("x", Type::U32)]),
        Err(LayoutError::DuplicateField("x".into()))
    );
    assert_eq!(
        target.union_layout([("huge", Type::array(Type::U64, u64::MAX))]),
        Err(LayoutError::Overflow)
    );
}

#[test]
fn agrees_with_native_repr_c_union_and_packed_union() {
    #[repr(C)]
    union Payload {
        bytes: [u8; 9],
        number: u64,
    }
    #[repr(C, packed(2))]
    union PackedPayload {
        bytes: [u8; 9],
        number: u64,
    }
    let native = UnionLayout::builder()
        .field(
            "bytes",
            Layout::new(size_of::<[u8; 9]>() as u64, align_of::<[u8; 9]>() as u64).unwrap(),
        )
        .field(
            "number",
            Layout::new(size_of::<u64>() as u64, align_of::<u64>() as u64).unwrap(),
        );
    let plain = native.clone().build().unwrap();
    assert_eq!(
        plain.layout(),
        Layout::new(size_of::<Payload>() as u64, align_of::<Payload>() as u64).unwrap()
    );
    assert_eq!(
        plain.offset("number"),
        Some(core::mem::offset_of!(Payload, number) as u64)
    );
    let packed = native.packed(2).build().unwrap();
    assert_eq!(
        packed.layout(),
        Layout::new(
            size_of::<PackedPayload>() as u64,
            align_of::<PackedPayload>() as u64
        )
        .unwrap()
    );
}
