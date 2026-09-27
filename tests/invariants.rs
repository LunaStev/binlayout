// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Deterministically generated cases for aggregate and array invariants.

use binlayout::{Layout, LayoutError, StructLayout, UnionLayout};

#[test]
fn generated_structs_and_unions_obey_placement_invariants() {
    // Include zero-sized and deliberately unpadded fields. Enumerate all
    // three-field combinations and multiple independent packing/alignment rules.
    let candidates = [(0, 1), (0, 8), (1, 1), (3, 2), (4, 4), (9, 8)];
    for a in candidates {
        for b in candidates {
            for c in candidates {
                for pack in [None, Some(1), Some(2), Some(8)] {
                    for requested_align in [1, 4, 16] {
                        let mut structure = StructLayout::builder().align(requested_align);
                        let mut union = UnionLayout::builder().align(requested_align);
                        if let Some(pack) = pack {
                            structure = structure.packed(pack);
                            union = union.packed(pack);
                        }
                        for (name, (size, align)) in ["a", "b", "c"].into_iter().zip([a, b, c]) {
                            let layout = Layout::new(size, align).unwrap();
                            structure = structure.field(name, layout);
                            union = union.field(name, layout);
                        }
                        let structure = structure.build().unwrap();
                        let union = union.build().unwrap();
                        let mut end = 0;
                        let mut padding = structure.tail_padding();
                        let mut required_align = requested_align;
                        let mut payload_bytes = 0;
                        for field in structure.fields() {
                            let expected_align = pack.map_or(field.layout().align(), |cap| {
                                cap.min(field.layout().align())
                            });
                            assert_eq!(field.align(), expected_align);
                            assert_eq!(field.offset() % field.align(), 0);
                            assert!(field.offset() >= end);
                            assert_eq!(field.offset() - end, field.padding_before());
                            // Padding must be minimal, not just sufficient.
                            assert!(field.padding_before() < field.align());
                            end = field.offset() + field.layout().size();
                            required_align = required_align.max(field.align());
                            payload_bytes += field.layout().size();
                            padding += field.padding_before();
                        }
                        assert_eq!(structure.align(), required_align);
                        assert_eq!(structure.size() % structure.align(), 0);
                        assert_eq!(structure.size(), payload_bytes + padding);
                        assert_eq!(structure.size() - end, structure.tail_padding());
                        assert!(structure.tail_padding() < structure.align());
                        assert_eq!(union.align(), required_align);
                        assert_eq!(union.size() % union.align(), 0);
                        assert_eq!(
                            union.size() - [a.0, b.0, c.0].into_iter().max().unwrap(),
                            union.tail_padding()
                        );
                        assert!(union.tail_padding() < union.align());
                        for field in union.fields() {
                            assert_eq!(field.offset(), 0);
                            assert!(field.layout().size() <= union.size());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn array_rounding_and_multiplication_match_a_wider_integer_oracle() {
    for size in [0, 1, 3, 8, 1 << 32, 1 << 63, u64::MAX - 1, u64::MAX] {
        for shift in 0..64 {
            for count in [0, 1, 2, 17, 1 << 32, u64::MAX] {
                let align = 1u64 << shift;
                let a = u128::from(align);
                let stride = u128::from(size).div_ceil(a) * a;
                // The widest stride is 2^64, so this product still fits u128.
                let bytes = stride * u128::from(count);
                let actual = Layout::new(size, align).unwrap().repeat(count);
                if stride > u128::from(u64::MAX) || bytes > u128::from(u64::MAX) {
                    assert_eq!(actual, Err(LayoutError::Overflow));
                } else {
                    let (layout, actual_stride) = actual.unwrap();
                    assert_eq!(u128::from(actual_stride), stride);
                    assert_eq!(u128::from(layout.size()), bytes);
                    assert_eq!(layout.align(), align);
                }
            }
        }
    }
}
