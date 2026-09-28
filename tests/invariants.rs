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

#[test]
fn padding_is_idempotent_across_boundary_cases() {
    let successful_cases = [
        (0, 1),
        (0, 1u64 << 63),
        (1, 1u64 << 63),
        (u64::MAX - 8, 8),
        (u64::MAX - 7, 1),
    ];
    for (size, align) in successful_cases {
        let layout = Layout::new(size, align).unwrap();
        let padded = layout.pad_to_align().unwrap();
        assert_eq!(
            padded.pad_to_align(),
            Ok(padded),
            "padding must be idempotent for size {size} and alignment {align}"
        );
    }

    let overflowing_cases = [(u64::MAX, 2), (u64::MAX - 1, 1u64 << 63)];
    for (size, align) in overflowing_cases {
        assert_eq!(
            Layout::new(size, align).unwrap().pad_to_align(),
            Err(LayoutError::Overflow),
            "padding size {size} to alignment {align} must report overflow"
        );
    }
}

#[test]
fn raising_alignment_twice_matches_the_maximum_and_preserves_size() {
    let alignments: [u64; 6] = [1, 2, 4, 8, 1 << 32, 1 << 63];
    let sizes = [0, 1, 3, u64::MAX - 1, u64::MAX];

    for size in sizes {
        for initial_align in alignments {
            let layout = Layout::new(size, initial_align).unwrap();
            for first in alignments {
                for second in alignments {
                    let raised_twice = layout.align_to(first).unwrap().align_to(second).unwrap();
                    let raised_to_max = layout.align_to(first.max(second)).unwrap();
                    assert_eq!(
                        raised_twice, raised_to_max,
                        "size {size}, initial alignment {initial_align}, requests {first} and {second}"
                    );
                    assert_eq!(raised_twice.size(), size);
                }
            }
        }
    }
}

#[test]
fn weaker_alignment_requests_leave_layout_unchanged() {
    let alignments: [u64; 6] = [1, 2, 4, 8, 1 << 32, 1 << 63];
    let sizes = [0, 1, 3, u64::MAX - 1, u64::MAX];

    for size in sizes {
        for current_align in alignments {
            let layout = Layout::new(size, current_align).unwrap();
            for requested_align in alignments {
                if requested_align <= current_align {
                    assert_eq!(
                        layout.align_to(requested_align),
                        Ok(layout),
                        "size {size}, current alignment {current_align}, weaker request {requested_align}"
                    );
                }
            }
        }
    }
}
