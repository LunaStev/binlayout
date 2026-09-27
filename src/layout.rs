// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::LayoutError;

/// Size and alignment in bytes, independent of the host's address space.
///
/// Alignment is a nonzero power of two. Size may be zero and need not be a
/// multiple of alignment: intermediate records can omit trailing padding.
/// Unlike an allocation layout, size is not limited to `isize::MAX`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Layout {
    size: u64,
    align: u64,
}

impl Layout {
    /// An empty layout with size zero and alignment one.
    pub const EMPTY: Self = Self { size: 0, align: 1 };

    /// Creates a layout, validating alignment without adding trailing padding.
    ///
    /// Returns [`LayoutError::InvalidAlignment`] for zero or non-power-of-two
    /// alignment. Rounding a valid size up may still overflow in later operations.
    pub const fn new(size: u64, align: u64) -> Result<Self, LayoutError> {
        if !align.is_power_of_two() {
            return Err(LayoutError::InvalidAlignment(align));
        }
        Ok(Self { size, align })
    }

    /// Returns the size in bytes, including any padding already applied.
    pub const fn size(self) -> u64 {
        self.size
    }

    /// Returns the required alignment in bytes.
    pub const fn align(self) -> u64 {
        self.align
    }

    /// Returns the bytes needed after this layout to reach `align` alignment.
    ///
    /// Returns an error for invalid alignment. The padding itself always fits;
    /// adding it to the size can still overflow.
    pub fn padding_needed_for(self, align: u64) -> Result<u64, LayoutError> {
        Self::new(0, align)?;
        Ok(self.size.wrapping_neg() & (align - 1))
    }

    /// Appends `next`, returning the combined layout and `next`'s offset.
    ///
    /// Inserts padding before `next`, but does not add trailing padding to the
    /// result. Call [`Self::pad_to_align`] when finishing a structure.
    /// Returns [`LayoutError::Overflow`] if the offset or size cannot fit.
    pub fn extend(self, next: Self) -> Result<(Self, u64), LayoutError> {
        let offset = self
            .size
            .checked_add(self.padding_needed_for(next.align)?)
            .ok_or(LayoutError::Overflow)?;
        let size = offset.checked_add(next.size).ok_or(LayoutError::Overflow)?;
        Ok((
            Self {
                size,
                align: self.align.max(next.align),
            },
            offset,
        ))
    }

    /// Raises alignment to at least `align`, without changing size.
    ///
    /// Returns an error if `align` is not a nonzero power of two.
    pub fn align_to(self, align: u64) -> Result<Self, LayoutError> {
        Self::new(self.size, self.align.max(Self::new(0, align)?.align))
    }

    /// Rounds size up to a multiple of this layout's alignment.
    ///
    /// Returns [`LayoutError::Overflow`] if the padded size cannot fit.
    pub fn pad_to_align(self) -> Result<Self, LayoutError> {
        let size = self
            .size
            .checked_add(self.padding_needed_for(self.align)?)
            .ok_or(LayoutError::Overflow)?;
        Ok(Self { size, ..self })
    }

    /// Repeats this layout, returning the array layout and element stride.
    ///
    /// Stride includes each element's trailing padding. Array size is
    /// `stride * count`, including the final element's padding. A zero count
    /// retains element alignment. Zero-sized elements have zero stride.
    /// Returns [`LayoutError::Overflow`] if stride or total size cannot fit,
    /// even when `count` is zero.
    pub fn repeat(self, count: u64) -> Result<(Self, u64), LayoutError> {
        let stride = self.pad_to_align()?.size;
        let size = stride.checked_mul(count).ok_or(LayoutError::Overflow)?;
        Ok((
            Self {
                size,
                align: self.align,
            },
            stride,
        ))
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::EMPTY
    }
}
