//! Ice (FREEZE): lakes freeze in a cold spell, flowing water later, the sea only in the coldest places. Ice is a pure
//! function of the place's climate and the date, like the season: no water cell is ever edited, nothing is saved, and
//! co-op peers agree for free. A place's frost is its freezing degree-days: every day whose mean temperature
//! (`Climate::year_mean` plus the season's `year_wave`) is below 0 °C adds its cold, every warmer day melts
//! `MELT_NUM / MELT_DEN` as fast, counted from mid-summer on top of the frost a whole year settles to (so it never
//! jumps, at midnight or at the turn of the ice year). Ice bears a body once the frost passes the water's
//! `BEARS` (a small still lake soonest, the sea last), and thickens on to `FULL` times that.
//!
//! Places are binned by their year's mean temperature into a `cold` index (whole degrees, `NEVER` for places too warm
//! ever to freeze), so one small table per tick ([`IceTable::at`]) answers every place: bodies and loose items stand on
//! it (`authority.rs`), and the water shader draws it from the same table (`api/weather.rs` `ice_table`), each water
//! vertex carrying its cold index and kind (`mesher/water.rs`). The climate's warmth is read on a `LATTICE` and
//! interpolated, so the mesher and the walking agree to the bin.
//!
//! Winter is mild (user, 2026-11-27): ice is to walk on; pumps and water wheels on a frozen lake keep working.
//! To tune: `BEARS`, `FULL`, `MELT_NUM` / `MELT_DEN`; the temperatures are `climate.rs`'s.

use std::sync::OnceLock;

use super::climate::{year_wave_by_clock, Climate};
use crate::block::{flow_level, BlockId, LIQUID};
use crate::daytime::{DAY_TICKS, START, YEAR_DAYS};
use crate::math::{IVec3, Vec3};
use crate::physics::Aabb;
use crate::shape::CellBoxes;
use crate::world::World;
use crate::worldgen::{WorldGen, SEA_LEVEL};

/// Cold indices: whole degrees of the year's mean temperature from `COLDEST`, the last (`NEVER`) for every place warm
/// enough that no day's mean falls below 0 °C.
pub const COLD_LEVELS: usize = 64;
pub const NEVER: u8 = COLD_LEVELS as u8 - 1;
// A vertex packs the cold index in 6 bits beside the flowing bit (`vertex_code`).
const _: () = assert!(COLD_LEVELS <= 64);
/// The kinds of water ice knows (the table's rows, the shader's order).
pub const KINDS: usize = 3;
/// Ice levels: 0 open water, `BEARS_LEVEL` and up bears a body, 255 the thickest.
pub const BEARS_LEVEL: u8 = 64;
/// The year's mean temperature (whole degrees) of cold index 0.
const COLDEST: i32 = -51;
/// Freezing degree-days (tenths of a degree × days) after which each kind of water bears a body: a still lake in a few
/// cold days, flowing water later, the sea only in the coldest climates (shore ice).
const BEARS: [i32; KINDS] = [30, 120, 1500];
/// Ice goes on thickening to this many times `BEARS` (level 255).
const FULL: i32 = 4;
/// The most frost there is: the sea's thickest ice.
const CAP: i64 = (BEARS[Water::Sea as usize] * FULL) as i64;
/// A warm day melts this much faster than a cold day of the same size freezes.
const MELT_NUM: i32 = 3;
const MELT_DEN: i32 = 1;
/// The ice year starts on this day of the calendar (mid-summer: the year's wave crests at day 13.5) and repeats every
/// year; its first day starts from the frost a whole year settles to (`settled`, a few rounds of `SETTLE_YEARS`).
const ICE_YEAR_START: u64 = YEAR_DAYS * 3 / 8;
const SETTLE_YEARS: usize = 8;
/// The spacing (blocks) at which the climate's warmth is read and between which it is interpolated.
pub const LATTICE: i32 = 32;
/// Ice is a slab this thick (sixteenths) under the water's surface; it bears what is no more than `ABOVE` below its top.
const SLAB: i32 = 2;
const ABOVE: f64 = 0.05;

/// The kind of water at a surface, as ice sees it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Water {
    /// Still water above the sea: lakes, ponds, a river's pools.
    Lake = 0,
    /// Flowing water (`FLOW_1..=FLOW_7`).
    Flowing = 1,
    /// Still water at or below `SEA_LEVEL` (`sim/water.rs` keeps it endless).
    Sea = 2,
}

impl Water {
    /// The kind of water block `b` at height `y`, if it is water.
    pub fn of(b: BlockId, y: i32) -> Option<Water> {
        if !LIQUID[b as usize] {
            None
        } else if flow_level(b).is_some() {
            Some(Water::Flowing)
        } else if y <= SEA_LEVEL {
            Some(Water::Sea)
        } else {
            Some(Water::Lake)
        }
    }
}

/// The cold index of a place whose year's mean temperature is `year_mean` (tenths of a degree).
pub fn cold_index(year_mean: i32) -> u8 {
    (year_mean.div_euclid(10) - COLDEST).clamp(0, i32::from(NEVER)) as u8
}

/// The cold index of the water surface in cell (x, y, z).
pub fn cold_at(gen: &WorldGen, x: i32, y: i32, z: i32) -> u8 {
    cold_of(warmth_at(gen, x, z), y)
}

/// The cold index at height `y` where the climate's warmth is `warmth`.
pub fn cold_of(warmth: i32, y: i32) -> u8 {
    cold_index(Climate::of(warmth, 0).year_mean(y))
}

/// The climate's warmth (thousandths) at column (x, z), interpolated between the `LATTICE` points around it.
pub fn warmth_at(gen: &WorldGen, x: i32, z: i32) -> i32 {
    let (gx, gz) = (x.div_euclid(LATTICE) * LATTICE, z.div_euclid(LATTICE) * LATTICE);
    warmth_between(lattice_corners(gen, gx, gz), x - gx, z - gz)
}

/// The warmth at the four lattice points round the lattice cell whose lowest corner is (gx, gz): (0,0), (1,0), (0,1),
/// (1,1) cells on. A chunk is one lattice cell, so the mesher reads these once a chunk (`mesher/cover.rs` `Cover`).
pub fn lattice_corners(gen: &WorldGen, gx: i32, gz: i32) -> [i32; 4] {
    [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(dx, dz)| Climate::at(gen, gx + dx * LATTICE, gz + dz * LATTICE).warmth)
}

/// The warmth `fx`, `fz` blocks (`0..LATTICE`) into a lattice cell with `corners`.
pub fn warmth_between(corners: [i32; 4], fx: i32, fz: i32) -> i32 {
    let [a, b, c, d] = corners.map(i64::from);
    let (fx, fz, l) = (i64::from(fx), i64::from(fz), i64::from(LATTICE));
    let near = a * (l - fx) + b * fx;
    let far = c * (l - fx) + d * fx;
    ((near * (l - fz) + far * fz) / (l * l)) as i32
}

/// What a water surface vertex carries for the shader (7 bits): its cold index, and 64 if the water flows. `NEVER`
/// for every face but a surface's top, and wherever the mesher has no climate.
pub fn vertex_code(b: BlockId, y: i32, cold: Option<u8>) -> u32 {
    match (cold, Water::of(b, y)) {
        (Some(cold), Some(kind)) => u32::from(cold) | u32::from(kind == Water::Flowing) << 6,
        _ => u32::from(NEVER),
    }
}

/// Ice at every kind of water and cold index at one tick: the levels (0..=255) a place reads.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IceTable([u8; KINDS * COLD_LEVELS]);

impl IceTable {
    /// The table at `tick`.
    pub fn at(tick: u64) -> IceTable {
        let clock = tick + START;
        // Days of the ice year so far (it starts at mid-summer, when ice is least), and how far into today.
        let days = (clock / DAY_TICKS + YEAR_DAYS - ICE_YEAR_START) % YEAR_DAYS;
        let part = (clock % DAY_TICKS) as i64;
        let IceYear { waves, settled } = ice_year();
        let full = DAY_TICKS as i64;
        let mut levels = [0u8; KINDS * COLD_LEVELS];
        for cold in 0..NEVER {
            let mean = bin_mean(cold);
            let mut frost = settled[cold as usize];
            for w in &waves[..days as usize] {
                frost = add_day(frost, mean + w, full);
            }
            frost = add_day(frost, mean + waves[days as usize], part);
            for (kind, bears) in BEARS.iter().enumerate() {
                let level = frost * i64::from(BEARS_LEVEL) / i64::from(*bears);
                levels[kind * COLD_LEVELS + cold as usize] = level.min(255) as u8;
            }
        }
        IceTable(levels)
    }

    /// The ice on water of `kind` at cold index `cold`.
    pub fn level(&self, kind: Water, cold: u8) -> u8 {
        self.0[kind as usize * COLD_LEVELS + usize::from(cold.min(NEVER))]
    }

    /// The table's bytes for the host: `KINDS` rows of `COLD_LEVELS` levels.
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

/// The slab of ice in a loaded cell that bears something whose underside is at height `feet`, or `CellBoxes::EMPTY`:
/// for bodies and loose items (the authority's physics), which only live where chunks are loaded. Ice bears from
/// above only, so a swimmer under it is never trapped and one caught at the surface as it sets is not stuck in it.
pub fn ice_boxes(world: &World, table: &IceTable, feet: f64, x: i32, y: i32, z: i32) -> CellBoxes {
    let Some(b) = world.get_block(IVec3::new(x, y, z)) else { return CellBoxes::EMPTY };
    let Some(kind) = Water::of(b, y) else { return CellBoxes::EMPTY };
    if world.get_block(IVec3::new(x, y + 1, z)).is_some_and(|a| LIQUID[a as usize]) {
        return CellBoxes::EMPTY;
    }
    if table.level(kind, cold_at(world.generator(), x, y, z)) < BEARS_LEVEL {
        return CellBoxes::EMPTY;
    }
    let top = flow_level(b).map_or(15, |l| 2 * i32::from(l) - 1);
    if feet < f64::from(y) + f64::from(top) / 16.0 - ABOVE {
        return CellBoxes::EMPTY;
    }
    let base = Vec3::new(f64::from(x), f64::from(y), f64::from(z));
    let mut out = CellBoxes::EMPTY;
    out.boxes[0] = Aabb {
        min: base + Vec3::new(0.0, f64::from((top - SLAB).max(0)) / 16.0, 0.0),
        max: base + Vec3::new(1.0, f64::from(top) / 16.0, 1.0),
    };
    out.len = 1;
    out
}

/// What every tick's table shares: the season's wave on each day of the ice year (at noon) and each cold index's
/// frost at mid-summer. Worked out once.
struct IceYear {
    waves: [i32; DAYS],
    settled: [i64; NEVER as usize],
}

const DAYS: usize = YEAR_DAYS as usize;

fn ice_year() -> &'static IceYear {
    static YEAR: OnceLock<IceYear> = OnceLock::new();
    YEAR.get_or_init(|| {
        let year = YEAR_DAYS * DAY_TICKS;
        let waves = std::array::from_fn(|d| {
            year_wave_by_clock(((ICE_YEAR_START + d as u64) * DAY_TICKS + DAY_TICKS / 2) % year)
        });
        let settled = std::array::from_fn(|cold| settled(bin_mean(cold as u8), &waves));
        IceYear { waves, settled }
    })
}

/// The year's mean temperature (tenths) in the middle of cold index `cold`'s degree.
fn bin_mean(cold: u8) -> i32 {
    (i32::from(cold) + COLDEST) * 10 + 5
}

/// The frost a mid-summer carries over at a place whose year's mean is `mean`: the steady state, which a whole ice year
/// (`waves`, a day each) brings back to itself. The yearly map only rises; between the clamps (no frost, `CAP`) it adds
/// the same each year, so a run that touches neither jumps straight to where one is touched.
fn settled(mean: i32, waves: &[i32]) -> i64 {
    let mut frost = 0i64;
    for _ in 0..SETTLE_YEARS {
        let (mut f, mut lo, mut hi, mut clamped) = (frost, frost, frost, false);
        for w in waves {
            let next = add_day(f, mean + w, DAY_TICKS as i64);
            clamped |= next == 0 || next == CAP;
            f = next;
            (lo, hi) = (lo.min(f), hi.max(f));
        }
        let gain = f - frost;
        if gain == 0 {
            return frost;
        }
        frost = match (clamped, gain > 0) {
            (true, _) => f,
            (false, true) => frost + gain * ((CAP - hi + gain - 1) / gain).max(1),
            (false, false) => frost + gain * ((lo - gain - 1) / -gain).max(1),
        }
        .clamp(0, CAP);
    }
    frost
}

/// One day's (or `part` of a day's, in ticks) mean temperature `mean` (tenths) added to the frost: cold adds, warmth
/// melts faster; never below none nor past `CAP`.
fn add_day(frost: i64, mean: i32, part: i64) -> i64 {
    let day = DAY_TICKS as i64;
    let change = if mean < 0 {
        -i64::from(mean) * part / day
    } else {
        -i64::from(mean) * part * i64::from(MELT_NUM) / (day * i64::from(MELT_DEN))
    };
    (frost + change).clamp(0, CAP)
}

#[cfg(test)]
mod tests;
