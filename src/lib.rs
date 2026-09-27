// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;

mod error;
mod layout;
mod structure;
mod target;
mod types;
mod union;

pub use error::LayoutError;
pub use layout::Layout;
pub use structure::{FieldLayout, StructLayout, StructLayoutBuilder};
pub use target::{TargetLayout, TargetLayoutBuilder};
pub use types::{Scalar, Type};
pub use union::{UnionLayout, UnionLayoutBuilder};
