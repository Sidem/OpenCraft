//! Prospecting readings for the host (`web/src/ui/prospect.ts`): the latest scan or core sample as
//! flat records (their layout is in `prospect.rs`), where it was taken, and deposit names.

use wasm_bindgen::prelude::*;

use crate::block;
use crate::deposits::Tier;
use crate::prospect::{DRILL_FIELDS, READING_DRILL, READING_SCAN, SCAN_FIELDS, SCAN_RANGE};
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

    /// The latest reading's records, `prospect_fields` numbers each.
    pub fn prospect_records(&self) -> Vec<i32> {
        self.prospect.records.clone()
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

    pub fn scan_range(&self) -> u32 {
        SCAN_RANGE as u32
    }

    /// A deposit's name from its ore and tier, e.g. "Iron vein".
    pub fn deposit_label(&self, ore: u8, tier: u8) -> String {
        format!("{} {}", block::ore_label(ore), Tier::from_u8(tier).map_or("deposit", Tier::name))
    }
}
