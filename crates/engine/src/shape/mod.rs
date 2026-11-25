//! Block shape modifiers (`L-SLAB`): horizontal slabs (¼, ½, ¾, bottom & top), vertical slabs (½, ¼),
//! 2-step stairs (upright & upside-down, auto-connecting inner/outer corners), and 45° pitched slopes/roofs
//! (upright & inverted, auto-connecting hip/valley corners).
//!
//! Shapes live as a sparse `u8` modifier per cell on `Chunk` (`0` = full cube, never stored), so every
//! building material (masonry, woods, concretes, roof tiles, metal, glass, stained glass) gains all shapes
//! without adding new block ids, items, or recipes.
//!
//! Invariants: `FULL == 0` is never stored in a chunk's sparse list. Direction bits `0..4` match
//! `factory::DIRS` (`0=-Z, 1=+X, 2=+Z, 3=-X`, clockwise when seen from above). To add a shape: append a
//! constant below, extend `is_valid`, `boxes.rs` and `mesher/blocks.rs`.

use crate::block::{self, BlockId, TABLE_LEN};
use crate::factory::DIRS;
use crate::math::IVec3;

mod boxes;
#[cfg(test)]
mod tests;

pub use boxes::{boxes_for, hit_boxes, CellBoxes};

/// Default unshaped full cube (never stored in a chunk's sparse shape list).
pub const FULL: u8 = 0;

/// Horizontal bottom slabs: heights ¼, ½, ¾ of a block (`0.25`, `0.50`, `0.75`).
pub const SLAB_B1: u8 = 1;
pub const SLAB_B2: u8 = 2;
pub const SLAB_B3: u8 = 3;

/// Horizontal top slabs (ceiling-aligned): heights ¼, ½, ¾ hanging from `y = 1`.
pub const SLAB_T1: u8 = 4;
pub const SLAB_T2: u8 = 5;
pub const SLAB_T3: u8 = 6;

/// Vertical slabs: half thickness (`8..12`) and quarter thickness (`12..16`), pressed against `DIRS[dir]`.
pub const VSLAB_HALF_BASE: u8 = 8;
pub const VSLAB_QUARTER_BASE: u8 = 12;

/// 2-step stairs: upright (`16..20`) and upside-down (`20..24`), ascending toward `DIRS[dir]`.
pub const STAIR_BASE: u8 = 16;
pub const STAIR_INV_BASE: u8 = 20;

/// 45° pitched slopes / roofs: upright (`24..28`) and inverted (`28..32`), rising toward `DIRS[dir]`.
pub const SLOPE_BASE: u8 = 24;
pub const SLOPE_INV_BASE: u8 = 28;

/// Player-selectable placement modes cycled with Shift+R / Shift+Wheel.
pub const MODE_COUNT: u8 = 8;
pub const MODE_CUBE: u8 = 0;
pub const MODE_SLAB_HALF: u8 = 1;
pub const MODE_SLAB_QUARTER: u8 = 2;
pub const MODE_SLAB_THREE_QUARTER: u8 = 3;
pub const MODE_STAIR: u8 = 4;
pub const MODE_SLOPE: u8 = 5;
pub const MODE_VSLAB_HALF: u8 = 6;
pub const MODE_VSLAB_QUARTER: u8 = 7;

/// Decoded kind of a non-full shape.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Full,
    /// Horizontal slab: thickness in quarters (`1..=3`) and whether aligned to the top (`y = 1`).
    Slab {
        quarters: u8,
        top: bool,
    },
    /// Vertical slab pressed against `DIRS[dir]`, thickness in quarters (`1` or `2`).
    VSlab {
        quarters: u8,
        dir: u8,
    },
    /// 2-step stair ascending toward `DIRS[dir]`.
    Stair {
        dir: u8,
        inverted: bool,
    },
    /// 45° pitched slope rising toward `DIRS[dir]`.
    Slope {
        dir: u8,
        inverted: bool,
    },
}

/// Auto-connected corner state of a stair or 45° slope, derived purely from its horizontal neighbours.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Corner {
    Straight,
    OuterLeft,
    OuterRight,
    InnerLeft,
    InnerRight,
}

/// Every building block that may be placed in slab, stair, vertical-slab or pitched-roof mode.
pub const SHAPEABLE: [bool; TABLE_LEN] = build_shapeable();

const fn build_shapeable() -> [bool; TABLE_LEN] {
    let mut t = [false; TABLE_LEN];
    let mut i = 0;
    while i < block::BLOCK_COUNT {
        let id = i as BlockId;
        t[i] = is_building_block(id);
        i += 1;
    }
    t
}

const fn is_building_block(id: BlockId) -> bool {
    use block::*;
    match id {
        STONE | GRANITE | SANDSTONE | BASALT | LIMESTONE | SLAG | STONE_BRICKS | CONCRETE | PLANKS | GLASS
        | RED_CONCRETE | GREEN_CONCRETE | BLACK_CONCRETE | WHITE_CONCRETE | YELLOW_CONCRETE | BASALT_TILES
        | GRANITE_BRICKS | SANDSTONE_BRICKS | SLAG_BRICKS | FRAMED_GLASS | METAL_PANEL | CATWALK | RED_ROOF
        | SLATE_ROOF | GREEN_ROOF | THATCH | TIMBER_FRAME | WOOD_FLOOR | WINDOW | CHIMNEY_BRICKS | PLASTER
        | CLAY_BRICKS | TERRACOTTA | PORCELAIN | TURQUOISE_TILES | AMBER_TILES => true,
        _ => {
            if let Some((_, kind)) = wood_block(id) {
                return kind == 0 || kind == 1;
            }
            (id >= BLUE_CONCRETE && id < BLUE_CONCRETE + TINT_COUNT as BlockId)
                || (id >= STAINED_GLASS_FIRST && id < STAINED_GLASS_FIRST + STAINED_GLASS_COUNT as BlockId)
        }
    }
}

/// Whether `id` can take a non-full shape modifier.
#[inline]
pub const fn is_shapeable(id: BlockId) -> bool {
    (id as usize) < TABLE_LEN && SHAPEABLE[id as usize]
}

/// Whether `shape` is a valid encoded shape byte (used to validate saves and co-op actions).
#[inline]
pub const fn is_valid(shape: u8) -> bool {
    matches!(shape, FULL | SLAB_B1..=SLAB_T3 | VSLAB_HALF_BASE..=31)
}

/// Decodes a shape byte into its structured [`Kind`]. Invalid bytes decode as [`Kind::Full`].
#[inline]
pub const fn kind(shape: u8) -> Kind {
    match shape {
        SLAB_B1..=SLAB_B3 => Kind::Slab { quarters: shape - SLAB_B1 + 1, top: false },
        SLAB_T1..=SLAB_T3 => Kind::Slab { quarters: shape - SLAB_T1 + 1, top: true },
        VSLAB_HALF_BASE..=11 => Kind::VSlab { quarters: 2, dir: shape - VSLAB_HALF_BASE },
        VSLAB_QUARTER_BASE..=15 => Kind::VSlab { quarters: 1, dir: shape - VSLAB_QUARTER_BASE },
        STAIR_BASE..=19 => Kind::Stair { dir: shape - STAIR_BASE, inverted: false },
        STAIR_INV_BASE..=23 => Kind::Stair { dir: shape - STAIR_INV_BASE, inverted: true },
        SLOPE_BASE..=27 => Kind::Slope { dir: shape - SLOPE_BASE, inverted: false },
        SLOPE_INV_BASE..=31 => Kind::Slope { dir: shape - SLOPE_INV_BASE, inverted: true },
        _ => Kind::Full,
    }
}

/// Encodes a [`Kind`] back to its compact `u8` representation.
#[inline]
pub const fn encode(k: Kind) -> u8 {
    match k {
        Kind::Full => FULL,
        Kind::Slab { quarters, top: false } if quarters >= 1 && quarters <= 3 => SLAB_B1 + quarters - 1,
        Kind::Slab { quarters, top: true } if quarters >= 1 && quarters <= 3 => SLAB_T1 + quarters - 1,
        Kind::Slab { .. } => FULL,
        Kind::VSlab { quarters: 2, dir } => VSLAB_HALF_BASE + (dir & 3),
        Kind::VSlab { quarters: 1, dir } => VSLAB_QUARTER_BASE + (dir & 3),
        Kind::VSlab { .. } => FULL,
        Kind::Stair { dir, inverted: false } => STAIR_BASE + (dir & 3),
        Kind::Stair { dir, inverted: true } => STAIR_INV_BASE + (dir & 3),
        Kind::Slope { dir, inverted: false } => SLOPE_BASE + (dir & 3),
        Kind::Slope { dir, inverted: true } => SLOPE_INV_BASE + (dir & 3),
    }
}

/// Whether `shape` has a horizontal facing and responds to `R` (rotate).
#[inline]
pub const fn is_rotatable(shape: u8) -> bool {
    matches!(kind(shape), Kind::VSlab { .. } | Kind::Stair { .. } | Kind::Slope { .. })
}

/// Rotates a directional shape 90° clockwise (seen from above); horizontal slabs and full cubes are unchanged.
#[inline]
pub const fn rotate_cw(shape: u8) -> u8 {
    match kind(shape) {
        Kind::VSlab { quarters, dir } => encode(Kind::VSlab { quarters, dir: (dir + 1) & 3 }),
        Kind::Stair { dir, inverted } => encode(Kind::Stair { dir: (dir + 1) & 3, inverted }),
        Kind::Slope { dir, inverted } => encode(Kind::Slope { dir: (dir + 1) & 3, inverted }),
        _ => shape,
    }
}

/// Maps placement mode (`0..MODE_COUNT`), player horizontal facing (`0..4`), and whether the player aimed at
/// the upper half of a block / underside of a ceiling into the concrete `u8` shape byte.
#[inline]
pub const fn from_mode(mode: u8, facing: u8, upper: bool) -> u8 {
    let dir = facing & 3;
    match mode {
        MODE_SLAB_HALF => {
            if upper {
                SLAB_T2
            } else {
                SLAB_B2
            }
        }
        MODE_SLAB_QUARTER => {
            if upper {
                SLAB_T1
            } else {
                SLAB_B1
            }
        }
        MODE_SLAB_THREE_QUARTER => {
            if upper {
                SLAB_T3
            } else {
                SLAB_B3
            }
        }
        MODE_STAIR => encode(Kind::Stair { dir, inverted: upper }),
        MODE_SLOPE => encode(Kind::Slope { dir, inverted: upper }),
        MODE_VSLAB_HALF => encode(Kind::VSlab { quarters: 2, dir }),
        MODE_VSLAB_QUARTER => encode(Kind::VSlab { quarters: 1, dir }),
        _ => FULL,
    }
}

/// If the player right-clicks the open inner face of an existing horizontal slab of the same material while
/// holding a horizontal slab mode, returns the grown slab shape (`¼ -> ½ -> ¾ -> Full`).
pub const fn stack_slab(existing: u8, mode: u8, normal: IVec3) -> Option<u8> {
    let add = match mode {
        MODE_SLAB_QUARTER => 1,
        MODE_SLAB_HALF => 2,
        MODE_SLAB_THREE_QUARTER => 3,
        _ => return None,
    };
    match (kind(existing), normal.y) {
        (Kind::Slab { quarters, top: false }, 1) => {
            let sum = quarters + add;
            Some(if sum >= 4 { FULL } else { encode(Kind::Slab { quarters: sum, top: false }) })
        }
        (Kind::Slab { quarters, top: true }, -1) => {
            let sum = quarters + add;
            Some(if sum >= 4 { FULL } else { encode(Kind::Slab { quarters: sum, top: true }) })
        }
        _ => None,
    }
}

/// Auto-connected corner state for a stair or 45° slope at `pos`. `neighbor_shape(p)` returns the shape byte
/// of the cell at `p` (`FULL` for air or non-shaped blocks). Stairs connect only with stairs of the same
/// upright/inverted parity; slopes connect only with slopes of the same parity.
pub fn corner_of(shape: u8, pos: IVec3, mut neighbor_shape: impl FnMut(IVec3) -> u8) -> Corner {
    let (is_slope, dir, inv) = match kind(shape) {
        Kind::Stair { dir, inverted } => (false, dir, inverted),
        Kind::Slope { dir, inverted } => (true, dir, inverted),
        _ => return Corner::Straight,
    };
    let same_family = |s: u8| -> Option<u8> {
        match (is_slope, kind(s)) {
            (false, Kind::Stair { dir: d, inverted }) if inverted == inv => Some(d),
            (true, Kind::Slope { dir: d, inverted }) if inverted == inv => Some(d),
            _ => None,
        }
    };
    let back = DIRS[dir as usize];
    let right_dir = (dir + 1) & 3;
    let left_dir = (dir + 3) & 3;
    let right = DIRS[right_dir as usize];
    let left = DIRS[left_dir as usize];

    if let Some(bd) = same_family(neighbor_shape(pos + back)) {
        if bd == right_dir && same_family(neighbor_shape(pos + left)) != Some(dir) {
            return Corner::OuterRight;
        }
        if bd == left_dir && same_family(neighbor_shape(pos + right)) != Some(dir) {
            return Corner::OuterLeft;
        }
    }
    if let Some(fd) = same_family(neighbor_shape(pos - back)) {
        if fd == right_dir && same_family(neighbor_shape(pos + right)) != Some(dir) {
            return Corner::InnerRight;
        }
        if fd == left_dir && same_family(neighbor_shape(pos + left)) != Some(dir) {
            return Corner::InnerLeft;
        }
    }
    Corner::Straight
}

/// Bitmask of the 6 unit-cube faces (`0:+X, 1:-X, 2:+Y, 3:-Y, 4:+Z, 5:-Z`) that `shape` + `corner`
/// completely covers. Used by the mesher so full cubes adjacent to a flat back/bottom face still cull it.
pub const fn full_face_mask(shape: u8, corner: Corner) -> u8 {
    match kind(shape) {
        Kind::Full => 0b11_1111,
        Kind::Slab { top: false, .. } => 1 << 3, // -Y
        Kind::Slab { top: true, .. } => 1 << 2,  // +Y
        Kind::VSlab { dir, .. } => 1 << dir_face(dir),
        Kind::Stair { dir, inverted } | Kind::Slope { dir, inverted } => {
            let cap = if inverted { 1 << 2 } else { 1 << 3 };
            let back = 1 << dir_face(dir);
            match corner {
                Corner::Straight => cap | back,
                Corner::OuterLeft | Corner::OuterRight => cap,
                Corner::InnerLeft => cap | back | (1 << dir_face((dir + 3) & 3)),
                Corner::InnerRight => cap | back | (1 << dir_face((dir + 1) & 3)),
            }
        }
    }
}

/// Maps `factory::DIRS` index (`0=-Z, 1=+X, 2=+Z, 3=-X`) to `mesher::FACES` index (`0=+X, 1=-X, 4=+Z, 5=-Z`).
#[inline]
pub const fn dir_face(dir: u8) -> u8 {
    match dir & 3 {
        0 => 5, // -Z
        1 => 0, // +X
        2 => 4, // +Z
        _ => 1, // -X
    }
}

/// Converts canonical `(right, back)` cell coordinates `(r, b) ∈ [0, 1]²` into world-aligned cell-local
/// `(x, z)` for facing `dir` (`0=-Z, 1=+X, 2=+Z, 3=-X`). Preserves winding for all four facings.
#[inline]
pub const fn to_local(dir: u8, r: f32, b: f32) -> (f32, f32) {
    match dir & 3 {
        0 => (r, 1.0 - b),
        1 => (b, r),
        2 => (1.0 - r, b),
        _ => (1.0 - b, 1.0 - r),
    }
}

/// Same as [`to_local`] in `f64`, for collision boxes.
#[inline]
pub const fn to_local_f64(dir: u8, r: f64, b: f64) -> (f64, f64) {
    match dir & 3 {
        0 => (r, 1.0 - b),
        1 => (b, r),
        2 => (1.0 - r, b),
        _ => (1.0 - b, 1.0 - r),
    }
}

/// Short player-facing label for a placement mode (`0..MODE_COUNT`).
pub const fn mode_name(mode: u8) -> &'static str {
    match mode {
        MODE_CUBE => "Cube",
        MODE_SLAB_HALF => "Half Slab (½)",
        MODE_SLAB_QUARTER => "Quarter Slab (¼)",
        MODE_SLAB_THREE_QUARTER => "Tall Slab (¾)",
        MODE_STAIR => "Stairs",
        MODE_SLOPE => "Pitched Slope",
        MODE_VSLAB_HALF => "Vertical Slab (½)",
        MODE_VSLAB_QUARTER => "Trim Panel (¼)",
        _ => "Cube",
    }
}

/// Short player-facing label for an already-placed shape (`""` for full cube).
pub const fn shape_label(shape: u8) -> &'static str {
    match kind(shape) {
        Kind::Full => "",
        Kind::Slab { quarters: 1, top: false } => "Quarter Slab",
        Kind::Slab { quarters: 2, top: false } => "Half Slab",
        Kind::Slab { quarters: 3, top: false } => "Tall Slab",
        Kind::Slab { quarters: 1, top: true } => "Top Quarter Slab",
        Kind::Slab { quarters: 2, top: true } => "Top Half Slab",
        Kind::Slab { quarters: 3, top: true } => "Top Tall Slab",
        Kind::Slab { .. } => "Slab",
        Kind::VSlab { quarters: 2, .. } => "Vertical Slab",
        Kind::VSlab { .. } => "Trim Panel",
        Kind::Stair { inverted: false, .. } => "Stairs",
        Kind::Stair { inverted: true, .. } => "Inverted Stairs",
        Kind::Slope { inverted: false, .. } => "Pitched Slope",
        Kind::Slope { inverted: true, .. } => "Inverted Slope",
    }
}
