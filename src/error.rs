// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use alloc::string::String;
use core::fmt;

/// A layout could not be constructed or represented in `u64` bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LayoutError {
    /// An alignment was zero or not a power of two.
    InvalidAlignment(u64),
    /// A size, offset, or array stride exceeded `u64::MAX`.
    Overflow,
    /// A structure or union contained a repeated field name.
    DuplicateField(String),
    /// The target builder requires an explicit pointer size and alignment.
    MissingPointerLayout,
    /// A target scalar or pointer must have nonzero size divisible by alignment.
    InvalidTypeSize {
        /// The requested size in bytes.
        size: u64,
        /// The requested alignment in bytes.
        align: u64,
    },
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAlignment(align) => {
                write!(f, "alignment must be a nonzero power of two, got {align}")
            }
            Self::Overflow => f.write_str("layout calculation exceeds u64::MAX bytes"),
            Self::DuplicateField(name) => write!(f, "duplicate field name: {name}"),
            Self::MissingPointerLayout => {
                f.write_str("pointer size and alignment must be specified")
            }
            Self::InvalidTypeSize { size, align } => {
                write!(
                    f,
                    "type size {size} must be nonzero and divisible by alignment {align}"
                )
            }
        }
    }
}

impl core::error::Error for LayoutError {}
