//! Towns for the host (queries only): the town section of a depot's panel (`web/src/ui/town.ts`), and each town's state
//! with its news line, which the HUD toasts when a town stalls or is supplied again (`web/src/ui/hud.ts`), and the
//! kinds for the world map's legend.

use wasm_bindgen::prelude::*;

use crate::math::IVec3;
use crate::settlements::{upkeep, KINDS, STAGES};
use crate::Game;

#[cfg(test)]
mod tests;

#[wasm_bindgen]
impl Game {
    /// The town depot as an item, for the icon of a town's news.
    pub fn town_depot_item(&self) -> u32 {
        u32::from(crate::item::ItemId::block(crate::block::TOWN_DEPOT).0)
    }

    /// The panel role of a town depot's store (`machine_panel`): its town's goods.
    pub fn panel_role_store(&self) -> u32 {
        crate::factory::ROLE_STORE as u32
    }

    /// The town whose depot has a cell at (x, y, z) as `[kind, stages built, stages in all, stalled, order lines, then
    /// (item, delivered, wanted) per line of the open order, upkeep lines, then (item, a minute, held) per upkeep good]`.
    /// Before its camp the upkeep is the camp's to come. Empty when no awake town's depot is there.
    pub fn town_panel(&self, x: i32, y: i32, z: i32) -> Vec<u32> {
        let Some(t) = self.town_at(IVec3::new(x, y, z)) else { return Vec::new() };
        let mut out = vec![u32::from(t.kind), u32::from(t.built), STAGES as u32, u32::from(t.stalled)];
        out.push(t.order().len() as u32);
        for (&(item, want), &got) in t.order().iter().zip(&t.delivered) {
            out.extend([u32::from(item.0), got, want]);
        }
        let held = self.sim.factory.depot_held(t.pos);
        let upkeep = upkeep(t.kind, t.built.max(1));
        out.push(upkeep.len() as u32);
        for (item, n) in upkeep {
            out.extend([u32::from(item.0), n, held.map_or(0, |b| b.count(item))]);
        }
        out
    }

    /// How many kinds of town there are (for the world map's legend).
    pub fn town_kind_count(&self) -> u32 {
        KINDS.len() as u32
    }

    /// Town kind `k`'s name ("Basalt Forge"), or empty.
    pub fn town_kind_name(&self, k: u32) -> String {
        KINDS.get(k as usize).map_or_else(String::new, |d| d.name.to_string())
    }

    /// Town kind `k`'s colour on the maps (`0xRRGGBB`), or 0.
    pub fn town_kind_colour(&self, k: u32) -> i32 {
        KINDS.get(k as usize).map_or(0, |d| d.colour)
    }

    /// "Basalt Forge, a village, 474 blocks from spawn" for the depot with a cell at (x, y, z); empty for none.
    pub fn town_heading(&self, x: i32, y: i32, z: i32) -> String {
        self.town_at(IVec3::new(x, y, z)).map_or_else(String::new, |t| self.town_heading_of(t))
    }

    /// What its perk does ("Perk: …"), or empty.
    pub fn town_perk(&self, x: i32, y: i32, z: i32) -> String {
        self.town_at(IVec3::new(x, y, z)).and_then(|t| t.perk_text()).unwrap_or_default()
    }

    /// Each town's state in founding order: 0 growing or supplied, 1 stalled, 2 dormant (its depot broken).
    pub fn town_states(&self) -> Vec<u32> {
        self.sim.settlements.towns.iter().map(|t| u32::from(t.state())).collect()
    }

    /// Town `i`'s news in its state now: "Your basalt forge has stalled: short of Coal Ore and Quicklime" or "Your basalt
    /// forge is supplied again"; empty for a dormant town or none.
    pub fn town_news(&self, i: u32) -> String {
        let Some(t) = self.sim.settlements.towns.get(i as usize).filter(|t| !t.dormant) else { return String::new() };
        let kind = KINDS[t.kind as usize].name.to_lowercase();
        if !t.stalled {
            return format!("Your {kind} is supplied again");
        }
        let short = self.short_of(t);
        let list = match short.split_last() {
            Some((last, [])) => last.to_string(),
            Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
            None => "its upkeep".to_string(),
        };
        format!("Your {kind} has stalled: short of {list}")
    }
}
