//! Prospecting readings for the host (`web/src/ui/prospect.ts`): the latest scan or core sample as
//! flat records (their layout is in `prospect.rs`), where it was taken, and deposit names.

use wasm_bindgen::prelude::*;

use crate::block;
use crate::deposits::Tier;
use crate::prospect::{is_advanced, scan_range_of, DRILL_FIELDS, READING_DRILL, READING_SCAN, SCANNERS, SCAN_FIELDS};
use crate::tools::{self, ToolKind};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Changes whenever a new reading arrives.
    pub fn prospect_seq(&self) -> u32 {
        self.prospect.seq
    }

    /// 1 scan, 2 core sample (0 before the first).
    pub fn prospect_kind(&self) -> u8 {
        self.prospect.kind
    }

    /// The latest reading's records, `prospect_fields` numbers each. A scan taken with the ore filter on
    /// keeps only that ore's deposits.
    pub fn prospect_records(&self) -> Vec<i32> {
        let p = &self.prospect;
        match p.filter.filter(|_| p.kind == READING_SCAN && self.scanner_advanced()) {
            Some(ore) => {
                p.records.chunks_exact(SCAN_FIELDS).filter(|r| r[0] == ore as i32).flatten().copied().collect()
            }
            None => p.records.clone(),
        }
    }

    /// Whether the scanner in hand has the advanced readout (the ore filter on the R key, reserves, tracking).
    pub fn scanner_advanced(&self) -> bool {
        is_advanced(self.inventory().selected_stack().item)
    }

    /// The ore the advanced scanner's list is limited to (0 for every ore).
    pub fn scan_filter(&self) -> u8 {
        self.prospect.filter.filter(|_| self.scanner_advanced()).unwrap_or(0)
    }

    pub fn prospect_fields(&self) -> u32 {
        (if self.prospect.kind == READING_DRILL { DRILL_FIELDS } else { SCAN_FIELDS }) as u32
    }

    /// Where the latest reading was taken: the player's feet or the drilled block.
    pub fn prospect_origin(&self) -> Vec<i32> {
        let o = self.prospect.origin;
        vec![o.x, o.y, o.z]
    }

    /// The prospecting device in the local player's hand: 0 none, 1 scanner, 2 core drill.
    pub fn held_device(&self) -> u8 {
        match tools::device(self.inventory().selected_stack().item) {
            Some(ToolKind::Scanner) => READING_SCAN,
            Some(_) => READING_DRILL,
            None => 0,
        }
    }

    /// The range shown for scanning: the latest scan's, or else the held scanner's.
    pub fn scan_range(&self) -> u32 {
        let held = scan_range_of(self.inventory().selected_stack().item);
        (if self.prospect.kind == READING_SCAN { self.prospect.range } else { held.unwrap_or(SCANNERS[0].1) }) as u32
    }

    /// The nearest ground that can hold `ore`, as seen from where the latest scan was taken: `[x, z, blocks away]`
    /// (0 blocks: the scan itself stood on it), or empty when this world has none (or no biome ores).
    pub fn ore_bearing(&self, ore: u8) -> Vec<i32> {
        self.ore_bearing_of(ore).map(|b| vec![b.x, b.z, b.distance]).unwrap_or_default()
    }

    /// An ore's name, e.g. "Iron".
    pub fn ore_name(&self, ore: u8) -> String {
        block::ore_label(ore).to_string()
    }

    /// A deposit's name from its ore and tier, e.g. "Iron vein".
    pub fn deposit_label(&self, ore: u8, tier: u8) -> String {
        format!("{} {}", block::ore_label(ore), Tier::from_u8(tier).map_or("deposit", Tier::name))
    }
}
