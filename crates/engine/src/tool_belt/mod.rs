//! The Tool Belt (GEAR-BELT): worn in the Tool belt slot (`equipment.rs`), it does what the nine hand tools did, with
//! nothing held and no wear. Five tiers ([`BELTS`]): each breaks the blocks a tool is for (stone and ore, wood, soil:
//! `tools::kind_for`) faster and keeps more ore by hand, and from Mk3 on crouch-mining takes more than one block
//! ([`Reach`], `Action::MineWide`). Without a belt the bare hands break at 1× and keep `HAND_YIELD`.
//!
//! Invariants: a wide break takes only blocks of the hit's kind (Mk3's tunnel: of its tool class) that are no machine,
//! tree wood or town block ([`wide_cells`]), so it never eats a build or a factory; the cells come from the core world
//! in a fixed order (deterministic), and the hands call the same function on loaded chunks only to time the swing.
//! Presentation: while mining, the right hand shows the head the belt uses on the block ([`head`]).
//! To add a tier: an item and a [`BELTS`] row, a [`STEPS`] entry (what the step up takes) and a hand recipe
//! (`recipes/gear.rs`, the lint checks the two match).

use crate::block::{self, BlockId, AIR};
use crate::deposits::HAND_YIELD;
use crate::inventory::Inventory;
use crate::item::{
    ItemId, GOLD_KIT, IRON_AXE, IRON_PICKAXE, IRON_PLATE, IRON_SHOVEL, MINING_RIG, STEEL_AXE, STEEL_PICKAXE,
    STEEL_PLATE, STEEL_SHOVEL, STONE_AXE, STONE_PICKAXE, STONE_SHOVEL, TOOL_BELT, TOOL_BELT_MK2, TOOL_BELT_MK3,
    TOOL_BELT_MK4, TOOL_BELT_MK5, VIOLET_KIT,
};
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim};
use crate::tools::{self, ToolKind};

mod hands;
pub(crate) use hands::hand_item;
#[cfg(test)]
mod tests;

/// What a crouched swing takes besides the block hit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    /// Only the block (Mk1, Mk2, and every tier standing up).
    One,
    /// A 1×2 tunnel: the block below a side hit, the next one in on a floor or ceiling (Mk3).
    Column,
    /// The 3×3 face across the hit, of the hit's block only (Mk4).
    Face,
    /// The joined blocks of the hit's kind, up to [`VEIN_MAX`] in all (Mk5).
    Vein,
}

pub struct BeltTier {
    pub item: ItemId,
    /// Break speed on stone, wood and soil, as a multiple of bare hands.
    pub speed: f32,
    /// Ore kept per ore block broken by hand.
    pub ore_yield: u32,
    pub reach: Reach,
    /// The heads the hand shows (pick, axe, spade): the old hand tools' models.
    heads: [ItemId; 3],
}

const STONE_HEADS: [ItemId; 3] = [STONE_PICKAXE, STONE_AXE, STONE_SHOVEL];
const IRON_HEADS: [ItemId; 3] = [IRON_PICKAXE, IRON_AXE, IRON_SHOVEL];
const STEEL_HEADS: [ItemId; 3] = [STEEL_PICKAXE, STEEL_AXE, STEEL_SHOVEL];

/// Mk1 to Mk5: 2× to 10× on its blocks; ore kept stays far below a miner's recovery (docs/TECH_ERAS.md).
pub const BELTS: [BeltTier; 5] = [
    BeltTier { item: TOOL_BELT, speed: 2.0, ore_yield: HAND_YIELD, reach: Reach::One, heads: STONE_HEADS },
    BeltTier { item: TOOL_BELT_MK2, speed: 4.0, ore_yield: HAND_YIELD + 1, reach: Reach::One, heads: IRON_HEADS },
    BeltTier { item: TOOL_BELT_MK3, speed: 6.0, ore_yield: HAND_YIELD + 2, reach: Reach::Column, heads: STEEL_HEADS },
    BeltTier { item: TOOL_BELT_MK4, speed: 8.0, ore_yield: HAND_YIELD + 2, reach: Reach::Face, heads: STEEL_HEADS },
    BeltTier { item: TOOL_BELT_MK5, speed: 10.0, ore_yield: HAND_YIELD + 2, reach: Reach::Vein, heads: STEEL_HEADS },
];

/// What each step up takes, Mk2 first: iron plates and steel plates (as the iron and steel tools took, so the early
/// tiers wait for no kit), then four kits of the tier like the packs.
pub const STEPS: [(ItemId, u32); 4] = [(IRON_PLATE, 6), (STEEL_PLATE, 6), (VIOLET_KIT, 4), (GOLD_KIT, 4)];

/// Items kept only so old saves load (save 87): the nine hand tools and the Mining Rig. No recipe makes them, the
/// creative picker leaves them out, and the recycler still takes them.
pub const RETIRED: [ItemId; 10] = [
    STONE_PICKAXE,
    STONE_AXE,
    STONE_SHOVEL,
    IRON_PICKAXE,
    IRON_AXE,
    IRON_SHOVEL,
    STEEL_PICKAXE,
    STEEL_AXE,
    STEEL_SHOVEL,
    MINING_RIG,
];

/// The most blocks a Mk5 vein cut takes, the one hit included.
pub const VEIN_MAX: usize = 32;

/// The hit faces' outward normals, in `Action::MineWide`'s `face` order.
pub const FACES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// The tier of belt `item`, if it is one.
pub fn tier(item: ItemId) -> Option<&'static BeltTier> {
    BELTS.iter().find(|t| t.item == item)
}

/// The belt above `item` with the item and count that step takes (`equipment::next_tier`).
pub fn next_belt(item: ItemId) -> Option<(ItemId, ItemId, u32)> {
    let i = BELTS.iter().position(|t| t.item == item)?;
    let (cost, n) = *STEPS.get(i)?;
    Some((BELTS[i + 1].item, cost, n))
}

/// How much faster than bare hands `belt` breaks `block`: its tier's speed on stone, wood and soil, 1 elsewhere.
pub fn speed(belt: ItemId, block: BlockId) -> f32 {
    match tier(belt) {
        Some(t) if tools::kind_for(block).is_some() => t.speed,
        _ => 1.0,
    }
}

/// Ore kept from one ore block broken by hand while wearing `belt`.
pub fn ore_yield(belt: ItemId) -> u32 {
    tier(belt).map_or(HAND_YIELD, |t| t.ore_yield)
}

/// What a crouched swing with `belt` takes.
pub fn reach(belt: ItemId) -> Reach {
    tier(belt).map_or(Reach::One, |t| t.reach)
}

/// The head the hand shows while mining `block` with `belt` (an old tool's model), or `NONE` for nothing to show.
pub fn head(belt: ItemId, block: BlockId) -> ItemId {
    let (Some(t), Some(kind)) = (tier(belt), tools::kind_for(block)) else { return ItemId::NONE };
    t.heads[match kind {
        ToolKind::Pickaxe => 0,
        ToolKind::Axe => 1,
        _ => 2,
    }]
}

/// The index in [`FACES`] of `normal`, if it is one.
pub fn face_of(normal: IVec3) -> Option<u8> {
    FACES.iter().position(|&f| f == normal).map(|i| i as u8)
}

/// The other cells a `reach` swing at `pos` (hit through face `face`) takes, in a fixed order. `block_at` reads the
/// world; `free` says a cell is no machine or town block (the hit's own check is the caller's). Only breakable blocks
/// of the hit's kind count: the same block for a face or a vein, the same tool class for a tunnel.
pub fn wide_cells(
    reach: Reach,
    pos: IVec3,
    face: u8,
    mut block_at: impl FnMut(IVec3) -> BlockId,
    free: impl Fn(IVec3) -> bool,
) -> Vec<IVec3> {
    let Some(&normal) = FACES.get(usize::from(face)) else { return Vec::new() };
    let hit = block_at(pos);
    let Some(class) = tools::kind_for(hit).filter(|_| breakable(hit)) else { return Vec::new() };
    let mut takes = |p: IVec3, same_class: bool| {
        let b = block_at(p);
        let kind = if same_class { tools::kind_for(b) == Some(class) } else { b == hit };
        kind && breakable(b) && free(p)
    };
    match reach {
        Reach::One => Vec::new(),
        Reach::Column => {
            let next = if normal.y == 0 { pos - IVec3::new(0, 1, 0) } else { pos - normal };
            if takes(next, true) {
                vec![next]
            } else {
                Vec::new()
            }
        }
        Reach::Face => {
            let (u, v) = across(normal);
            let mut out = Vec::new();
            for du in -1..=1 {
                for dv in -1..=1 {
                    let p = pos + u * du + v * dv;
                    if (du, dv) != (0, 0) && takes(p, false) {
                        out.push(p);
                    }
                }
            }
            out
        }
        Reach::Vein => {
            let mut seen = vec![pos];
            let mut next = 0;
            while next < seen.len() && seen.len() < VEIN_MAX {
                let at = seen[next];
                next += 1;
                for f in FACES {
                    let p = at + f;
                    if seen.len() < VEIN_MAX && !seen.contains(&p) && takes(p, false) {
                        seen.push(p);
                    }
                }
            }
            seen.remove(0);
            seen
        }
    }
}

/// Whether a swing may take `b` at all: something there, breakable, and no grown tree's wood (trees are felled).
fn breakable(b: BlockId) -> bool {
    b != AIR && block::def(b).break_time >= 0.0 && !block::is_tree_wood(b)
}

/// The two axes across a face with `normal`.
fn across(normal: IVec3) -> (IVec3, IVec3) {
    if normal.x != 0 {
        (IVec3::new(0, 1, 0), IVec3::new(0, 0, 1))
    } else if normal.y != 0 {
        (IVec3::new(1, 0, 0), IVec3::new(0, 0, 1))
    } else {
        (IVec3::new(1, 0, 0), IVec3::new(0, 1, 0))
    }
}

impl Inventory {
    /// The worn Tool Belt (`NONE` without one).
    pub fn belt(&self) -> ItemId {
        self.worn[crate::equipment::BELT]
    }

    /// Reading a save from before the belt (version 87): the best hand tool carried (slots, cursor or left hand)
    /// becomes a worn belt of its tier, stone Mk1 to steel Mk3, when the slot is empty or holds the Mining Rig, which
    /// counts as Mk2 (it cost more than one). The old tools stay as items.
    pub(crate) fn belt_from_tools(&mut self) {
        let worn = self.belt();
        if worn != ItemId::NONE && worn != MINING_RIG {
            return;
        }
        let held = self.slots.iter().chain([&self.cursor, &self.offhand]).filter(|s| s.count > 0);
        let tool_tier = |item| [STONE_HEADS, IRON_HEADS, STEEL_HEADS].iter().position(|h| h.contains(&item));
        let best = held.filter_map(|s| tool_tier(s.item)).max();
        let best = if worn == MINING_RIG { Some(best.map_or(1, |b| b.max(1))) } else { best };
        self.worn[crate::equipment::BELT] = best.map_or(ItemId::NONE, |b| BELTS[b].item);
    }
}

impl Sim {
    /// `Action::MineWide`: a crouched swing at `pos` through face `face`. With a belt that reaches further and a hit
    /// that is no machine or town block, it breaks the hit and then [`wide_cells`]; otherwise just the hit.
    pub(crate) fn mine_wide(&mut self, player: PlayerId, pos: IVec3, face: u8) {
        let Some(core) = self.player(player) else { return };
        let reach = reach(core.inventory.belt());
        let (world, factory, towns) = (&mut self.world, &self.factory, &self.settlements);
        let free = |p: IVec3| !factory.occupies(p) && !towns.in_plot(p);
        let cells = if reach != Reach::One && free(pos) {
            wide_cells(reach, pos, face, |p| world.block_anywhere_or_generate(p), free)
        } else {
            Vec::new()
        };
        self.break_block(player, pos);
        for p in cells {
            self.break_block(player, p);
        }
    }
}
