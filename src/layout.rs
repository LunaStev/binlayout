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

    /// Computes this layout's placement after an absolute byte `cursor`.
    ///
    /// Returns `(leading_padding, start, end)`. `cursor`, `start`, and `end`
    /// are byte positions measured from the beginning of the containing
    /// buffer; unlike the field offset returned by [`Self::extend`], they are
    /// not relative to this layout's origin. `end` is exclusive. A zero-sized
    /// layout still requires its alignment, so placement may advance the
    /// cursor even when `start == end`.
    ///
    /// Returns [`LayoutError::Overflow`] if aligning the cursor or adding the
    /// layout size cannot fit in `u64`.
    pub fn place_at(self, cursor: u64) -> Result<(u64, u64, u64), LayoutError> {
        let leading_padding = cursor.wrapping_neg() & (self.align - 1);
        let start = cursor
            .checked_add(leading_padding)
            .ok_or(LayoutError::Overflow)?;
        let end = start.checked_add(self.size).ok_or(LayoutError::Overflow)?;
        Ok((leading_padding, start, end))
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
    ///
    /// An unpadded size-3 element with alignment 2 keeps its extent until it is
    /// repeated:
    ///
    /// ```
    /// use binlayout::Layout;
    ///
    /// let element = Layout::new(3, 2)?;
    /// assert_eq!(element.size(), 3);
    ///
    /// let (array, stride) = element.repeat(3)?;
    /// assert_eq!(stride, 4);
    /// assert_eq!(array.size(), 12);
    /// # Ok::<(), binlayout::LayoutError>(())
    /// ```
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
