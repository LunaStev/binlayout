// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Arithmetic boundaries and independent layout geometry references.

use binlayout::{Layout, LayoutError};

#[test]
fn alignment_validation_and_zero_size() {
    for align in [0, 3, 6, u64::MAX] {
        assert_eq!(
            Layout::new(1, align),
            Err(LayoutError::InvalidAlignment(align))
        );
    }
    for shift in 0..64 {
        assert_eq!(Layout::new(0, 1 << shift).unwrap().align(), 1 << shift);
    }
    assert_eq!(Layout::default(), Layout::EMPTY);
}

#[test]
fn extend_defers_tail_padding() {
    let (layout, offset) = Layout::new(1, 1)
        .unwrap()
        .extend(Layout::new(4, 4).unwrap())
        .unwrap();
    assert_eq!((layout.size(), layout.align(), offset), (8, 4, 4));
    let (layout, offset) = layout.extend(Layout::new(2, 2).unwrap()).unwrap();
    assert_eq!((layout.size(), layout.align(), offset), (10, 4, 8));
    assert_eq!(layout.pad_to_align().unwrap().size(), 12);
}

#[test]
fn explicit_alignment_does_not_change_size() {
    let layout = Layout::new(3, 2).unwrap().align_to(16).unwrap();
    assert_eq!((layout.size(), layout.align()), (3, 16));
    assert_eq!(layout.align_to(1).unwrap(), layout);
    assert_eq!(layout.pad_to_align().unwrap().size(), 16);
    assert_eq!(layout.align_to(0), Err(LayoutError::InvalidAlignment(0)));
    assert_eq!(
        layout.padding_needed_for(3),
        Err(LayoutError::InvalidAlignment(3))
    );
}

#[test]
fn arrays_use_padded_stride() {
    let element = Layout::new(3, 2).unwrap();
    let (array, stride) = element.repeat(3).unwrap();
    assert_eq!((array.size(), array.align(), stride), (12, 2, 4));
    let (empty, stride) = element.repeat(0).unwrap();
    assert_eq!((empty.size(), empty.align(), stride), (0, 2, 4));
    let (zero_sized, stride) = Layout::new(0, 8).unwrap().repeat(u64::MAX).unwrap();
    assert_eq!((zero_sized.size(), zero_sized.align(), stride), (0, 8, 0));
}

#[test]
fn supports_sizes_beyond_host_allocation_limits() {
    let layout = Layout::new(u64::MAX, 1).unwrap();
    assert_eq!(layout.pad_to_align().unwrap(), layout);
    assert_eq!(layout.extend(Layout::EMPTY).unwrap(), (layout, u64::MAX));
    assert_eq!(layout.repeat(1).unwrap(), (layout, u64::MAX));
}

#[test]
fn checked_arithmetic_reports_every_overflow_path() {
    let max = Layout::new(u64::MAX, 1).unwrap();
    assert_eq!(
        max.extend(Layout::new(1, 1).unwrap()),
        Err(LayoutError::Overflow)
    );
    assert_eq!(
        max.extend(Layout::new(0, 2).unwrap()),
        Err(LayoutError::Overflow)
    );
    assert_eq!(max.repeat(2), Err(LayoutError::Overflow));
    let unpadded = Layout::new(u64::MAX, 2).unwrap();
    assert_eq!(unpadded.padding_needed_for(2), Ok(1));
    assert_eq!(unpadded.pad_to_align(), Err(LayoutError::Overflow));
    assert_eq!(unpadded.repeat(0), Err(LayoutError::Overflow));
    let largest_align = Layout::new(1, 1 << 63).unwrap();
    assert_eq!(largest_align.pad_to_align().unwrap().size(), 1 << 63);
    assert_eq!(largest_align.repeat(2), Err(LayoutError::Overflow));
}

#[test]
fn extension_agrees_with_u128_reference_at_boundaries() {
    let sizes = [0, 1, 3, 8, 255, 1 << 32, 1 << 63, u64::MAX - 1, u64::MAX];
    for size in sizes {
        for next_size in sizes {
            for shift in 0..64 {
                let align = 1u64 << shift;
                let base = Layout::new(size, 8).unwrap();
                let next = Layout::new(next_size, align).unwrap();
                // Wider integer arithmetic is an independent overflow oracle.
                let a = u128::from(align);
                let offset = u128::from(size).div_ceil(a) * a;
                let end = offset + u128::from(next_size);
                let result = base.extend(next);
                if end > u128::from(u64::MAX) {
                    assert_eq!(result, Err(LayoutError::Overflow));
                } else {
                    let (layout, actual_offset) = result.unwrap();
                    assert_eq!(u128::from(actual_offset), offset);
                    assert_eq!(u128::from(layout.size()), end);
                    assert_eq!(layout.align(), align.max(8));
                }
            }
        }
    }
}

#[test]
fn ordinary_layouts_agree_with_std_allocator_geometry() {
    for size in 0..16 {
        for align in [1, 2, 4, 8, 16] {
            for next_size in 0..16 {
                for next_align in [1, 2, 4, 8, 16] {
                    let a = Layout::new(size, align).unwrap();
                    let b = Layout::new(next_size, next_align).unwrap();
                    let (ours, offset) = a.extend(b).unwrap();
                    let native =
                        std::alloc::Layout::from_size_align(size as usize, align as usize).unwrap();
                    let next = std::alloc::Layout::from_size_align(
                        next_size as usize,
                        next_align as usize,
                    )
                    .unwrap();
                    let (reference, reference_offset) = native.extend(next).unwrap();
                    assert_eq!(offset, reference_offset as u64);
                    assert_eq!(ours.size(), reference.size() as u64);
                    assert_eq!(ours.align(), reference.align() as u64);
                    assert_eq!(
                        ours.pad_to_align().unwrap().size(),
                        reference.pad_to_align().size() as u64
                    );
                }
            }
        }
    }
}
