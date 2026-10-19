//! What a town builds, as data (step 13.3a; talked with the user on 2026-10-16, `docs/DECISIONS.md`).
//!
//! - **Buildings** ([`BUILDINGS`]): the shapes each stage adds (2 to 4), the same for every kind. A shape is a box of
//!   storeys (3 cells of wall, then a ceiling) with a door facing the depot, windows, a lamp in each ceiling, a stepped
//!   or flat roof and maybe a chimney. [`cells`] turns one into cells by [`Role`], bottom first, on a foundation.
//! - **Palettes** ([`PALETTES`]): which block each role is, per town kind; the forge and the workshop give every
//!   building a chimney.
//! - **Plot slots** ([`slot_offset`]): a grid [`PITCH`] blocks apart round the depot, nearest first.
//!
//! Only a plot's slot, stage, building and base height are saved (`Town::plots`), so changing a shape changes the
//! town still to build; built blocks stay. To add a building: a row in `BUILDINGS` (append within a stage).

use crate::block::{BlockId, AIR, BASALT, CONCRETE, GLASS, GRANITE, LAMP, LOG, PLANKS, STONE, STONE_BRICKS};
use crate::math::IVec3;

use super::STAGES;

/// Blocks between plot centres; the biggest building is 9 wide.
pub const PITCH: i32 = 12;
/// Rings of plots round the depot (8, 16 and 24 slots).
pub const RINGS: i32 = 3;
/// How far below its floor a building's foundation can reach (a plot's slope plus shallow water).
pub const FOUNDATION_DEPTH: i32 = 7;

/// What a cell of a building is; its kind's palette says which block.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    /// The floor under the walls and the corner stilts.
    Foundation,
    /// Below the floor, between the stilts: filled on land, left open over water.
    Fill,
    Wall,
    Corner,
    Window,
    Roof,
    Lamp,
    Chimney,
    /// Inside a building or its door: cleared of leaves and plants.
    Clear,
}

/// One building: its size along x and z, storeys, and its roof.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shape {
    pub w: i32,
    pub d: i32,
    pub storeys: i32,
    pub peaked: bool,
    pub chimney: bool,
}

const fn shape(w: i32, d: i32, storeys: i32, peaked: bool, chimney: bool) -> Shape {
    Shape { w, d, storeys, peaked, chimney }
}

/// The buildings each stage adds (index: the stage just built minus 1).
pub const BUILDINGS: [&[Shape]; STAGES] = [
    // Camp: two huts and a store.
    &[shape(5, 5, 1, true, false), shape(5, 5, 1, true, false), shape(7, 5, 1, false, false)],
    // Hamlet: two houses and a workshop.
    &[shape(5, 7, 1, true, false), shape(7, 5, 1, true, false), shape(7, 7, 1, false, true)],
    // Village: a hall, two tall houses and a watchtower.
    &[
        shape(9, 7, 1, true, true),
        shape(5, 7, 2, true, false),
        shape(7, 5, 2, true, false),
        shape(3, 3, 3, false, false),
    ],
    // Town: two tall houses, a big hall and a tower.
    &[
        shape(7, 7, 2, true, false),
        shape(7, 7, 2, true, false),
        shape(9, 9, 2, false, true),
        shape(5, 5, 3, true, false),
    ],
    // City: two towers, a great hall and a tall house.
    &[
        shape(5, 5, 4, true, false),
        shape(5, 5, 4, true, false),
        shape(9, 9, 3, false, true),
        shape(7, 7, 3, true, true),
    ],
];

/// The blocks of a kind's buildings.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Palette {
    pub foundation: BlockId,
    pub wall: BlockId,
    pub corner: BlockId,
    pub roof: BlockId,
    pub chimney: BlockId,
    /// Whether every building has a chimney.
    pub chimneys: bool,
}

/// Per town kind (`KINDS` order).
pub const PALETTES: [Palette; 4] = [
    // Harbour: plank walls on log posts under stone-brick roofs.
    Palette {
        foundation: STONE,
        wall: PLANKS,
        corner: LOG,
        roof: STONE_BRICKS,
        chimney: STONE_BRICKS,
        chimneys: false,
    },
    // River town: brick walls on log posts under plank roofs.
    Palette {
        foundation: STONE,
        wall: STONE_BRICKS,
        corner: LOG,
        roof: PLANKS,
        chimney: STONE_BRICKS,
        chimneys: false,
    },
    // Highland workshop: concrete on granite, every house with a chimney.
    Palette {
        foundation: GRANITE,
        wall: CONCRETE,
        corner: GRANITE,
        roof: GRANITE,
        chimney: STONE_BRICKS,
        chimneys: true,
    },
    // Basalt forge: brick on basalt, every house with a chimney.
    Palette {
        foundation: BASALT,
        wall: STONE_BRICKS,
        corner: BASALT,
        roof: BASALT,
        chimney: STONE_BRICKS,
        chimneys: true,
    },
];

impl Palette {
    /// The block a role is built of (air for `Clear`).
    pub fn block(&self, role: Role) -> BlockId {
        match role {
            Role::Foundation | Role::Fill => self.foundation,
            Role::Wall => self.wall,
            Role::Corner => self.corner,
            Role::Window => GLASS,
            Role::Roof => self.roof,
            Role::Lamp => LAMP,
            Role::Chimney => self.chimney,
            Role::Clear => AIR,
        }
    }
}

/// The plot slots round the depot, nearest first: offsets in plots (multiply by [`PITCH`]).
pub fn slots() -> Vec<(i32, i32)> {
    let mut out: Vec<(i32, i32)> = (-RINGS..=RINGS)
        .flat_map(|i| (-RINGS..=RINGS).map(move |j| (i, j)))
        .filter(|&(i, j)| (i, j) != (0, 0))
        .collect();
    crate::math::sort_small_by_key(&mut out, |&(i, j)| (i * i + j * j) * 100 + (i + RINGS) * 10 + (j + RINGS));
    out
}

/// A slot's centre, from the depot's anchor.
pub fn slot_offset(slot: u8) -> IVec3 {
    let (i, j) = slots()[slot as usize];
    IVec3::new(i * PITCH, 0, j * PITCH)
}

/// Cells above a building's floor (walls, roof and chimney).
pub fn height(s: &Shape) -> i32 {
    s.storeys * 4 + s.w.min(s.d) / 2 + 3
}

/// A building's cells by role, from its lowest corner at floor level (foundation below 0), bottom first. `door`
/// is the side facing the depot: 0 is -x, 1 is +x, 2 is -z, 3 is +z.
pub fn cells(s: &Shape, chimney: bool, door: u8) -> Vec<(IVec3, Role)> {
    let (w, d, top) = (s.w, s.d, height(s));
    let low = -FOUNDATION_DEPTH;
    let at = |x: i32, y: i32, z: i32| (((y - low) * d + z) * w + x) as usize;
    let mut grid: Vec<Option<Role>> = vec![None; (w * d * (top - low)) as usize];
    let edge = |x: i32, z: i32| x == 0 || z == 0 || x == w - 1 || z == d - 1;
    let corner = |x: i32, z: i32| (x == 0 || x == w - 1) && (z == 0 || z == d - 1);
    for z in 0..d {
        for x in 0..w {
            for y in low..-1 {
                grid[at(x, y, z)] = Some(if corner(x, z) { Role::Foundation } else { Role::Fill });
            }
            grid[at(x, -1, z)] = Some(Role::Foundation);
        }
    }
    let ceiling = s.storeys * 4 - 1;
    for st in 0..s.storeys {
        let y0 = st * 4;
        for z in 0..d {
            for x in 0..w {
                for y in y0..y0 + 3 {
                    grid[at(x, y, z)] = Some(match () {
                        _ if !edge(x, z) => Role::Clear,
                        _ if corner(x, z) => Role::Corner,
                        _ if st == 0 && y < y0 + 2 && is_door(x, z, w, d, door) => Role::Clear,
                        _ if y == y0 + 1 && (x + z) % 2 == 1 => Role::Window,
                        _ => Role::Wall,
                    });
                }
                let lid = if y0 + 3 == ceiling { Role::Roof } else { Role::Wall };
                grid[at(x, y0 + 3, z)] = Some(lid);
            }
        }
        grid[at(w / 2, y0 + 3, d / 2)] = Some(Role::Lamp);
    }
    if s.peaked {
        for k in 1..=w.min(d) / 2 {
            for z in k..d - k {
                for x in k..w - k {
                    if x == k || z == k || x == w - 1 - k || z == d - 1 - k {
                        grid[at(x, ceiling + k, z)] = Some(Role::Roof);
                    }
                }
            }
        }
    }
    if s.chimney || chimney {
        // A stove's flue from the floor up through the roof.
        for y in 0..top {
            grid[at(w - 2, y, 1)] = Some(Role::Chimney);
        }
    }
    let mut out = Vec::new();
    for y in low..top {
        for z in 0..d {
            for x in 0..w {
                if let Some(role) = grid[at(x, y, z)] {
                    out.push((IVec3::new(x, y, z), role));
                }
            }
        }
    }
    out
}

/// Where a building with a chimney puffs smoke, from its lowest corner at floor level.
pub fn chimney_mouth(s: &Shape) -> IVec3 {
    IVec3::new(s.w - 2, height(s), 1)
}

/// Whether (x, z) is the door's cell on side `door`.
fn is_door(x: i32, z: i32, w: i32, d: i32, door: u8) -> bool {
    match door {
        0 => x == 0 && z == d / 2,
        1 => x == w - 1 && z == d / 2,
        2 => z == 0 && x == w / 2,
        _ => z == d - 1 && x == w / 2,
    }
}
