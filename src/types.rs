// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use alloc::{boxed::Box, vec::Vec};

/// A fixed-width scalar whose alignment can be customized for a target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scalar {
    /// Unsigned 8-bit integer.
    U8,
    /// Unsigned 16-bit integer.
    U16,
    /// Unsigned 32-bit integer.
    U32,
    /// Unsigned 64-bit integer.
    U64,
    /// Signed 8-bit integer.
    I8,
    /// Signed 16-bit integer.
    I16,
    /// Signed 32-bit integer.
    I32,
    /// Signed 64-bit integer.
    I64,
    /// 32-bit floating-point value.
    F32,
    /// 64-bit floating-point value.
    F64,
}

impl Scalar {
    pub(crate) const ALL: [Self; 10] = [
        Self::U8,
        Self::U16,
        Self::U32,
        Self::U64,
        Self::I8,
        Self::I16,
        Self::I32,
        Self::I64,
        Self::F32,
        Self::F64,
    ];

    /// Returns the fixed size in bytes, regardless of target alignment.
    pub const fn size(self) -> u64 {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::U64 | Self::I64 | Self::F64 => 8,
        }
    }

    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}

/// A target-independent type description; no binary data is stored or decoded.
///
/// Structures preserve declaration order and include trailing padding. Arrays
/// include the stride of every element. Empty structures and zero-length arrays
/// are supported as mathematical layouts, without claiming portable C semantics.
/// Unions overlap their members and do not insert a discriminant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    /// Unsigned 8-bit integer.
    U8,
    /// Unsigned 16-bit integer.
    U16,
    /// Unsigned 32-bit integer.
    U32,
    /// Unsigned 64-bit integer.
    U64,
    /// Signed 8-bit integer.
    I8,
    /// Signed 16-bit integer.
    I16,
    /// Signed 32-bit integer.
    I32,
    /// Signed 64-bit integer.
    I64,
    /// 32-bit floating-point value.
    F32,
    /// 64-bit floating-point value.
    F64,
    /// A pointer using the target's explicit size and alignment.
    Pointer,
    /// An element type and element count.
    Array(Box<Type>, u64),
    /// Ordered field types; names do not influence placement.
    Struct(Vec<Type>),
    /// Overlapping member types, all starting at offset zero.
    Union(Vec<Type>),
}

impl Type {
    /// Describes an array without manually boxing its element type.
    pub fn array(element: Self, count: u64) -> Self {
        Self::Array(Box::new(element), count)
    }

    pub(crate) fn scalar(&self) -> Option<Scalar> {
        Some(match self {
            Self::U8 => Scalar::U8,
            Self::U16 => Scalar::U16,
            Self::U32 => Scalar::U32,
            Self::U64 => Scalar::U64,
            Self::I8 => Scalar::I8,
            Self::I16 => Scalar::I16,
            Self::I32 => Scalar::I32,
            Self::I64 => Scalar::I64,
            Self::F32 => Scalar::F32,
            Self::F64 => Scalar::F64,
            Self::Pointer | Self::Array(..) | Self::Struct(..) | Self::Union(..) => return None,
        })
    }
}
