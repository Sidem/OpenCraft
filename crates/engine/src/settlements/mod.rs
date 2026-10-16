//! Settlements (Milestone 13): towns the player founds by placing a Town Depot at a site that fits a town kind. Core
//! state on `Sim`: the towns, each with its depot's anchor, its kind, the stages it has built, what it has delivered
//! toward the next, whether it is stalled, its building plots and its drones.
//!
//! - **Kinds** are rows ([`KINDS`]): what place each needs (open water near, or a biome) and how it is called. A site
//!   takes the first row that fits, so water comes before biome: a basalt shore is a harbour, and a step inland a forge.
//! - **Sites** ([`Settlements::site`]): a pure function of the generator and the position (and the towns already
//!   founded): never within [`SPAWN_CLEAR`] blocks of spawn (the spawn plains), at least [`SPACING`] blocks from any
//!   other town, and a kind's place within [`SHORE`] blocks. It reads the generator's fields, never chunks, so a site
//!   is the same explored or not; a canal dug by hand doesn't make a harbour.
//! - **Founding**: `put_footprint` (`action/multiblock.rs`) refuses a depot at an unfitting site and founds the town
//!   when one is placed. Breaking the depot (`dismantle`) leaves its town **dormant** (`sleep`: stage, progress and
//!   plots kept, nothing runs), and a depot put back on its old spot wakes it (`site`, `found`). Breaking any block of a
//!   finished building sends the town's cursor back to it (`broken`), so its drones rebuild it; a town's own block drops
//!   nothing, so rebuilt blocks can't be farmed.
//! - **Orders and upkeep** (`orders.rs` data, `run.rs` the tick): the depot takes in only its town's goods; each minute
//!   the town uses up its upkeep from the depot, or stalls until it is there; while not stalled the rest goes toward the
//!   open order, and a complete order builds the next stage.
//! - **Growth** (`plans.rs` data, `plots.rs` where, `build.rs` the town's drones): each stage built adds its buildings
//!   on plots round the depot, and the town's own drones build them, free, bottom first.
//! - **Perks** (`perks/`, one module each): what a supplied town gives back, paused while it is stalled.
//!
//! Invariants: the towns are in founding order; their bytes follow the opening scene (save version 48; the stage, order
//! and upkeep state since 49; plots, cursor and drones since 50; dormant since 51). A town's `delivered` has one count per line of its
//! open order (none once fully grown).
//! To add a kind: a row in `KINDS` (append only: saves store the index), its `Needs` and its `orders::UPKEEP` row.

mod build;
mod orders;
mod perks;
mod plans;
mod plots;
mod run;
mod state;

pub use build::{fleet, TownDrone};
pub use orders::{upkeep, Line, ORDERS, STAGES, STAGE_NAMES};
pub use perks::harbour::{self, Made};
pub use perks::{discounted, gear_cost};
pub use plans::{chimney_mouth, height};
pub use plots::Plot;

use crate::block::BlockId;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::worldgen::{Biome, Water, WorldGen};

/// No town within this many blocks of spawn: the spawn plains, where the climate is still calm (`worldgen/biome.rs`).
pub const SPAWN_CLEAR: i32 = 200;
/// Least distance between two towns, in blocks, horizontally.
pub const SPACING: i32 = 300;
/// How near a town's water must be, in blocks, horizontally.
pub const SHORE: i32 = 6;
/// Sampling step for the water nearby, in blocks.
const SHORE_STEP: usize = 2;

/// What place a town kind needs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Needs {
    /// This water within `SHORE` blocks.
    Water(Water),
    /// The depot's column in this biome.
    Biome(Biome),
}

/// A kind of town: its name, the place it needs and what that is called, and its colour on the maps (`0xRRGGBB`: its
/// marker and the shading of the places it fits).
pub struct KindDef {
    pub name: &'static str,
    pub needs: Needs,
    pub place: &'static str,
    pub colour: i32,
}

/// The town kinds, settled with the user on 2026-10-16 (`docs/DEV_PLAN.md` section 1). Saves store the index. Their
/// perks (step 13.4): contract work, drone relay, gear works, better smelting.
pub const KINDS: [KindDef; 4] = [
    KindDef { name: "Harbour", needs: Needs::Water(Water::Sea), place: "on a sea shore", colour: 0x4aa3ff },
    KindDef {
        name: "River Town",
        needs: Needs::Water(Water::River),
        place: "on a river or lake bank",
        colour: 0x7be0c8,
    },
    KindDef {
        name: "Highland Workshop",
        needs: Needs::Biome(Biome::Highlands),
        place: "in the granite highlands",
        colour: 0xc9a46a,
    },
    KindDef {
        name: "Basalt Forge",
        needs: Needs::Biome(Biome::BasaltFields),
        place: "in the basalt fields",
        colour: 0xe0603a,
    },
];
/// The kinds by index, for their perks.
pub const HARBOUR: u8 = 0;
pub const RIVER_TOWN: u8 = 1;
pub const WORKSHOP: u8 = 2;
pub const FORGE: u8 = 3;

/// Minutes of upkeep a depot stocks of each upkeep good.
pub const STOCK_MINUTES: u32 = 2;

/// A town's state for the host (`Town::state`): growing or supplied, stalled, dormant (its depot broken).
pub const GROWING: u8 = 0;
pub const STALLED: u8 = 1;
pub const DORMANT: u8 = 2;

/// A town: where its depot stands (the anchor), its kind (an index into `KINDS`) and how far it has grown.
#[derive(Clone, PartialEq, Debug)]
pub struct Town {
    pub pos: IVec3,
    pub kind: u8,
    /// Stages built, 0 (only the depot) to `STAGES` (a city).
    pub built: u8,
    /// Delivered toward each line of the open order (empty once fully grown).
    pub delivered: Vec<u32>,
    /// Whether a minute's upkeep is missing: the order stops filling and the perk pauses.
    pub stalled: bool,
    /// The tick the next minute's upkeep is due (once a camp stands).
    pub due: u64,
    /// Whether its depot is broken: no orders, upkeep or building until a depot is put back on its spot.
    pub dormant: bool,
    /// Stages whose buildings have plots (behind `built` for a tick, or after loading an older save).
    pub planned: u8,
    /// Its buildings' places, in building order (`plots.rs`).
    pub plots: Vec<Plot>,
    /// Plots before this one are finished.
    pub cursor: u16,
    /// Its drones out of the depot (`build.rs`).
    pub drones: Vec<TownDrone>,
    /// A harbour's chosen product (`NONE` for none) and what is paid toward the next, per line of its price
    /// (`perks/harbour/`).
    pub product: ItemId,
    pub paid: Vec<u32>,
}

impl Town {
    /// A town just founded: only its depot stands, and the first order is open.
    pub fn new(pos: IVec3, kind: u8) -> Town {
        let delivered = vec![0; ORDERS[0].len()];
        Town {
            pos,
            kind,
            built: 0,
            delivered,
            stalled: false,
            due: 0,
            dormant: false,
            planned: 0,
            plots: Vec::new(),
            cursor: 0,
            drones: Vec::new(),
            product: ItemId::NONE,
            paid: Vec::new(),
        }
    }

    /// `GROWING`, `STALLED` or `DORMANT`.
    pub fn state(&self) -> u8 {
        match (self.dormant, self.stalled) {
            (true, _) => DORMANT,
            (_, true) => STALLED,
            _ => GROWING,
        }
    }

    /// The open order: what builds the next stage (empty once fully grown).
    pub fn order(&self) -> &'static [Line] {
        ORDERS.get(self.built as usize).copied().unwrap_or(&[])
    }

    /// What it uses up a minute (none before the camp).
    pub fn upkeep(&self) -> Vec<Line> {
        upkeep(self.kind, self.built)
    }

    /// What its depot takes in, and up to how many it holds of each: `STOCK_MINUTES` of upkeep (the camp's already
    /// before it stands, so the player can stock up), plus, unless stalled, what the open order still misses. So the
    /// store never fills with one good, and a stalled town only takes what ends the stall.
    pub fn wants(&self) -> Vec<Line> {
        let mut wants: Vec<Line> =
            upkeep(self.kind, self.built.max(1)).iter().map(|&(i, n)| (i, n * STOCK_MINUTES)).collect();
        if self.stalled {
            return wants;
        }
        for (&(item, want), &got) in self.order().iter().zip(&self.delivered) {
            if got >= want {
                continue;
            }
            match wants.iter_mut().find(|w| w.0 == item) {
                Some(w) => w.1 += want - got,
                None => wants.push((item, want - got)),
            }
        }
        wants
    }
}

/// Why no town can be founded at a place.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unfit {
    /// Within `SPAWN_CLEAR` of spawn; how far it is.
    NearSpawn(i32),
    /// Within `SPACING` of a town of this kind; how far it is.
    NearTown(u8, i32),
    /// Within `SPACING` of a dormant town of this kind (its depot broken), but not on its old spot; how far that is.
    Dormant(u8, i32),
    /// No kind's place is here.
    NoKind,
}

impl Unfit {
    /// One line for the placing preview.
    pub fn text(self) -> String {
        match self {
            Unfit::NearSpawn(d) => {
                format!("No town here: too close to spawn ({d} blocks); towns stand at least {SPAWN_CLEAR} out")
            }
            Unfit::NearTown(kind, d) => format!(
                "No town here: your {} is {d} blocks away; towns stand at least {SPACING} apart",
                KINDS[kind as usize].name.to_lowercase()
            ),
            Unfit::Dormant(kind, d) => format!(
                "Your {} sleeps without its depot: put one back on its old spot ({d} blocks away, marked) to wake it",
                KINDS[kind as usize].name.to_lowercase()
            ),
            Unfit::NoKind => format!(
                "No town fits here: towns need the sea or a river within {SHORE} blocks, granite highlands or basalt \
                 fields"
            ),
        }
    }
}

/// The towns founded so far, in founding order.
#[derive(Default)]
pub struct Settlements {
    pub towns: Vec<Town>,
    /// What the players' machines have ever made (the harbour offers only those).
    pub made: Made,
}

impl Settlements {
    /// The kind of town a depot anchored at `at` would found (an index into `KINDS`), or why none.
    pub fn site(&self, gen: &WorldGen, at: IVec3) -> Result<u8, Unfit> {
        if let Some(t) = self.towns.iter().find(|t| t.dormant && t.pos == at) {
            return Ok(t.kind);
        }
        let spawn = far(IVec3::ZERO, at);
        if spawn < SPAWN_CLEAR {
            return Err(Unfit::NearSpawn(spawn));
        }
        let nearest = self.towns.iter().map(|t| (t, far(t.pos, at))).min_by_key(|t| t.1);
        if let Some((t, d)) = nearest.filter(|t| t.1 < SPACING) {
            return Err(if t.dormant { Unfit::Dormant(t.kind, far3(t.pos, at)) } else { Unfit::NearTown(t.kind, d) });
        }
        kind_at(gen, at.x, at.z).ok_or(Unfit::NoKind)
    }

    /// The town (awake) whose depot is anchored at `pos`.
    pub fn at(&self, pos: IVec3) -> Option<&Town> {
        self.towns.iter().find(|t| t.pos == pos && !t.dormant)
    }

    /// Founds a town of `kind` with its depot at `pos`, or wakes the dormant town whose depot stood there (its upkeep
    /// next due a minute after `tick`).
    pub fn found(&mut self, pos: IVec3, kind: u8, tick: u64) {
        match self.towns.iter_mut().find(|t| t.dormant && t.pos == pos) {
            Some(t) => (t.dormant, t.due) = (false, tick + run::MINUTE),
            None => self.towns.push(Town::new(pos, kind)),
        }
    }

    /// Its depot at `pos` was broken: that town goes dormant, keeping its stage, progress and plots; its drones out
    /// are lost.
    pub fn sleep(&mut self, pos: IVec3) {
        if let Some(t) = self.towns.iter_mut().find(|t| t.pos == pos) {
            (t.dormant, t.drones) = (true, Vec::new());
        }
    }

    /// Block `id` at `pos` was broken: a town whose finished plot held it rebuilds from that plot on. Whether it was a
    /// town's own block, which drops nothing (so rebuilt blocks can't be farmed): its plan's block there, from the
    /// floor up (below the floor a foundation cell may be the natural ground).
    pub fn broken(&mut self, pos: IVec3, id: BlockId) -> bool {
        let mut owned = false;
        for t in &mut self.towns {
            for (k, p) in t.plots.iter().enumerate().filter(|(_, p)| p.holds(t.pos, pos)) {
                t.cursor = t.cursor.min(k as u16);
                owned |= pos.y >= p.base - 1 && p.cells(t.pos, t.kind).iter().any(|c| c.0 == pos && c.1 == id);
            }
        }
        owned
    }
}

/// The kind of town column (x, z) fits by its place alone (an index into `KINDS`, the first that fits), ignoring spawn
/// and other towns.
pub fn kind_at(gen: &WorldGen, x: i32, z: i32) -> Option<u8> {
    let (sea, river) = water_near(gen, x, z);
    let biome = (gen.version() >= 2).then(|| gen.biome_at(x, z, gen.height_at(x, z)));
    let fits = |needs: Needs| match needs {
        Needs::Water(Water::Sea) => sea,
        Needs::Water(Water::River) => river,
        Needs::Biome(b) => biome == Some(b),
    };
    KINDS.iter().position(|k| fits(k.needs)).map(|i| i as u8)
}

/// Whole blocks between two cells (rounded down).
fn far3(a: IVec3, b: IVec3) -> i32 {
    let d = (a - b).as_vec3();
    d.length() as i32
}

/// Whole blocks between two columns, horizontally (rounded down).
fn far(a: IVec3, b: IVec3) -> i32 {
    let (dx, dz) = ((a.x - b.x) as f64, (a.z - b.z) as f64);
    (dx * dx + dz * dz).sqrt() as i32
}

/// Whether the sea, and whether a river or lake, lies within `SHORE` blocks of (x, z), sampled every `SHORE_STEP`.
fn water_near(gen: &WorldGen, x: i32, z: i32) -> (bool, bool) {
    let (mut sea, mut river) = (false, false);
    for dz in (-SHORE..=SHORE).step_by(SHORE_STEP) {
        for dx in (-SHORE..=SHORE).step_by(SHORE_STEP) {
            if dx * dx + dz * dz > SHORE * SHORE {
                continue;
            }
            match gen.open_water(x + dx, z + dz) {
                Some(Water::Sea) => sea = true,
                Some(Water::River) => river = true,
                None => {}
            }
            if sea && river {
                return (sea, river);
            }
        }
    }
    (sea, river)
}

#[cfg(test)]
mod tests;
