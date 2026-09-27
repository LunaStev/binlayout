// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use alloc::string::String;

use crate::{Layout, LayoutError, Scalar, StructLayout, Type, UnionLayout};

/// Explicit scalar and pointer rules for an external data model.
///
/// Presets cover the supported fixed-width scalar types, ordinary pointers,
/// arrays as members, declaration-order structures, and unions. They do not model a
/// complete ABI (calling conventions, bit-fields, vectors, or placement-specific
/// over-alignment of standalone arrays). Host layout is never consulted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetLayout {
    pointer: Layout,
    scalar_alignments: [u64; 10],
}

impl TargetLayout {
    /// Starts a builder with natural scalar alignments and no pointer defaults.
    pub fn builder() -> TargetLayoutBuilder {
        TargetLayoutBuilder::default()
    }

    /// The supported subset of the WebAssembly Basic C ABI for wasm32.
    ///
    /// Pointers have size/alignment four; scalar alignment equals scalar size.
    pub fn wasm32() -> Self {
        Self::builder()
            .pointer_size(4)
            .pointer_align(4)
            .build()
            .expect("valid wasm32 preset")
    }

    /// The supported subset of the x86-64 System V LP64 data model.
    ///
    /// Pointers have size/alignment eight; scalar alignment equals scalar size.
    /// This is not the x32/ILP32 ABI.
    pub fn x86_64() -> Self {
        Self::builder()
            .pointer_size(8)
            .pointer_align(8)
            .build()
            .expect("valid x86-64 preset")
    }

    /// Returns this target's pointer layout.
    pub const fn pointer_layout(&self) -> Layout {
        self.pointer
    }

    /// Returns this target's layout for a fixed-width scalar.
    pub fn scalar_layout(&self, scalar: Scalar) -> Layout {
        Layout::new(scalar.size(), self.scalar_alignments[scalar.index()])
            .expect("target builder validated scalar alignment")
    }

    /// Calculates size and alignment, recursively, using only this target.
    ///
    /// Returns [`LayoutError::Overflow`] if any intermediate layout cannot fit
    /// in `u64`. A zero-length array still validates its element layout.
    /// This computes byte geometry, not whether the result fits target memory.
    pub fn layout_of(&self, ty: &Type) -> Result<Layout, LayoutError> {
        if let Some(scalar) = ty.scalar() {
            return Ok(self.scalar_layout(scalar));
        }
        match ty {
            Type::Pointer => Ok(self.pointer),
            Type::Array(element, count) => Ok(self.layout_of(element)?.repeat(*count)?.0),
            Type::Struct(fields) => {
                let mut layout = Layout::EMPTY;
                for field in fields {
                    layout = layout.extend(self.layout_of(field)?)?.0;
                }
                layout.pad_to_align()
            }
            Type::Union(fields) => {
                let mut size = 0;
                let mut align = 1;
                for field in fields {
                    let layout = self.layout_of(field)?;
                    size = size.max(layout.size());
                    align = align.max(layout.align());
                }
                Layout::new(size, align)?.pad_to_align()
            }
            _ => unreachable!("scalar types were handled above"),
        }
    }

    /// Calculates a named structure from target-independent field types.
    ///
    /// Use [`StructLayout::builder`] with [`Self::layout_of`] for packing or
    /// explicit aggregate alignment. Returns errors for overflow or duplicate names.
    pub fn struct_layout<I, N>(&self, fields: I) -> Result<StructLayout, LayoutError>
    where
        I: IntoIterator<Item = (N, Type)>,
        N: Into<String>,
    {
        let mut builder = StructLayout::builder();
        for (name, ty) in fields {
            builder = builder.field(name, self.layout_of(&ty)?);
        }
        builder.build()
    }

    /// Calculates a named union from target-independent member types.
    ///
    /// All member offsets are zero. No discriminant is inserted. Returns errors
    /// for overflow or duplicate names. For packing or explicit alignment, pass
    /// [`Self::layout_of`] results to [`UnionLayout::builder`].
    pub fn union_layout<I, N>(&self, fields: I) -> Result<UnionLayout, LayoutError>
    where
        I: IntoIterator<Item = (N, Type)>,
        N: Into<String>,
    {
        let mut builder = UnionLayout::builder();
        for (name, ty) in fields {
            builder = builder.field(name, self.layout_of(&ty)?);
        }
        builder.build()
    }
}

/// Configures a target without inspecting the host.
///
/// Scalar alignments default to their byte sizes. Both pointer settings are
/// required. Scalar and pointer sizes must be nonzero multiples of alignment.
#[derive(Clone, Debug)]
#[must_use]
pub struct TargetLayoutBuilder {
    pointer_size: Option<u64>,
    pointer_align: Option<u64>,
    scalar_alignments: [u64; 10],
}

impl Default for TargetLayoutBuilder {
    fn default() -> Self {
        Self {
            pointer_size: None,
            pointer_align: None,
            scalar_alignments: [1, 2, 4, 8, 1, 2, 4, 8, 4, 8],
        }
    }
}

impl TargetLayoutBuilder {
    /// Sets the pointer size in bytes.
    pub fn pointer_size(mut self, size: u64) -> Self {
        self.pointer_size = Some(size);
        self
    }

    /// Sets the pointer alignment in bytes.
    pub fn pointer_align(mut self, align: u64) -> Self {
        self.pointer_align = Some(align);
        self
    }

    /// Overrides one scalar's alignment; signed and unsigned types are separate.
    ///
    /// Alignment must be a nonzero power of two dividing the scalar's fixed size.
    /// The last override for a scalar wins.
    pub fn scalar_alignment(mut self, scalar: Scalar, align: u64) -> Self {
        self.scalar_alignments[scalar.index()] = align;
        self
    }

    /// Validates pointer configuration and every scalar alignment.
    pub fn build(self) -> Result<TargetLayout, LayoutError> {
        let size = self.pointer_size.ok_or(LayoutError::MissingPointerLayout)?;
        let align = self
            .pointer_align
            .ok_or(LayoutError::MissingPointerLayout)?;
        let pointer = validate_type(size, align)?;
        for scalar in Scalar::ALL {
            validate_type(scalar.size(), self.scalar_alignments[scalar.index()])?;
        }
        Ok(TargetLayout {
            pointer,
            scalar_alignments: self.scalar_alignments,
        })
    }
}

fn validate_type(size: u64, align: u64) -> Result<Layout, LayoutError> {
    let layout = Layout::new(size, align)?;
    if size == 0 || size % align != 0 {
        return Err(LayoutError::InvalidTypeSize { size, align });
    }
    Ok(layout)
}
