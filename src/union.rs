// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use alloc::{string::String, vec::Vec};

use crate::{FieldLayout, Layout, LayoutError, structure::validate_names};

/// Overlapping named fields, all starting at offset zero.
///
/// The largest field determines the unpadded size. The final size is rounded to
/// the union's alignment, which is the largest effective field alignment or the
/// explicitly requested alignment, whichever is greater. No discriminant is
/// inserted. An empty union has size zero and alignment one by default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnionLayout {
    layout: Layout,
    fields: Vec<FieldLayout>,
    tail_padding: u64,
}

impl UnionLayout {
    /// Starts a union builder.
    pub fn builder() -> UnionLayoutBuilder {
        UnionLayoutBuilder::default()
    }

    /// Returns the complete layout for nesting or repetition.
    pub const fn layout(&self) -> Layout {
        self.layout
    }

    /// Returns size including trailing padding.
    pub const fn size(&self) -> u64 {
        self.layout.size()
    }

    /// Returns the union's alignment.
    pub const fn align(&self) -> u64 {
        self.layout.align()
    }

    /// Returns fields in declaration order; all offsets are zero.
    pub fn fields(&self) -> &[FieldLayout] {
        &self.fields
    }

    /// Looks up a field by name in linear time.
    pub fn field(&self, name: &str) -> Option<&FieldLayout> {
        self.fields.iter().find(|field| field.name() == name)
    }

    /// Returns `Some(0)` for a known field or `None` for an unknown field.
    pub fn offset(&self, name: &str) -> Option<u64> {
        self.field(name).map(FieldLayout::offset)
    }

    /// Returns padding after the largest field, not the unused space in smaller fields.
    pub const fn tail_padding(&self) -> u64 {
        self.tail_padding
    }
}

/// Builds a union, reporting validation and arithmetic errors at `build`.
#[derive(Clone, Debug)]
#[must_use]
pub struct UnionLayoutBuilder {
    fields: Vec<(String, Layout)>,
    pack: Option<u64>,
    align: u64,
}

impl Default for UnionLayoutBuilder {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            pack: None,
            align: 1,
        }
    }
}

impl UnionLayoutBuilder {
    /// Adds an overlapping field. Names must be unique in this union.
    pub fn field(mut self, name: impl Into<String>, layout: Layout) -> Self {
        self.fields.push((name.into(), layout));
        self
    }

    /// Caps field alignment at a nonzero power of two; the last call wins.
    ///
    /// A value of one makes the union byte-packed. This does not remove internal
    /// padding from nested field layouts.
    pub fn packed(mut self, max_align: u64) -> Self {
        self.pack = Some(max_align);
        self
    }

    /// Sets a minimum union alignment independently of packing; the last call wins.
    /// Must be a nonzero power of two.
    pub fn align(mut self, align: u64) -> Self {
        self.align = align;
        self
    }

    /// Calculates the union layout and trailing padding.
    ///
    /// Returns an error for invalid alignment, duplicate names, or overflow.
    pub fn build(self) -> Result<UnionLayout, LayoutError> {
        Layout::new(0, self.align)?;
        if let Some(pack) = self.pack {
            Layout::new(0, pack)?;
        }
        validate_names(&self.fields)?;
        let mut size = 0;
        let mut align = self.align;
        let mut fields = Vec::with_capacity(self.fields.len());
        for (name, layout) in self.fields {
            let field_align = self
                .pack
                .map_or(layout.align(), |pack| layout.align().min(pack));
            size = size.max(layout.size());
            align = align.max(field_align);
            fields.push(FieldLayout {
                name,
                layout,
                offset: 0,
                align: field_align,
                padding_before: 0,
            });
        }
        let layout = Layout::new(size, align)?.pad_to_align()?;
        Ok(UnionLayout {
            layout,
            fields,
            tail_padding: layout.size() - size,
        })
    }
}
