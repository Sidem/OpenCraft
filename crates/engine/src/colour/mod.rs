//! Per-face surface paint modifier (L-COLOUR): realistic architectural spray-painting on concrete, plaster, bricks,
//! tiles, metal panels, roof tiles and timber frame. Woods stay natural-only (`block/woods.rs`); stained glass is
//! melted in the Glass Kiln (`block/kiln.rs`).
//!
//! Each painted cell stores a packed [`FacePaint`] (`u32`, 6 faces × 5 bits = 30 bits, face order `+X,-X,+Y,-Y,+Z,-Z`)
//! sparsely on `Chunk::paint` (0 heap bytes on unpainted chunks). Palette index `0` is unpainted; `1..=16` match the
//! 16 pigments 1-to-1 (`PIGMENT_RED..PIGMENT_YELLOW`, then `PIGMENT_BLUE..PIGMENT_NAVY`). One pigment fills
//! [`FACES_PER_PIGMENT`] (24) painted surfaces — 4 full cubes' worth of paint — tracked deterministically in
//! [`PaintCharges`] at the end of `Sim`.

use crate::block::{
    BlockId, BASALT_TILES, BLACK_CONCRETE, BLUE_CONCRETE, CATWALK, CHIMNEY_BRICKS, CONCRETE, GRANITE_BRICKS,
    GREEN_CONCRETE, GREEN_ROOF, METAL_PANEL, PLASTER, RED_CONCRETE, RED_ROOF, SANDSTONE_BRICKS, SLAG_BRICKS,
    SLATE_ROOF, STONE_BRICKS, TABLE_LEN, TIMBER_FRAME, TINT_COUNT, WHITE_CONCRETE, YELLOW_CONCRETE,
};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Inventory;
use crate::item::{ItemId, PIGMENT_BLACK, PIGMENT_BLUE, PIGMENT_GREEN, PIGMENT_RED, PIGMENT_WHITE, PIGMENT_YELLOW};
use crate::math::IVec3;

#[cfg(test)]
mod tests;

/// Unpainted surface (natural block texture).
pub const NONE: u8 = 0;
/// Number of paint colours (`1..=16`), 1-to-1 with the 16 pigments.
pub const COUNT: usize = 16;
/// Surfaces painted per single pigment item in normal mode (4 full 6-sided cubes).
pub const FACES_PER_PIGMENT: u8 = 24;

/// Six faces × 5 bits (`0..=16`) packed into one `u32`, in `block::FACE_*` order (`+X, -X, +Y, -Y, +Z, -Z`).
pub type FacePaint = u32;

/// Paint palette in linear-friendly sRGB `[0..1]`; index `0` is neutral white (unused when `paintId == 0`).
pub const PALETTE: [[f32; 3]; COUNT + 1] = [
    [1.00, 1.00, 1.00], // 0: unpainted
    [0.72, 0.24, 0.20], // 1: Red
    [0.25, 0.50, 0.28], // 2: Green
    [0.16, 0.17, 0.20], // 3: Black
    [0.92, 0.92, 0.90], // 4: White
    [0.88, 0.72, 0.22], // 5: Yellow
    [0.22, 0.40, 0.72], // 6: Blue
    [0.84, 0.46, 0.16], // 7: Orange
    [0.50, 0.28, 0.62], // 8: Purple
    [0.20, 0.60, 0.64], // 9: Cyan
    [0.56, 0.74, 0.22], // 10: Lime
    [0.86, 0.56, 0.62], // 11: Pink
    [0.52, 0.70, 0.88], // 12: Sky
    [0.50, 0.52, 0.55], // 13: Grey
    [0.45, 0.30, 0.20], // 14: Brown
    [0.44, 0.13, 0.20], // 15: Maroon
    [0.15, 0.21, 0.44], // 16: Navy
];

/// Display names of swatch `0..=16`.
pub const NAMES: [&str; COUNT + 1] = [
    "Strip Paint",
    "Red",
    "Green",
    "Black",
    "White",
    "Yellow",
    "Blue",
    "Orange",
    "Purple",
    "Cyan",
    "Lime",
    "Pink",
    "Sky",
    "Grey",
    "Brown",
    "Maroon",
    "Navy",
];

/// Swatch CSS hex strings for the held-item HUD chip (`0` = empty / strip).
pub const HEX: [&str; COUNT + 1] = [
    "", "#b83d33", "#408047", "#292b33", "#ebebeb", "#e0b838", "#3866b8", "#d67529", "#80479e", "#3399a3", "#8fbd38",
    "#db8f9e", "#85b3e0", "#80858c", "#734d33", "#702133", "#263670",
];

/// Flat float slice (`17 * 3 = 51` floats) for uploading `uniform vec3 u_colourPalette[17]` to WebGL2.
pub fn palette_flat() -> Vec<f32> {
    let mut out = Vec::with_capacity((COUNT + 1) * 3);
    for c in PALETTE {
        out.extend_from_slice(&c);
    }
    out
}

/// Paint colour (`0..=16`) on `face` (`0..6`) of a packed cell.
#[inline]
pub const fn get_face(packed: FacePaint, face: usize) -> u8 {
    if face < 6 {
        ((packed >> (face * 5)) & 31) as u8
    } else {
        NONE
    }
}

/// Returns `packed` with `face` (`0..6`) set to `colour` (`0..=16`).
#[inline]
pub const fn set_face(packed: FacePaint, face: usize, colour: u8) -> FacePaint {
    if face >= 6 {
        return packed;
    }
    let shift = face * 5;
    let mask = !(31u32 << shift);
    (packed & mask) | ((colour as u32 & 31) << shift)
}

/// Packs all six faces with the same `colour` (used when migrating legacy dyed concrete blocks).
#[inline]
pub const fn all_faces(colour: u8) -> FacePaint {
    if colour == NONE {
        return 0;
    }
    let c = colour as u32 & 31;
    c | (c << 5) | (c << 10) | (c << 15) | (c << 20) | (c << 25)
}

/// Maps a unit axis normal (`hit.normal`) to its face index (`0..6`: `+X, -X, +Y, -Y, +Z, -Z`).
#[inline]
pub const fn face_of_normal(n: IVec3) -> Option<usize> {
    match (n.x, n.y, n.z) {
        (1, 0, 0) => Some(0),
        (-1, 0, 0) => Some(1),
        (0, 1, 0) => Some(2),
        (0, -1, 0) => Some(3),
        (0, 0, 1) => Some(4),
        (0, 0, -1) => Some(5),
        _ => None,
    }
}

/// Pigment item corresponding to paint colour `1..=16` (`None` for `0` or out of range).
#[inline]
pub const fn pigment_of(colour: u8) -> Option<ItemId> {
    match colour {
        1 => Some(PIGMENT_RED),
        2 => Some(PIGMENT_GREEN),
        3 => Some(PIGMENT_BLACK),
        4 => Some(PIGMENT_WHITE),
        5 => Some(PIGMENT_YELLOW),
        6..=16 => Some(ItemId(PIGMENT_BLUE.0 + (colour - 6) as u16)),
        _ => None,
    }
}

/// Paint colour (`1..=16`) corresponding to a pigment item.
#[inline]
pub const fn colour_of_pigment(item: ItemId) -> Option<u8> {
    match item {
        PIGMENT_RED => Some(1),
        PIGMENT_GREEN => Some(2),
        PIGMENT_BLACK => Some(3),
        PIGMENT_WHITE => Some(4),
        PIGMENT_YELLOW => Some(5),
        _ if item.0 >= PIGMENT_BLUE.0 && item.0 < PIGMENT_BLUE.0 + TINT_COUNT as u16 => {
            Some(6 + (item.0 - PIGMENT_BLUE.0) as u8)
        }
        _ => None,
    }
}

/// If `id` is one of the 16 retired dyed concrete blocks, returns its paint colour (`1..=16`).
#[inline]
pub const fn legacy_concrete_colour(id: BlockId) -> Option<u8> {
    match id {
        RED_CONCRETE => Some(1),
        GREEN_CONCRETE => Some(2),
        BLACK_CONCRETE => Some(3),
        WHITE_CONCRETE => Some(4),
        YELLOW_CONCRETE => Some(5),
        _ if id >= BLUE_CONCRETE && id < BLUE_CONCRETE + TINT_COUNT as BlockId => Some(6 + (id - BLUE_CONCRETE) as u8),
        _ => None,
    }
}

/// Whether `id` is a retired dyed concrete block.
#[inline]
pub const fn is_legacy_concrete(id: BlockId) -> bool {
    legacy_concrete_colour(id).is_some()
}

/// Whether `item` is one of the 16 retired dyed concrete items (hidden from new crafting/creative, still placeable on old saves).
#[inline]
pub fn is_legacy_concrete_item(item: ItemId) -> bool {
    item.as_block().is_some_and(is_legacy_concrete)
}

/// Whether `id` accepts surface paint from the Paint Sprayer.
/// Concrete, plaster, masonry bricks/tiles, metal panels, catwalks, roof tiles and timber frame are paintable;
/// all 10 wood species stay natural-only and glass is coloured via the Glass Kiln.
pub const PAINTABLE: [bool; TABLE_LEN] = build_paintable();

const fn build_paintable() -> [bool; TABLE_LEN] {
    let mut t = [false; TABLE_LEN];
    let list = [
        CONCRETE,
        PLASTER,
        STONE_BRICKS,
        GRANITE_BRICKS,
        SANDSTONE_BRICKS,
        SLAG_BRICKS,
        CHIMNEY_BRICKS,
        BASALT_TILES,
        METAL_PANEL,
        CATWALK,
        RED_ROOF,
        SLATE_ROOF,
        GREEN_ROOF,
        TIMBER_FRAME,
    ];
    let mut i = 0;
    while i < list.len() {
        t[list[i] as usize] = true;
        i += 1;
    }
    t
}

#[inline]
pub fn is_paintable(id: BlockId) -> bool {
    PAINTABLE.get(id as usize).copied().unwrap_or(false)
}

/// Deterministic per-world spray charges remaining per pigment colour (1 pigment = 24 painted surfaces).
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct PaintCharges {
    pub charges: [u8; COUNT],
}

impl PaintCharges {
    /// Remaining buffered spray charges for `colour` (`1..=16`).
    #[inline]
    pub fn remaining(&self, colour: u8) -> u8 {
        if colour == NONE || colour as usize > COUNT {
            0
        } else {
            self.charges[colour as usize - 1]
        }
    }

    /// Whether the player can spray `colour` (stripping is always free; creative is always free).
    pub fn can_paint(&self, colour: u8, inv: &Inventory, creative: bool) -> bool {
        if colour == NONE || creative {
            return true;
        }
        let Some(pigment) = pigment_of(colour) else { return false };
        self.remaining(colour) > 0 || inv.count(pigment) > 0
    }

    /// Consumes one surface charge for `colour`, refilling from 1 pigment in `inv` if needed.
    pub fn spend(&mut self, colour: u8, inv: &mut Inventory, creative: bool) -> bool {
        if colour == NONE || creative {
            return true;
        }
        let Some(pigment) = pigment_of(colour) else { return false };
        let slot = &mut self.charges[colour as usize - 1];
        if *slot > 0 {
            *slot -= 1;
            return true;
        }
        if inv.remove(pigment, 1) {
            *slot = FACES_PER_PIGMENT - 1;
            return true;
        }
        false
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        for &c in &self.charges {
            w.u8(c);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Self> {
        let mut charges = [0u8; COUNT];
        for c in &mut charges {
            *c = r.u8()?;
        }
        Some(Self { charges })
    }
}
