// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Structure placement, packing, nesting, and native representation checks.

use binlayout::{Layout, LayoutError, StructLayout};

fn header_builder() -> binlayout::StructLayoutBuilder {
    StructLayout::builder()
        .field("tag", Layout::new(1, 1).unwrap())
        .field("length", Layout::new(4, 4).unwrap())
        .field("flags", Layout::new(2, 2).unwrap())
}

#[test]
fn header_offsets_padding_and_lookup() {
    let header = header_builder().build().unwrap();
    assert_eq!((header.size(), header.align()), (12, 4));
    assert_eq!(header.offset("tag"), Some(0));
    assert_eq!(header.offset("length"), Some(4));
    assert_eq!(header.offset("flags"), Some(8));
    assert_eq!(header.offset("missing"), None);
    assert_eq!(header.field("missing"), None);
    assert_eq!(
        header
            .fields()
            .iter()
            .map(|f| f.padding_before())
            .collect::<Vec<_>>(),
        [0, 3, 0]
    );
    assert_eq!(header.fields()[1].name(), "length");
    assert_eq!(header.tail_padding(), 2);
}

#[test]
fn byte_packed_header() {
    let header = header_builder().packed(1).build().unwrap();
    assert_eq!((header.size(), header.align()), (7, 1));
    assert_eq!(header.offset("length"), Some(1));
    assert_eq!(header.offset("flags"), Some(5));
    assert_eq!(header.tail_padding(), 0);
    let length = header.field("length").unwrap();
    assert_eq!(length.align(), 1);
    assert_eq!(length.layout().align(), 4);
}

#[test]
fn packing_cap_and_explicit_alignment() {
    let header = header_builder().packed(2).build().unwrap();
    assert_eq!((header.size(), header.align()), (8, 2));
    assert_eq!(header.offset("length"), Some(2));
    assert_eq!(header.offset("flags"), Some(6));
    let over_aligned = header_builder().packed(1).align(16).build().unwrap();
    assert_eq!((over_aligned.size(), over_aligned.align()), (16, 16));
    assert_eq!(over_aligned.offset("length"), Some(1));
    assert_eq!(over_aligned.tail_padding(), 9);
    assert_eq!(header_builder().align(1).build().unwrap().align(), 4);
    assert_eq!(
        header_builder().packed(16).build().unwrap(),
        header_builder().build().unwrap()
    );
}

#[test]
fn empty_and_zero_sized_fields_preserve_alignment() {
    let empty = StructLayout::builder().build().unwrap();
    assert_eq!(empty.layout(), Layout::EMPTY);
    assert!(empty.fields().is_empty());
    assert_eq!(empty.tail_padding(), 0);
    let aligned = StructLayout::builder().align(32).build().unwrap();
    assert_eq!((aligned.size(), aligned.align()), (0, 32));
    let zst = StructLayout::builder()
        .field("byte", Layout::new(1, 1).unwrap())
        .field("zero", Layout::new(0, 8).unwrap())
        .field("also_zero", Layout::new(0, 8).unwrap())
        .build()
        .unwrap();
    assert_eq!((zst.size(), zst.align()), (8, 8));
    assert_eq!(zst.offset("zero"), Some(8));
    assert_eq!(zst.offset("also_zero"), Some(8));
    assert_eq!(zst.fields()[1].padding_before(), 7);
}

#[test]
fn nesting_retains_internal_padding_even_when_outer_is_packed() {
    let inner = header_builder().build().unwrap();
    let outer = StructLayout::builder()
        .packed(1)
        .field("tag", Layout::new(1, 1).unwrap())
        .field("inner", inner.layout())
        .build()
        .unwrap();
    assert_eq!(outer.offset("inner"), Some(1));
    assert_eq!((outer.size(), outer.align()), (13, 1));
    assert_eq!(outer.fields()[1].layout().size(), 12);
}

#[test]
fn builder_validation() {
    assert_eq!(
        StructLayout::builder().packed(0).build(),
        Err(LayoutError::InvalidAlignment(0))
    );
    assert_eq!(
        StructLayout::builder().align(3).build(),
        Err(LayoutError::InvalidAlignment(3))
    );
    assert_eq!(
        StructLayout::builder()
            .field("x", Layout::EMPTY)
            .field("x", Layout::EMPTY)
            .build(),
        Err(LayoutError::DuplicateField("x".into()))
    );
    let overflow = StructLayout::builder()
        .field("huge", Layout::new(u64::MAX, 2).unwrap())
        .build();
    assert_eq!(overflow, Err(LayoutError::Overflow));
    let overflow = StructLayout::builder()
        .field("huge", Layout::new(u64::MAX, 1).unwrap())
        .field("byte", Layout::new(1, 1).unwrap())
        .build();
    assert_eq!(overflow, Err(LayoutError::Overflow));
}

#[test]
fn agrees_with_native_repr_c_using_explicit_host_field_layouts() {
    #[repr(C)]
    struct Header {
        tag: u8,
        length: u32,
        flags: u16,
    }
    fn native<T>() -> Layout {
        Layout::new(size_of::<T>() as u64, align_of::<T>() as u64).unwrap()
    }
    let header = StructLayout::builder()
        .field("tag", native::<u8>())
        .field("length", native::<u32>())
        .field("flags", native::<u16>())
        .build()
        .unwrap();
    assert_eq!(header.size(), size_of::<Header>() as u64);
    assert_eq!(header.align(), align_of::<Header>() as u64);
    assert_eq!(
        header.offset("tag"),
        Some(core::mem::offset_of!(Header, tag) as u64)
    );
    assert_eq!(
        header.offset("length"),
        Some(core::mem::offset_of!(Header, length) as u64)
    );
    assert_eq!(
        header.offset("flags"),
        Some(core::mem::offset_of!(Header, flags) as u64)
    );
}

#[test]
fn agrees_with_native_packed_structure() {
    #[repr(C, packed(2))]
    struct Header {
        tag: u8,
        length: u32,
        flags: u16,
    }
    let header = header_builder().packed(2).build().unwrap();
    assert_eq!(header.size(), size_of::<Header>() as u64);
    assert_eq!(header.align(), align_of::<Header>() as u64);
    assert_eq!(
        header.offset("length"),
        Some(core::mem::offset_of!(Header, length) as u64)
    );
    assert_eq!(
        header.offset("flags"),
        Some(core::mem::offset_of!(Header, flags) as u64)
    );
}
