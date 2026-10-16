//! Settlements (Milestone 13): towns the player founds by placing a Town Depot at a site that fits a town kind. Core
//! state on `Sim`: the towns, each with its depot's anchor, its kind, the stages it has built, what it has delivered
//! toward the next, whether it is stalled, its building plots and its drones. The perks come in step 13.4.
//!
//! - **Kinds** are rows ([`KINDS`]): what place each needs (open water near, or a biome) and how it is called. A site
//!   takes the first row that fits, so water comes before biome: a basalt shore is a harbour, and a step inland a forge.
//! - **Sites** ([`Settlements::site`]): a pure function of the generator and the position (and the towns already
//!   founded): never within [`SPAWN_CLEAR`] blocks of spawn (the spawn plains), at least [`SPACING`] blocks from any
//!   other town, and a kind's place within [`SHORE`] blocks. It reads the generator's fields, never chunks, so a site
//!   is the same explored or not; a canal dug by hand doesn't make a harbour.
//! - **Founding**: `put_footprint` (`action/multiblock.rs`) refuses a depot at an unfitting site and founds the town
//!   when one is placed; breaking the depot (`dismantle`) removes its town.
//! - **Orders and upkeep** (`orders.rs` data, `run.rs` the tick): the depot takes in only its town's goods; each minute
//!   the town uses up its upkeep from the depot, or stalls until it is there; while not stalled the rest goes toward the
//!   open order, and a complete order builds the next stage.
//! - **Growth** (`plans.rs` data, `plots.rs` where, `build.rs` the town's drones): each stage built adds its buildings
//!   on plots round the depot, and the town's own drones build them, free, bottom first.
//!
//! Invariants: the towns are in founding order; their bytes follow the opening scene (save version 48; the stage, order
//! and upkeep state since 49; plots, cursor and drones since 50). A town's `delivered` has one count per line of its
//! open order (none once fully grown).
//! To add a kind: a row in `KINDS` (append only: saves store the index), its `Needs` and its `orders::UPKEEP` row.

mod build;
mod orders;
mod plans;
mod plots;
mod run;

pub use build::{fleet, TownDrone};
pub use orders::{upkeep, Line, ORDERS, STAGES, STAGE_NAMES};
pub use plans::{chimney_mouth, height};
pub use plots::Plot;

use crate::bytes::{ByteReader, ByteWriter};
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

/// A kind of town: its name, the place it needs and what that is called.
pub struct KindDef {
    pub name: &'static str,
    pub needs: Needs,
    pub place: &'static str,
}

/// The town kinds, settled with the user on 2026-10-16 (`docs/DEV_PLAN.md` section 1). Saves store the index. Their
/// perks (step 13.4): contract work, drone relay, gear works, better smelting.
pub const KINDS: [KindDef; 4] = [
    KindDef { name: "Harbour", needs: Needs::Water(Water::Sea), place: "on a sea shore" },
    KindDef { name: "River Town", needs: Needs::Water(Water::River), place: "on a river or lake bank" },
    KindDef { name: "Highland Workshop", needs: Needs::Biome(Biome::Highlands), place: "in the granite highlands" },
    KindDef { name: "Basalt Forge", needs: Needs::Biome(Biome::BasaltFields), place: "in the basalt fields" },
];

/// Minutes of upkeep a depot stocks of each upkeep good.
pub const STOCK_MINUTES: u32 = 2;

/// A town: where its depot stands (the anchor), its kind (an index into `KINDS`) and how far it has grown.
#[derive(Clone, PartialEq, Debug)]
pub struct Town {
    pub pos: IVec3,
    pub kind: u8,
    /// Stages built, 0 (only the depot) to `STAGES` (a city).
    pub built: u8,
    /// Delivered toward each line of the open order (empty once fully grown).
    pub delivered: Vec<u32>,
    /// Whether a minute's upkeep is missing: the order stops filling (and, from 13.4, the perk pauses).
    pub stalled: bool,
    /// The tick the next minute's upkeep is due (once a camp stands).
    pub due: u64,
    /// Stages whose buildings have plots (behind `built` for a tick, or after loading an older save).
    pub planned: u8,
    /// Its buildings' places, in building order (`plots.rs`).
    pub plots: Vec<Plot>,
    /// Plots before this one are finished.
    pub cursor: u16,
    /// Its drones out of the depot (`build.rs`).
    pub drones: Vec<TownDrone>,
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
            planned: 0,
            plots: Vec::new(),
            cursor: 0,
            drones: Vec::new(),
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
}

impl Settlements {
    /// The kind of town a depot anchored at `at` would found (an index into `KINDS`), or why none.
    pub fn site(&self, gen: &WorldGen, at: IVec3) -> Result<u8, Unfit> {
        let spawn = far(IVec3::ZERO, at);
        if spawn < SPAWN_CLEAR {
            return Err(Unfit::NearSpawn(spawn));
        }
        let nearest = self.towns.iter().map(|t| (t.kind, far(t.pos, at))).min_by_key(|t| t.1);
        if let Some((kind, d)) = nearest.filter(|t| t.1 < SPACING) {
            return Err(Unfit::NearTown(kind, d));
        }
        let (sea, river) = water_near(gen, at.x, at.z);
        let biome = (gen.version() >= 2).then(|| gen.biome_at(at.x, at.z, gen.height_at(at.x, at.z)));
        let fits = |needs: Needs| match needs {
            Needs::Water(Water::Sea) => sea,
            Needs::Water(Water::River) => river,
            Needs::Biome(b) => biome == Some(b),
        };
        KINDS.iter().position(|k| fits(k.needs)).map(|i| i as u8).ok_or(Unfit::NoKind)
    }

    /// The town whose depot is anchored at `pos`.
    pub fn at(&self, pos: IVec3) -> Option<&Town> {
        self.towns.iter().find(|t| t.pos == pos)
    }

    /// Founds a town of `kind` with its depot at `pos`.
    pub fn found(&mut self, pos: IVec3, kind: u8) {
        self.towns.push(Town::new(pos, kind));
    }

    /// Removes the town whose depot was anchored at `pos`, if there is one.
    pub fn remove(&mut self, pos: IVec3) {
        self.towns.retain(|t| t.pos != pos);
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.towns.len());
        for t in &self.towns {
            w.ivec3(t.pos);
            w.u8(t.kind);
            w.u8(t.built);
            w.count(t.delivered.len());
            t.delivered.iter().for_each(|&d| w.u32(d));
            w.bool(t.stalled);
            w.u64(t.due);
            w.u8(t.planned);
            w.count(t.plots.len());
            t.plots.iter().for_each(|p| p.write(w));
            w.u16(t.cursor);
            w.count(t.drones.len());
            t.drones.iter().for_each(|d| d.write(w));
        }
    }

    /// Reads what `write_state` wrote (a town from before save version 49 has just its depot); `None` on a kind or
    /// stage that doesn't exist, or deliveries that don't match the open order.
    pub fn read_state(r: &mut ByteReader) -> Option<Settlements> {
        let mut towns = Vec::new();
        for _ in 0..r.count()? {
            let (pos, kind) = (r.ivec3()?, r.u8()?);
            if kind as usize >= KINDS.len() {
                return None;
            }
            let mut town = Town::new(pos, kind);
            if r.version >= 49 {
                town.built = r.u8()?;
                town.delivered = (0..r.count()?).map(|_| r.u32()).collect::<Option<_>>()?;
                (town.stalled, town.due) = (r.bool()?, r.u64()?);
                if town.built as usize > STAGES || town.delivered.len() != town.order().len() {
                    return None;
                }
            }
            if r.version >= 50 {
                town.planned = r.u8()?.min(town.built);
                town.plots = (0..r.count()?).map(|_| Plot::read(r)).collect::<Option<_>>()?;
                town.cursor = r.u16()?;
                town.drones = (0..r.count()?).map(|_| TownDrone::read(r)).collect::<Option<_>>()?;
            }
            towns.push(town);
        }
        Some(Settlements { towns })
    }
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
