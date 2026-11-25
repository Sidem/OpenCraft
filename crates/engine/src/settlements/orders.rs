//! What towns ask for, as data (step 13.2; numbers settled with the user on 2026-10-16, `docs/DECISIONS.md`).
//!
//! - **Orders** ([`ORDERS`]): one per stage, the same for every kind; delivering all of it builds the next stage. They
//!   double from about 1,000 items: basics first, then concrete, steel and lamps, later plastic, circuits and processors.
//! - **Upkeep** ([`UPKEEP_PER_MINUTE`] and each kind's `UPKEEP` row): what a built town uses up every minute, split over
//!   its kind's goods by weight. A camp uses the first two goods; each stage after adds the next one.
//! - **Late goods** ([`LATE`], TOWN-LATE): from the village on, each kind also uses up one late good of its trade,
//!   [`LATE_PER_MINUTE`] on top of the rest, once the players' machines have made one (`Settlements::made`), so no
//!   village stalls on a good its players cannot make yet. A standing order: it pauses the town, never takes back.
//! - **Wares** ([`WARES`], MAT-CLAY): from the hamlet on, a kind also uses up what it is built from, [`WARES_PER_MINUTE`]
//!   split by weight, under the same rule: only goods the players' machines have made (river towns: brick and tiles).
//!
//! To tune: the tables below. Orders' shape is saved (a town keeps one delivered count per line of its open order), so
//! changing a line's count is safe, but adding or removing order lines needs a save version. Upkeep is not saved.

use crate::block::{CLAY_BRICKS, COAL_ORE, CONCRETE, GLASS, LAMP, PLANKS, STONE_BRICKS, TERRACOTTA};
use crate::item::{ItemId, CIRCUIT, GEAR, MOTOR, PLASTIC, PROCESSOR, QUICKLIME, STEEL_BEAM, STEEL_PLATE};
use crate::item::{AI_ACCELERATOR, ALUMINIUM_PLATE, BATTERY, DRONE};

/// How many stages a town can build (camp, hamlet, village, town, city).
pub const STAGES: usize = 5;

/// What each stage is called, by the number of stages built (0: only the depot stands).
pub const STAGE_NAMES: [&str; STAGES + 1] = ["depot", "camp", "hamlet", "village", "town", "city"];

/// Items a minute a town uses up once it has built this many stages (index: stages built minus 1).
pub const UPKEEP_PER_MINUTE: [u32; STAGES] = [20, 60, 150, 300, 600];

/// A line of an order or of an upkeep: the item and how many (for upkeep, its weight).
pub type Line = (ItemId, u32);

const fn b(id: crate::block::BlockId) -> ItemId {
    ItemId::block(id)
}

/// The order that builds each stage, by the number of stages built.
pub const ORDERS: [&[Line]; STAGES] = [
    &[(b(PLANKS), 400), (b(STONE_BRICKS), 400), (b(GLASS), 200)],
    &[(b(PLANKS), 600), (b(CONCRETE), 800), (b(GLASS), 400), (b(LAMP), 200)],
    &[(b(CONCRETE), 2000), (STEEL_BEAM, 1000), (b(GLASS), 800), (b(LAMP), 200)],
    &[(b(CONCRETE), 4000), (STEEL_PLATE, 2000), (PLASTIC, 1000), (CIRCUIT, 1000)],
    &[(b(CONCRETE), 8000), (STEEL_BEAM, 4000), (PLASTIC, 2000), (CIRCUIT, 1000), (PROCESSOR, 1000)],
];

/// Upkeep goods and their weights, per town kind (`KINDS` order): the two basics weigh most, the better goods less.
pub const UPKEEP: [[Line; STAGES + 1]; 4] = [
    // Harbour.
    [(b(PLANKS), 4), (b(GLASS), 4), (b(LAMP), 2), (b(CONCRETE), 2), (PLASTIC, 1), (CIRCUIT, 1)],
    // River town.
    [(b(STONE_BRICKS), 4), (b(LAMP), 4), (b(GLASS), 2), (STEEL_PLATE, 2), (MOTOR, 1), (CIRCUIT, 1)],
    // Highland workshop.
    [(GEAR, 4), (STEEL_PLATE, 4), (MOTOR, 2), (STEEL_BEAM, 2), (CIRCUIT, 1), (PROCESSOR, 1)],
    // Basalt forge.
    [(b(COAL_ORE), 4), (QUICKLIME, 4), (b(STONE_BRICKS), 2), (STEEL_PLATE, 2), (MOTOR, 1), (CIRCUIT, 1)],
];

/// Each kind's late good (`KINDS` order): harbours store power in batteries, river towns relay drones, highland
/// workshops run AI accelerators, basalt forges roll aluminium.
pub const LATE: [ItemId; 4] = [BATTERY, DRONE, AI_ACCELERATOR, ALUMINIUM_PLATE];

/// Late goods a minute, by stages built minus 1: none before the village.
pub const LATE_PER_MINUTE: [u32; STAGES] = [0, 0, 2, 4, 8];

/// Each kind's wares (`KINDS` order) and their weights: what its houses are built from. River towns build in clay
/// brick under terracotta roofs; the other kinds' come with their materials (`docs/ROADMAP_WORLD.md`).
pub const WARES: [&[Line]; 4] = [&[], &[(b(CLAY_BRICKS), 2), (b(TERRACOTTA), 1)], &[], &[]];

/// Wares a minute in all, by stages built minus 1: none before the hamlet.
pub const WARES_PER_MINUTE: [u32; STAGES] = [0, 12, 24, 36, 48];

/// What a town of `kind` with `built` stages uses up a minute: its goods and how many of each (none before the camp),
/// then its wares and its late good, each once `made` says the players' machines have made one.
pub fn upkeep(kind: u8, built: u8, made: impl Fn(ItemId) -> bool) -> Vec<Line> {
    let i = (built as usize).wrapping_sub(1);
    let Some(&total) = UPKEEP_PER_MINUTE.get(i) else { return Vec::new() };
    let goods = &UPKEEP[kind as usize][..built as usize + 1];
    let weights: u32 = goods.iter().map(|g| g.1).sum();
    let mut lines: Vec<Line> = goods.iter().map(|&(item, w)| (item, (total * w / weights).max(1))).collect();
    let wares: Vec<Line> = WARES[kind as usize].iter().copied().filter(|l| made(l.0)).collect();
    let weights: u32 = wares.iter().map(|w| w.1).sum();
    if WARES_PER_MINUTE[i] > 0 && weights > 0 {
        lines.extend(wares.iter().map(|&(item, w)| (item, (WARES_PER_MINUTE[i] * w / weights).max(1))));
    }
    let late = LATE[kind as usize];
    if LATE_PER_MINUTE[i] > 0 && made(late) {
        lines.push((late, LATE_PER_MINUTE[i]));
    }
    lines
}
