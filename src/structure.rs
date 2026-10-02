// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use alloc::{collections::BTreeSet, string::String, vec::Vec};

use crate::{Layout, LayoutError};

/// One named field's placement within a structure or union.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldLayout {
    pub(crate) name: String,
    pub(crate) layout: Layout,
    pub(crate) offset: u64,
    pub(crate) align: u64,
    pub(crate) padding_before: u64,
}

impl FieldLayout {
    /// Returns the field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the original layout, before any outer packing constraint.
    pub const fn layout(&self) -> Layout {
        self.layout
    }

    /// Returns the field's byte offset from its immediate aggregate's start.
    /// Union fields always have offset zero.
    pub const fn offset(&self) -> u64 {
        self.offset
    }

    /// Returns the field's occupied half-open byte range in its immediate aggregate.
    ///
    /// The range covers the original field extent, including padding already
    /// inside that field, but excludes any aggregate padding before it. Union
    /// fields start at zero. A zero-sized field returns an empty range.
    pub fn range(&self) -> core::ops::Range<u64> {
        self.offset..self.offset + self.layout.size()
    }

    /// Returns the alignment used for placement, after applying packing.
    pub const fn align(&self) -> u64 {
        self.align
    }

    /// Returns padding between the previous structure field's end and this field.
    /// Union fields always report zero.
    pub const fn padding_before(&self) -> u64 {
        self.padding_before
    }
}

/// A declaration-order structure with field offsets and trailing padding.
///
/// An empty structure has size zero and alignment one unless explicitly aligned.
/// This is a layout convention, not a claim about empty structures in C or C++.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructLayout {
    layout: Layout,
    fields: Vec<FieldLayout>,
    tail_padding: u64,
}

impl StructLayout {
    /// Starts a structure builder.
    pub fn builder() -> StructLayoutBuilder {
        StructLayoutBuilder::default()
    }

    /// Returns the complete layout, suitable for nesting or repetition.
    pub const fn layout(&self) -> Layout {
        self.layout
    }

    /// Returns the size including trailing padding.
    pub const fn size(&self) -> u64 {
        self.layout.size()
    }

    /// Returns the structure's alignment.
    pub const fn align(&self) -> u64 {
        self.layout.align()
    }

    /// Returns all fields in declaration order.
    pub fn fields(&self) -> &[FieldLayout] {
        &self.fields
    }

    /// Looks up a field by name. Lookup is linear in the number of fields.
    pub fn field(&self, name: &str) -> Option<&FieldLayout> {
        self.fields.iter().find(|field| field.name == name)
    }

    /// Returns a field's byte offset, or `None` for an unknown name.
    pub fn offset(&self, name: &str) -> Option<u64> {
        self.field(name).map(FieldLayout::offset)
    }

    /// Returns padding after the final field.
    pub const fn tail_padding(&self) -> u64 {
        self.tail_padding
    }
}

/// Builds a structure. Validation and arithmetic errors are returned by `build`.
#[derive(Clone, Debug)]
#[must_use]
pub struct StructLayoutBuilder {
    fields: Vec<(String, Layout)>,
    pack: Option<u64>,
    align: u64,
}

impl Default for StructLayoutBuilder {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            pack: None,
            align: 1,
        }
    }
}

impl StructLayoutBuilder {
    /// Appends a field. Names must be unique within this structure.
    pub fn field(mut self, name: impl Into<String>, layout: Layout) -> Self {
        self.fields.push((name.into(), layout));
        self
    }

    /// Caps field alignment at `max_align` (one means byte-packed).
    ///
    /// Must be a nonzero power of two. This only changes outer field placement;
    /// it does not remove padding inside nested layouts. The last call wins.
    pub fn packed(mut self, max_align: u64) -> Self {
        self.pack = Some(max_align);
        self
    }

    /// Sets a minimum structure alignment, independently of field packing.
    ///
    /// Must be a nonzero power of two. Tail padding respects this alignment.
    /// The last call wins. Combining this with `packed` models a binary layout
    /// policy; it does not represent Rust's `repr` attribute restrictions.
    pub fn align(mut self, align: u64) -> Self {
        self.align = align;
        self
    }

    /// Calculates field placement and rounds the final size to alignment.
    ///
    /// Returns an error for invalid alignment, duplicate names, or overflow.
    pub fn build(self) -> Result<StructLayout, LayoutError> {
        let mut layout = Layout::new(0, self.align)?;
        if let Some(pack) = self.pack {
            Layout::new(0, pack)?;
        }
        validate_names(&self.fields)?;
        let mut fields = Vec::with_capacity(self.fields.len());
        for (name, original) in self.fields {
            let align = self
                .pack
                .map_or(original.align(), |pack| original.align().min(pack));
            let (combined, offset) = layout.extend(Layout::new(original.size(), align)?)?;
            fields.push(FieldLayout {
                name,
                layout: original,
                offset,
                align,
                padding_before: offset - layout.size(),
            });
            layout = combined;
        }
        let padded = layout.pad_to_align()?;
        Ok(StructLayout {
            layout: padded,
            fields,
            tail_padding: padded.size() - layout.size(),
        })
    }
}

pub(crate) fn validate_names(fields: &[(String, Layout)]) -> Result<(), LayoutError> {
    let mut names = BTreeSet::new();
    for (name, _) in fields {
        if !names.insert(name.as_str()) {
            return Err(LayoutError::DuplicateField(name.clone()));
        }
    }
    Ok(())
}
