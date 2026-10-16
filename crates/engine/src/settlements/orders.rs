//! What towns ask for, as data (step 13.2; numbers settled with the user on 2026-10-16, `docs/DEV_PLAN.md` section 1).
//!
//! - **Orders** ([`ORDERS`]): one per stage, the same for every kind; delivering all of it builds the next stage. They
//!   double from about 1,000 items: basics first, then concrete, steel and lamps, later plastic, circuits and processors.
//! - **Upkeep** ([`UPKEEP_PER_MINUTE`] and each kind's `UPKEEP` row): what a built town uses up every minute, split over
//!   its kind's goods by weight. A camp uses the first two goods; each stage after adds the next one.
//!
//! To tune: the tables below. Their shape is saved (a town keeps one delivered count per line of its open order), so
//! changing a line's count is safe, but adding or removing lines needs a save version.

use crate::block::{COAL_ORE, CONCRETE, GLASS, LAMP, PLANKS, STONE_BRICKS};
use crate::item::{ItemId, CIRCUIT, GEAR, MOTOR, PLASTIC, PROCESSOR, QUICKLIME, STEEL_BEAM, STEEL_PLATE};

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

/// What a town of `kind` with `built` stages uses up a minute: its goods and how many of each (none before the camp).
pub fn upkeep(kind: u8, built: u8) -> Vec<Line> {
    let Some(&total) = UPKEEP_PER_MINUTE.get((built as usize).wrapping_sub(1)) else { return Vec::new() };
    let goods = &UPKEEP[kind as usize][..built as usize + 1];
    let weights: u32 = goods.iter().map(|g| g.1).sum();
    goods.iter().map(|&(item, w)| (item, (total * w / weights).max(1))).collect()
}
