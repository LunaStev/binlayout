// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Model a packed tagged payload using a union and a separately stored tag.

use binlayout::{Layout, LayoutError, StructLayout, UnionLayout};

fn main() -> Result<(), LayoutError> {
    let payload = UnionLayout::builder()
        .packed(1)
        .field("number", Layout::new(8, 8)?)
        .field("bytes", Layout::new(9, 1)?)
        .build()?;
    let packet = StructLayout::builder()
        .packed(1)
        .field("tag", Layout::new(1, 1)?)
        .field("payload", payload.layout())
        .build()?;
    assert_eq!((packet.size(), packet.align()), (10, 1));
    assert_eq!(packet.offset("payload"), Some(1));
    assert_eq!(payload.offset("number"), Some(0));
    println!(
        "Packet: size={}, align={}, payload offset=1",
        packet.size(),
        packet.align()
    );
    println!(
        "Payload: size={}, both alternatives start at offset 0 within the payload",
        payload.size()
    );
    // Choosing a payload interpretation, encoding the tag, and selecting byte
    // order are responsibilities of the protocol implementation.
    Ok(())
}
