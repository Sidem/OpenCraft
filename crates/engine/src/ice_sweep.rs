//! The authority's ice sweep (FREEZE, modeled on `grass_sweep.rs`): turns exposed top-layer still water into mineable
//! `block::ICE` blocks below freezing, schedules `TimerKind::IceMelt` on ice blocks once air warms above 0 °C, and
//! triggers inventory ice melting once per second for players standing above freezing temperature.
//!
//! Invariants: never edits the core directly (queues `Action::FreezeWater`, `Action::CheckIceMelt`, and
//! `Action::MeltInventoryIce`), never touches `Sim.rng` (columns come from a tick+body hash), and co-op clients do
//! not sweep. A dry biome, a summer lake or an already-frozen winter lake queues zero actions and triggers zero
//! remeshes. To tune: the constants below.

use crate::action::Action;
use crate::block::{BlockId, AIR, FLOW_1, FLOW_7, ICE, STONE, WATER};
use crate::item::ICE_ITEM;
use crate::math::IVec3;
use crate::net::Role;
use crate::sim::{PlayerId, TimerKind, MAX_TIMERS};
use crate::weather::ice::{can_evaporate_puddle, can_melt, freezes_at};
use crate::weather::Reading;
use crate::{Game, TICK_RATE};

/// Columns within this many blocks of a body, on each axis.
const REACH: i32 = 48;
/// Cells scanned above and below a body's feet in each swept column.
const DEPTH: i32 = 24;
/// Columns sampled per body each tick: fast enough to freeze/thaw nearby lakes smoothly without stalling frames.
const COLUMNS_PER_TICK: u64 = 16;
/// Leave half of `MAX_TIMERS` free so melting ice never crowds out leaf/grass/sapling timers.
const ICE_TIMER_CAP: usize = MAX_TIMERS / 2;
/// How often (ticks) the authority triggers an inventory ice-melt check for warm players (matches `INVENTORY_MELT_STEP_SECS`).
const ICE_INV_CHECK_TICKS: u64 = TICK_RATE as u64;

impl Game {
    /// Sweeps columns near each body for top-layer freezing water and warm ice, plus once-per-second inventory melting.
    pub(crate) fn sweep_ice(&mut self) {
        if matches!(self.role, Role::Client(_)) {
            return;
        }
        let mut freeze = Vec::new();
        let mut melt = Vec::new();
        let mut inv_melt = Vec::new();
        let check_inv = self.sim.tick.is_multiple_of(ICE_INV_CHECK_TICKS);
        let span = (2 * REACH + 1) as u64;
        let mut timer_budget = ICE_TIMER_CAP.saturating_sub(self.sim.timers.len());

        for (slot, body) in self.bodies.iter().enumerate() {
            let Some(body) = body else { continue };
            let feet = body.pos.floor();
            if check_inv {
                let id = PlayerId(slot as u8);
                if let Some(core) = self.sim.player(id) {
                    let has_ice = core.inventory.count(ICE_ITEM) > 0
                        || core.inventory.cursor.item == ICE_ITEM && !core.inventory.cursor.is_empty()
                        || core.inventory.offhand.item == ICE_ITEM && !core.inventory.offhand.is_empty();
                    if has_ice {
                        let temp =
                            Reading::at(&self.sim.atmosphere, self.sim.world.generator(), feet.x, feet.y, feet.z).temp;
                        if can_melt(temp) {
                            inv_melt.push((id, feet));
                        }
                    }
                }
            }
            for step in 0..COLUMNS_PER_TICK {
                let h = mix(self.sim.tick.wrapping_mul(COLUMNS_PER_TICK).wrapping_add(step) ^ ((slot as u64) << 56));
                let (dx, dz) = ((h % span) as i32 - REACH, ((h >> 20) % span) as i32 - REACH);
                let top = feet + IVec3::new(dx, DEPTH, dz);
                if let Some(hit) = scan_column(&self.sim.world, top) {
                    let (air, gen) = (&self.sim.atmosphere, self.sim.world.generator());
                    match hit.block {
                        WATER | FLOW_7 if freezes_at(air, gen, hit.pos.x, hit.pos.y, hit.pos.z) => freeze.push(hit.pos),
                        ICE if can_melt(Reading::at(air, gen, hit.pos.x, hit.pos.y, hit.pos.z).temp)
                            && timer_budget > 0
                            && !self.sim.timers.pending(hit.pos, TimerKind::IceMelt) =>
                        {
                            melt.push(hit.pos);
                            timer_budget -= 1;
                        }
                        FLOW_1
                            if can_evaporate_puddle(
                                FLOW_1,
                                AIR,
                                hit.pos.y,
                                Reading::at(air, gen, hit.pos.x, hit.pos.y, hit.pos.z).temp,
                            ) && timer_budget > 0
                                && !self.sim.timers.pending(hit.pos, TimerKind::PuddleEvap) =>
                        {
                            melt.push(hit.pos);
                            timer_budget -= 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        for pos in freeze {
            self.act(Action::FreezeWater { pos });
        }
        for pos in melt {
            self.act(Action::CheckIceMelt { pos });
        }
        for (id, pos) in inv_melt {
            self.act_as(id, Action::MeltInventoryIce { pos });
        }
    }
}

struct ColumnHit {
    pos: IVec3,
    block: BlockId,
}

/// First exposed surface `WATER`/`FLOW_7`/`FLOW_1` (with `AIR` above) or `ICE` in the column from `top` down
/// `2 * DEPTH + 1` cells.
fn scan_column(world: &crate::world::World, top: IVec3) -> Option<ColumnHit> {
    let mut above = world.get_block(top + IVec3::new(0, 1, 0)).unwrap_or(STONE);
    for d in 0..=2 * DEPTH {
        let p = top - IVec3::new(0, d, 0);
        let b = world.get_block(p).unwrap_or(STONE);
        if b == ICE || (above == AIR && matches!(b, WATER | FLOW_7 | FLOW_1)) {
            return Some(ColumnHit { pos: p, block: b });
        }
        above = b;
    }
    None
}

/// A well-mixed 64-bit hash (splitmix64's finaliser).
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}
