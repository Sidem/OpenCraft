//! The AI survey (tech AI Survey; presentation, queries only): guesses where ore lies from the stained soil
//! on the explored map (`minimap/atlas.rs`) within [`RADIUS`] blocks of the player, and the maps draw the
//! guesses as faint rings (`minimap/marks.rs`, shape `MARK_GUESS`). Never core state: the state hash and the
//! save do not know it.
//!
//! A stain (`RUSTY_SOIL..=PALE_SAND`) stands for the ore below it (`ore_of`: rusty iron, dark coal, green copper,
//! pale limestone, which quartz shares). Stained columns are counted per [`CELL`]-block square, squares with at
//! least [`MIN_STAINS`] columns join their neighbours of the same ore into one guess at the cluster's mean, rounded
//! to [`FUZZ`] blocks (a hint, not the answer), strong when much ground is stained. A guess near a prospected
//! deposit of that ore is dropped: it has been found. Ores that leave no stain (bauxite, oil sand, uranium) are
//! never guessed.
//!
//! To guess from something else too: add its columns to `stained` in [`survey`].

use rustc_hash::FxHashMap;

use crate::block::{BlockId, COAL_ORE, COPPER_ORE, DARK_SAND, DARK_SOIL, GREEN_SAND, GREEN_SOIL, IRON_ORE};
use crate::block::{LIMESTONE, PALE_SAND, PALE_SOIL, RUSTY_SAND, RUSTY_SOIL};
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::sort_small_by_key;
use crate::minimap::{Atlas, Known};
use crate::research::{Feature, Unlock};
use crate::Game;

/// How far from the player the survey looks, in blocks.
pub const RADIUS: i32 = 256;
/// Side of the squares stained columns are counted in.
const CELL: i32 = 16;
/// Stained columns a square needs to count (a lone stained block proves nothing).
const MIN_STAINS: u32 = 4;
/// A guess within this many blocks of a prospected deposit of the same ore is dropped.
const KNOWN_CLEAR: i32 = 32;
/// Guesses sit on a grid of this many blocks.
const FUZZ: i32 = 8;
/// The most guesses kept (the nearest).
const MAX_GUESSES: usize = 128;
/// Stained columns for strength 2 and 3 (1 below the first).
const STRONG: [u32; 2] = [24, 96];

/// One guess: where, which ore, and how much stained ground backs it (1 to 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Guess {
    pub x: i32,
    pub z: i32,
    pub ore: BlockId,
    pub strength: u8,
}

/// The ore a stained block stands for (`None` for any other block).
pub fn ore_of(stain: BlockId) -> Option<BlockId> {
    match stain {
        RUSTY_SOIL | RUSTY_SAND => Some(IRON_ORE),
        DARK_SOIL | DARK_SAND => Some(COAL_ORE),
        GREEN_SOIL | GREEN_SAND => Some(COPPER_ORE),
        PALE_SOIL | PALE_SAND => Some(LIMESTONE),
        _ => None,
    }
}

/// The survey's guesses and when they were made (the player's square and the explored map's version).
#[derive(Default)]
pub struct Survey {
    guesses: Vec<Guess>,
    at: Option<((i32, i32), u32, usize)>,
}

impl Survey {
    pub fn guesses(&self) -> &[Guess] {
        &self.guesses
    }

    /// Brings the guesses up to date for a player at `centre`; clears them while the tech is not known (`on`).
    /// Does nothing unless the player changed square, the explored map changed or a deposit was prospected.
    pub fn update(&mut self, atlas: &Atlas, known: &Known, centre: (i32, i32), on: bool) {
        if !on {
            *self = Survey::default();
            return;
        }
        let stamp = ((centre.0 >> 4, centre.1 >> 4), atlas.changes, known.centres().count());
        if self.at == Some(stamp) {
            return;
        }
        self.at = Some(stamp);
        let known: Vec<_> = known.centres().collect();
        self.guesses = survey(atlas, centre, &known);
    }
}

/// The guesses for a player at `centre`, nearest first; `known` are the prospected deposits (x, z, ore).
pub fn survey(atlas: &Atlas, centre: (i32, i32), known: &[(i32, i32, BlockId)]) -> Vec<Guess> {
    // Stained columns per square: (count, sum of x, sum of z), by (ore, square x, square z).
    let mut cells: FxHashMap<(BlockId, i32, i32), (u32, i64, i64)> = FxHashMap::default();
    let lo = ((centre.0 - RADIUS) >> CHUNK_SHIFT, (centre.1 - RADIUS) >> CHUNK_SHIFT);
    let hi = ((centre.0 + RADIUS) >> CHUNK_SHIFT, (centre.1 + RADIUS) >> CHUNK_SHIFT);
    atlas.each_tile_in(lo, hi, |cx, cz, tile| {
        for (i, &c) in tile.columns.iter().enumerate() {
            let Some(ore) = ore_of((c & 0xff) as BlockId) else { continue };
            let x = (cx << CHUNK_SHIFT) + (i as i32 & (CHUNK_SIZE - 1));
            let z = (cz << CHUNK_SHIFT) + (i as i32 >> CHUNK_SHIFT);
            if (x - centre.0).pow(2) + (z - centre.1).pow(2) > RADIUS * RADIUS {
                continue;
            }
            let cell = cells.entry((ore, x.div_euclid(CELL), z.div_euclid(CELL))).or_default();
            *cell = (cell.0 + 1, cell.1 + x as i64, cell.2 + z as i64);
        }
    });
    cells.retain(|_, c| c.0 >= MIN_STAINS);

    // Join neighbouring squares of one ore into one guess.
    let mut out = Vec::new();
    while let Some(&seed) = cells.keys().next() {
        let (ore, mut total) = (seed.0, cells.remove(&seed).unwrap_or_default());
        let mut open = vec![seed];
        while let Some((_, qx, qz)) = open.pop() {
            for key in (-1..=1).flat_map(|dx| (-1..=1).map(move |dz| (ore, qx + dx, qz + dz))) {
                if let Some(c) = cells.remove(&key) {
                    total = (total.0 + c.0, total.1 + c.1, total.2 + c.2);
                    open.push(key);
                }
            }
        }
        let (n, mean) = (total.0, |sum: i64| (sum / total.0 as i64) as i32);
        let snap = |v: i32| v.div_euclid(FUZZ) * FUZZ + FUZZ / 2;
        let (x, z) = (snap(mean(total.1)), snap(mean(total.2)));
        if known.iter().any(|&(kx, kz, k)| k == ore && (kx - x).abs().max((kz - z).abs()) <= KNOWN_CLEAR) {
            continue;
        }
        let strength = 1 + STRONG.iter().filter(|&&s| n >= s).count() as u8;
        out.push(Guess { x, z, ore, strength });
    }
    sort_small_by_key(&mut out, |g| ((g.x - centre.0).pow(2) + (g.z - centre.1).pow(2), g.x, g.z));
    out.truncate(MAX_GUESSES);
    out
}
impl Game {
    /// Updates the survey for the local player (the maps call it before they draw).
    pub(crate) fn refresh_survey(&mut self) {
        let on = self.sim.factory.research.has(Unlock::Feature(Feature::AiSurvey));
        let p = self.body().pos.floor();
        let m = &mut self.minimap;
        m.survey.update(&m.atlas, &m.known, (p.x, p.z), on);
    }
}

#[cfg(test)]
mod tests;
