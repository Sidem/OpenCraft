//! The town's own drones (step 13.3a): a built town keeps a small fleet at its depot ([`fleet`]) that builds its
//! plots block by block, bottom first. The blocks are free: the delivered order paid for them. Every
//! [`LAUNCH_EVERY`] ticks a town that is not stalled and has a drone at home sends one to the next cell still to build;
//! it works [`WORK_TICKS`] there, places the block, hops to the next cell within [`HOP`] (up to [`LOAD`] cells a trip)
//! and flies home. They fly like the player's drones (`drones::approach`), but never touch ghosts or boxes.
//!
//! A cell still to build ([`pending`]) holds air, water, leaves, logs or plants and not its block yet; a `Fill` cell
//! over water stays open (stilts), a `Clear` cell only loses leaves, logs and plants. The player's blocks are never
//! replaced. `Town::cursor` skips the plots already finished.
//!
//! Invariants: core state, saved with the town (since save version 50); runs inside `Sim::step` (`step_towns`).

use crate::block::{BlockId, AIR, LIQUID};
use crate::bytes::{ByteReader, ByteWriter};
use crate::drones::{approach, target_centre, Phase, SPEED};
use crate::math::{IVec3, Vec3};
use crate::sim::Sim;
use crate::world::World;
use crate::{TICK, TICK_RATE};

use super::plans::Role;
use super::plots::takeable;

/// Drones a town keeps, by stages built: [`FLEET_BASE`] plus [`FLEET_PER_STAGE`] a stage.
pub const FLEET_BASE: usize = 2;
pub const FLEET_PER_STAGE: usize = 2;
/// Ticks between two launches from a depot.
pub const LAUNCH_EVERY: u64 = 15;
/// Ticks a drone works on one block.
pub const WORK_TICKS: u32 = TICK_RATE / 3;
/// Cells a drone builds a trip, and how far it hops between them (blocks).
pub const LOAD: u8 = 8;
pub const HOP: i32 = 6;
/// How high over its cell a drone hovers while it works.
const HOVER: f64 = 1.6;

const ROLES: [Role; 9] = [
    Role::Foundation,
    Role::Fill,
    Role::Wall,
    Role::Corner,
    Role::Window,
    Role::Roof,
    Role::Lamp,
    Role::Chimney,
    Role::Clear,
];

/// How many drones a town with `built` stages keeps.
pub fn fleet(built: u8) -> usize {
    FLEET_BASE + FLEET_PER_STAGE * built as usize
}

/// A town drone out of its depot.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TownDrone {
    pub pos: Vec3,
    /// Where it was a tick ago (derived, for drawing between ticks; not saved).
    pub prev: Vec3,
    /// The cell it builds, with its block and role.
    pub target: IVec3,
    pub block: BlockId,
    pub role: Role,
    pub phase: Phase,
    /// Ticks of work left (`Phase::Work`).
    pub timer: u32,
    /// Cells it may still build this trip, this one included.
    pub load: u8,
}

impl TownDrone {
    pub fn write(&self, w: &mut ByteWriter) {
        w.vec3(self.pos);
        w.ivec3(self.target);
        w.u8(self.block);
        w.u8(ROLES.iter().position(|&r| r == self.role).unwrap_or(0) as u8);
        w.u8(self.phase as u8);
        w.u32(self.timer);
        w.u8(self.load);
    }

    pub fn read(r: &mut ByteReader) -> Option<TownDrone> {
        let (pos, target, block) = (r.vec3()?, r.ivec3()?, r.u8()?);
        let role = *ROLES.get(r.u8()? as usize)?;
        let phase = [Phase::Out, Phase::Work, Phase::Home].get(r.u8()? as usize).copied()?;
        let (timer, load) = (r.u32()?, r.u8()?);
        let finite = pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite();
        finite.then_some(TownDrone { pos, prev: pos, target, block, role, phase, timer, load })
    }
}

/// Whether `cell` still needs `block` for `role` (see the module header).
pub fn pending(world: &mut World, cell: IVec3, block: BlockId, role: Role) -> bool {
    let now = world.block_anywhere_or_generate(cell);
    if now == block || !takeable(now) {
        return false;
    }
    match role {
        Role::Clear => now != AIR && !LIQUID[now as usize],
        Role::Fill => !LIQUID[now as usize] && !over_water(world, cell),
        _ => true,
    }
}

/// Whether the first block under `cell` that isn't air is water.
fn over_water(world: &mut World, cell: IVec3) -> bool {
    let mut at = cell - IVec3::new(0, 1, 0);
    for _ in 0..16 {
        let b = world.block_anywhere_or_generate(at);
        if b != AIR {
            return LIQUID[b as usize];
        }
        at.y -= 1;
    }
    false
}

/// A cell to build: where, which block, in which role.
type Job = (IVec3, BlockId, Role);

impl Sim {
    /// One tick of town `i`'s drones: their flights and work, then a launch when one is due.
    pub(crate) fn step_builders(&mut self, i: usize) {
        let mut drones = std::mem::take(&mut self.settlements.towns[i].drones);
        let mut k = 0;
        while k < drones.len() {
            let mut d = drones[k];
            d.prev = d.pos;
            let busy: Vec<IVec3> = drones.iter().map(|d| d.target).collect();
            let home = self.fly_town_drone(i, &mut d, &busy);
            if home {
                drones.remove(k);
            } else {
                drones[k] = d;
                k += 1;
            }
        }
        let t = &self.settlements.towns[i];
        let stagger = (t.pos.x.wrapping_mul(31).wrapping_add(t.pos.z.wrapping_mul(17))).rem_euclid(LAUNCH_EVERY as i32);
        let due = (self.tick + stagger as u64).is_multiple_of(LAUNCH_EVERY);
        if due && t.built > 0 && !t.stalled && drones.len() < fleet(t.built) {
            let busy: Vec<IVec3> = drones.iter().map(|d| d.target).collect();
            if let Some((target, block, role)) = self.town_job(i, &busy, None) {
                let pos = self.depot_top(i);
                let phase = Phase::Out;
                drones.push(TownDrone { pos, prev: pos, target, block, role, phase, timer: 0, load: LOAD });
            }
        }
        self.settlements.towns[i].drones = drones;
    }

    /// Advances a drone of town `i` one tick; true once it is home (it lands and leaves the list).
    fn fly_town_drone(&mut self, i: usize, d: &mut TownDrone, busy: &[IVec3]) -> bool {
        let step = SPEED * TICK;
        match d.phase {
            Phase::Out => {
                let (pos, arrived) = approach(d.pos, target_centre(d.target) + Vec3::new(0.0, HOVER, 0.0), step);
                d.pos = pos;
                if arrived {
                    (d.phase, d.timer) = (Phase::Work, WORK_TICKS);
                }
                false
            }
            Phase::Work => {
                d.timer = d.timer.saturating_sub(1);
                if d.timer > 0 {
                    return false;
                }
                if pending(&mut self.world, d.target, d.block, d.role) {
                    let old = self.world.block_anywhere_or_generate(d.target);
                    self.world.set_block_anywhere(d.target, d.block);
                    self.block_changed(d.target, old);
                }
                let stalled = self.settlements.towns[i].stalled;
                let next = (d.load > 1 && !stalled).then(|| self.town_job(i, busy, Some(d.target))).flatten();
                match next {
                    Some((target, block, role)) => {
                        (d.target, d.block, d.role, d.load, d.phase) = (target, block, role, d.load - 1, Phase::Out)
                    }
                    None => d.phase = Phase::Home,
                }
                false
            }
            Phase::Home => {
                let (pos, arrived) = approach(d.pos, self.depot_top(i), step);
                d.pos = pos;
                arrived
            }
        }
    }

    /// The next cell town `i` still has to build that no drone in `busy` is on: the first in plot order, or with
    /// `near`, the first within [`HOP`] of it. Moves the town's cursor past finished plots.
    fn town_job(&mut self, i: usize, busy: &[IVec3], near: Option<IVec3>) -> Option<Job> {
        let (anchor, kind) = (self.settlements.towns[i].pos, self.settlements.towns[i].kind);
        let plots = self.settlements.towns[i].plots.clone();
        let mut first = true;
        for (p, plot) in plots.iter().enumerate().skip(self.settlements.towns[i].cursor as usize) {
            let mut open = false;
            for (cell, block, role) in plot.cells(anchor, kind) {
                if busy.contains(&cell) {
                    open = true;
                    continue;
                }
                let far = |n: IVec3| (cell.x - n.x).abs().max((cell.y - n.y).abs()).max((cell.z - n.z).abs()) > HOP;
                if near.is_some_and(far) {
                    continue;
                }
                if pending(&mut self.world, cell, block, role) {
                    return Some((cell, block, role));
                }
            }
            if first && !open && near.is_none() {
                self.settlements.towns[i].cursor = p as u16 + 1;
            } else {
                first = false;
            }
        }
        None
    }

    /// Where town `i`'s drones leave from and land: just over its depot's roof.
    fn depot_top(&self, i: usize) -> Vec3 {
        let anchor = self.settlements.towns[i].pos;
        let cells = self.factory.footprint_at(anchor).map_or_else(|| vec![anchor], |f| f.2);
        let n = cells.len() as f64;
        let sum = cells.iter().fold(Vec3::new(0.0, 0.0, 0.0), |s, c| s + c.as_vec3());
        let top = cells.iter().map(|c| c.y).max().unwrap_or(anchor.y);
        Vec3::new(sum.x / n + 0.5, f64::from(top) + 1.5, sum.z / n + 0.5)
    }
}
