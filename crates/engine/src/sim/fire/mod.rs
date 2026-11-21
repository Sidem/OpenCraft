//! Fire simulation (FIRE1 core state, FIRE2 flamethrower rays): burning grown trees (`Blaze`, `blaze.rs`) and
//! localized weed-torch ground scorches (`Front`, `front.rs`).
//!
//! Six rules keep fire cheap and deterministic (docs/ROADMAP.md "Fire"):
//! 1. The unit is a tree (`Blaze`) or localized scorch cell (`Front`), never a block automaton. A burning tree touches
//!    its blocks at most twice (crown leaves, then trunk logs), through `set_block_anywhere_later`.
//! 2. Event-driven: only burning records exist, sorted by due tick; a world with no fire does zero fire work.
//! 3. Flat load: each blaze steps every `FIRE_STEP` ticks phased by `hash3(foot)`, capped at `MAX_FIRE_UPDATES`
//!    steps and `MAX_TREE_CLEARS` tree block-clearances per tick; record counts cap at `MAX_BLAZES` / `MAX_FRONT`.
//! 4. Bounded spread: every `SPREAD_STEP`, a blaze samples `SPREAD_SAMPLES` points via `hash3(foot, tick)` (never
//!    `Sim.rng`). Child blazes start with `heat - 1`, and at 0 nothing spreads.
//! 5. Weed-torch ground clearing: ground plants burned by the flamethrower clear via `world/cover_clear.rs` bits and
//!    glow briefly without ever igniting neighbouring ground plants.
//! 6. Peaceful: fire burns only what grows (grown trees via `felling::trunk`, saplings, ground cover); never a placed
//!    block (`LOG`, planks, walls), machine, house or player body.
//!
//! To tune: the constants below. Saved at `SAVE_VERSION >= 82`.

pub mod blaze;
pub mod front;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub use blaze::charcoal_for;
pub use blaze::{Blaze, Stage};
pub use front::Front;

use super::{PlayerId, Sim, SimEvent};
use crate::block::{is_leaves, is_sapling, is_tree_wood, AIR, LIQUID};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Inventory;
use crate::item::{ItemId, DIESEL_BARREL, EMPTY_BARREL, FLAMETHROWER, NAPHTHA_BARREL};
use crate::math::{IVec3, Vec3};
use crate::mesher::bears_cover;
use crate::raycast::raycast;

/// Ticks between blaze steps (0.25 s at 60 Hz).
pub const FIRE_STEP: u64 = 15;
/// Ticks before a burning tree's crown leaves burn off (12.0 s).
pub const CROWN_TICKS: u64 = 720;
/// Ticks a charred trunk smoulders after losing its crown before collapsing to charcoal (10.0 s).
pub const TRUNK_TICKS: u64 = 600;
/// Ticks between spread samples on a burning tree (1.5 s).
pub const SPREAD_STEP: u64 = 90;
/// Points sampled around a burning tree per spread step.
pub const SPREAD_SAMPLES: u32 = 4;
/// Horizontal radius (blocks) a burning tree samples for spread.
pub const SPREAD_RADIUS: i32 = 9;
/// Default heat when a tree is lit directly; each hop subtracts 1, stopping at 0.
pub const MAX_HEAT: u8 = 4;
/// Record caps so fire work stays bounded even in dense forests.
pub const MAX_BLAZES: usize = 128;
pub const MAX_FRONT: usize = 256;
/// Max fire records (blazes + fronts) stepped in a single tick; excess waits for the next tick.
pub const MAX_FIRE_UPDATES: usize = 16;
/// Max heavy tree block-clear transitions (crown or trunk removal) per tick.
pub const MAX_TREE_CLEARS: usize = 1;
/// Ticks a weed-torched ground patch smoulders with embers before going out (1.2 s; never spreads).
pub const SCORCH_TICKS: u64 = 72;
/// Rays cast per flamethrower tick and their reach in blocks.
pub const FLAME_RAYS: usize = 5;
pub const FLAME_RANGE: f64 = 9.0;
/// Liquid fuels from the refinery or cracker accepted by the flamethrower (never coal or charcoal).
pub const FLAME_FUELS: [ItemId; 2] = [NAPHTHA_BARREL, DIESEL_BARREL];
/// Flamethrower fuel ticks obtained from one barrel of naphtha or diesel (at 0.20 s/tick = 30.0 s per barrel).
pub const FLAME_TICKS_PER_BARREL: u8 = 150;
/// Cone half-spread (radians) of the outer flamethrower rays.
const FLAME_SPREAD: f64 = 0.09;

/// Total liquid fuel barrels (naphtha + diesel) in `inv` usable by the flamethrower.
pub fn flame_fuel_barrels(inv: &Inventory) -> u32 {
    FLAME_FUELS.iter().map(|&item| inv.count(item)).sum()
}

/// Radius (blocks) of the flamethrower's flame cone at distance `dist`: tight near the nozzle (`0.35`) and growing
/// smoothly with distance to a ~4-block diameter circle (`radius = 2.15`) across the roaring orange flame head (~6+ blocks).
pub fn flame_cone_radius(dist: f64) -> f64 {
    let frac = (dist / 6.0).clamp(0.0, 1.0);
    0.35 + 1.80 * frac
}

/// All active fires in the world (core state, saved since version 82).
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Fire {
    /// Burning grown trees, sorted by `(due, foot.x, foot.y, foot.z)`.
    pub(crate) blazes: Vec<Blaze>,
    /// Short-lived localized ground embers/scorches, sorted by `(due, pos.x, pos.y, pos.z)`.
    pub(crate) fronts: Vec<Front>,
    /// Fuel ticks left in each player's flamethrower canister (non-zero entries only, sorted by player id).
    pub(crate) tanks: Vec<(u8, u8)>,
    /// True while a blaze is popped and may re-insert itself, so `MAX_BLAZES` holds strictly; not saved.
    pub(crate) stepping_blaze: bool,
}

impl Fire {
    pub fn is_empty(&self) -> bool {
        self.blazes.is_empty() && self.fronts.is_empty()
    }

    pub fn blazes(&self) -> &[Blaze] {
        &self.blazes
    }

    pub fn fronts(&self) -> &[Front] {
        &self.fronts
    }

    /// Fuel ticks left in `player`'s flamethrower canister from the last liquid-fuel barrel.
    pub fn tank(&self, player: PlayerId) -> u8 {
        self.tanks.iter().find(|t| t.0 == player.0).map_or(0, |t| t.1)
    }

    pub(crate) fn set_tank(&mut self, player: PlayerId, shots: u8) {
        if let Some(i) = self.tanks.iter().position(|t| t.0 == player.0) {
            if shots == 0 {
                self.tanks.remove(i);
            } else {
                self.tanks[i].1 = shots;
            }
        } else if shots > 0 {
            let at = self.tanks.partition_point(|t| t.0 < player.0);
            self.tanks.insert(at, (player.0, shots));
        }
    }

    pub fn has_blaze(&self, foot: IVec3) -> bool {
        self.blazes.iter().any(|b| b.foot == foot)
    }

    pub fn has_front(&self, x: i32, z: i32) -> bool {
        self.fronts.iter().any(|f| f.pos.x == x && f.pos.z == z)
    }

    pub(crate) fn insert_blaze(&mut self, b: Blaze) {
        let key = |x: &Blaze| (x.due, x.foot.x, x.foot.y, x.foot.z);
        let at = self.blazes.partition_point(|x| key(x) < key(&b));
        self.blazes.insert(at, b);
    }

    pub(crate) fn insert_front(&mut self, f: Front) {
        let key = |x: &Front| (x.due, x.pos.x, x.pos.y, x.pos.z);
        let at = self.fronts.partition_point(|x| key(x) < key(&f));
        self.fronts.insert(at, f);
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.blazes.len());
        for b in &self.blazes {
            b.write_state(w);
        }
        w.count(self.fronts.len());
        for f in &self.fronts {
            f.write_state(w);
        }
        w.count(self.tanks.len());
        for &(p, shots) in &self.tanks {
            w.u8(p);
            w.u8(shots);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Fire> {
        let mut fire = Fire::default();
        let nb = r.count()?;
        if nb > MAX_BLAZES {
            return None;
        }
        for _ in 0..nb {
            fire.insert_blaze(Blaze::read_state(r)?);
        }
        let nf = r.count()?;
        if nf > MAX_FRONT {
            return None;
        }
        for _ in 0..nf {
            fire.insert_front(Front::read_state(r)?);
        }
        let nt = r.count()?;
        if nt > 256 {
            return None;
        }
        for _ in 0..nt {
            let (p, shots) = (r.u8()?, r.u8()?);
            fire.set_tank(PlayerId(p), shots);
        }
        Some(fire)
    }
}

impl Sim {
    /// Lights whatever grows at `pos` with spread budget `heat` (`Action::Ignite`, tests and creative).
    /// Returns true if a tree, sapling or localized ground scorch was ignited.
    pub(crate) fn ignite(&mut self, pos: IVec3, heat: u8) -> bool {
        let heat = heat.min(MAX_HEAT);
        let b = self.block(pos);
        if LIQUID[b as usize] {
            return false;
        }
        if is_tree_wood(b) {
            return self.ignite_tree(pos, heat);
        }
        if is_leaves(b) {
            if let Some(log) = self.find_tree_near_leaf(pos) {
                return self.ignite_tree(log, heat);
            }
            return false;
        }
        if is_sapling(b) {
            return self.burn_sapling(pos);
        }
        if bears_cover(b) {
            return self.scorch_ground(pos);
        }
        false
    }

    /// `Action::Flame`: casts `FLAME_RAYS` rays from `origin` along `dir` when `player` holds the flamethrower and
    /// has liquid fuel (canister or naphtha/diesel barrel in pack; returns empty barrel on refill). Never accepts coal.
    pub(crate) fn flame_cast(&mut self, player: PlayerId, origin: Vec3, dir: Vec3) {
        let len = dir.length();
        if !len.is_finite() || len < 1e-4 {
            return;
        }
        let mut tank = self.fire.tank(player);
        let mut drop_empty = false;
        if tank == 0 {
            let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
            if core.inventory.selected_stack().item != FLAMETHROWER {
                return;
            }
            let drew = FLAME_FUELS.iter().any(|&fuel| core.inventory.remove(fuel, 1));
            if !drew {
                return;
            }
            if core.inventory.add(EMPTY_BARREL, 1) > 0 {
                drop_empty = true;
            }
            tank = FLAME_TICKS_PER_BARREL;
        } else {
            let Some(Some(core)) = self.players.get(player.0 as usize) else { return };
            if core.inventory.selected_stack().item != FLAMETHROWER {
                return;
            }
        }
        self.fire.set_tank(player, tank - 1);
        if drop_empty {
            self.events.push(SimEvent::Dropped {
                pos: origin,
                vel: Vec3::new(0.0, 1.0, 0.0),
                item: EMPTY_BARREL,
                count: 1,
            });
        }

        let forward = dir * (1.0 / len);
        let up_ref = if forward.y.abs() < 0.95 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
        let right = unit(cross(forward, up_ref));
        let up = cross(right, forward);
        let offsets: [(f64, f64); FLAME_RAYS] =
            [(0.0, 0.0), (FLAME_SPREAD, 0.0), (-FLAME_SPREAD, 0.0), (0.0, FLAME_SPREAD), (0.0, -FLAME_SPREAD)];

        let mut center_reach = FLAME_RANGE;
        for (idx, (rx, ry)) in offsets.into_iter().enumerate() {
            let ray_dir = unit(forward + right * rx + up * ry);
            let hit = {
                let world = &mut self.world;
                raycast(origin, ray_dir, FLAME_RANGE, |p| {
                    let b = world.block_anywhere_or_generate(p);
                    (b != AIR).then_some(b)
                })
            };
            let Some(hit) = hit else { continue };
            let dist = (hit.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - origin).length().min(FLAME_RANGE);
            if idx == 0 {
                center_reach = dist;
            }
            let cone_r = flame_cone_radius(dist);
            let b = hit.id;
            if LIQUID[b as usize] {
                continue;
            }
            if is_tree_wood(b) {
                self.ignite_tree(hit.block, MAX_HEAT);
            } else if is_leaves(b) {
                if let Some(log) = self.find_tree_near_leaf(hit.block) {
                    self.ignite_tree(log, MAX_HEAT);
                }
            } else if is_sapling(b) {
                self.burn_sapling(hit.block);
                self.scorch_disc(hit.block - IVec3::new(0, 1, 0), cone_r);
            } else if bears_cover(b) {
                // Expanding weed-torch cone: scorches every weed inside the flame's circular footprint at `dist`
                // (up to ~4-block diameter) without ever spreading past what the flame touches.
                self.scorch_disc(hit.block, cone_r);
            }
        }
        // Sweep the underside of the expanding flame cone along the center ray so skimming a meadow scorches weeds
        // engulfed by the billowing flame even when the ray itself passes just above the ground.
        let mut d = 1.5;
        while d <= center_reach {
            let cone_r = flame_cone_radius(d);
            let under = (origin + forward * d - Vec3::new(0.0, cone_r * 0.75, 0.0)).floor();
            if bears_cover(self.block(under)) {
                self.scorch_disc(under, cone_r);
            }
            d += 1.0;
        }
        self.events.push(SimEvent::Flamed { player, origin, dir: forward });
    }

    /// Advances active blazes and localized ground scorches due at `self.tick`, capped at `MAX_FIRE_UPDATES` and
    /// `MAX_TREE_CLEARS`. Does zero work when nothing is burning.
    pub(crate) fn run_fire(&mut self) {
        if self.fire.is_empty() {
            return;
        }
        let tick = self.tick;
        let mut updates = 0;
        let mut clears = 0;

        // Interleave due blazes and fronts while staying within per-tick caps.
        while updates < MAX_FIRE_UPDATES {
            let blaze_due = self.fire.blazes.first().is_some_and(|b| b.due <= tick);
            let front_due = self.fire.fronts.first().is_some_and(|f| f.due <= tick);
            if !blaze_due && !front_due {
                break;
            }
            if blaze_due {
                let b = self.fire.blazes.remove(0);
                self.fire.stepping_blaze = true;
                let deferred = self.step_blaze(b, &mut clears);
                self.fire.stepping_blaze = false;
                updates += 1;
                if deferred && !front_due {
                    break;
                }
            }
            if updates < MAX_FIRE_UPDATES && self.fire.fronts.first().is_some_and(|f| f.due <= tick) {
                let f = self.fire.fronts.remove(0);
                self.step_front(f);
                updates += 1;
            }
        }
    }

    /// Burns a single sapling block immediately via `set_block_anywhere_later`.
    pub(super) fn burn_sapling(&mut self, pos: IVec3) -> bool {
        let b = self.block(pos);
        if !is_sapling(b) || !self.world.set_block_anywhere_later(pos, AIR) {
            return false;
        }
        self.block_changed(pos, b);
        true
    }

    /// Finds a grown tree log near leaf cell `leaf` (within ±3 horizontally, up to 8 down or 2 up).
    fn find_tree_near_leaf(&mut self, leaf: IVec3) -> Option<IVec3> {
        for dy in [0, -1, -2, -3, -4, -5, -6, -7, -8, 1, 2] {
            for r in 0..=3i32 {
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx.abs().max(dz.abs()) != r {
                            continue;
                        }
                        let p = leaf + IVec3::new(dx, dy, dz);
                        if is_tree_wood(self.block(p)) {
                            return Some(p);
                        }
                    }
                }
            }
        }
        None
    }
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

fn unit(v: Vec3) -> Vec3 {
    let l = v.length();
    if l > 1e-9 {
        v * (1.0 / l)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    }
}
