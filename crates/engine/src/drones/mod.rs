//! Construction drones (Milestone 7): the core's flying builders. A drone port (`factory/process/hangar.rs`)
//! keeps drones at home; every [`LAUNCH_EVERY`] ticks a powered port with a drone at home looks for work
//! within its reach and sends one out. Work is a ghost (`ghosts.rs`):
//!
//! - a **build** ghost (a block or machine): the drone takes its item from a storage box touching the pad,
//!   flies to the cell, works [`WORK_TICKS`] and places it for real (`Sim::put_block`, the same code a player's
//!   placing runs), clearing the ghost;
//! - a **tear-down** mark (a ghost of air, `Action::MarkRemoval`): the drone flies to the block, works as long
//!   as hands take to break it, and breaks it (`Sim::dismantle`); what it leaves goes into the boxes touching the
//!   pad, and spills out as loose items when they are full.
//!
//! Then it flies home, puts anything it still carries back in the boxes and lands on the pad.
//!
//! Core state ([`Drones`], saved since version 27): every drone out, its port, where it is, its target,
//! its phase and what it carries. A port's stored drones are its buffer's. Positions are `f64` moved by
//! `+ - * / sqrt` only, so every machine computes the same flight. A drone never takes a ghost another drone is
//! on, a build whose item the boxes lack, one whose cells are blocked, or a mark on nothing (stale marks are
//! cleared). If its port is gone when it comes home, it drops as an item; breaking a port brings its
//! drones back to the player as items (`Sim::recall_drones`).
//!
//! Invariants: at most one drone per target; `Drone::load` is one item or empty; a drone is either in this list
//! or on a pad (never both), so the fleet is conserved (`tests.rs`).
//! To change a number: the constants below, and a port tier's reach and fleet in `hangar.rs`.

pub mod cargo;
mod earthworks;
#[cfg(test)]
mod tests;

use crate::block::{self, AIR};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::PortInfo;
use crate::inventory::Stack;
use crate::item::{ItemId, CARGO_DRONE, DRONE};
use crate::math::{IVec3, Vec3};
use crate::sim::{PlayerId, Sim};
use crate::{TICK, TICK_RATE};

/// Blocks a second a drone flies.
pub const SPEED: f64 = 8.0;
/// Ticks between a port's launches.
pub const LAUNCH_EVERY: u64 = 30;
/// Ticks a drone works on placing a block.
pub const WORK_TICKS: u32 = TICK_RATE;
/// Most ticks it works on breaking one, however hard.
const MAX_BREAK_TICKS: u32 = 4 * TICK_RATE;
/// How high above its target a working drone hovers, and above its pad when it waits to land.
const HOVER: f64 = 2.2;
/// The player a drone's work is credited to in events (sounds, light).
const CREDIT: PlayerId = PlayerId(0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Flying to the target.
    Out,
    /// Hovering over it, working.
    Work,
    /// Flying back to the pad.
    Home,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Drone {
    /// The anchor of the port it flew from.
    pub port: IVec3,
    pub pos: Vec3,
    /// Where it was a tick ago (derived, for drawing between ticks; not saved).
    pub prev: Vec3,
    /// The ghost it works on: its anchor cell.
    pub target: IVec3,
    pub phase: Phase,
    /// Ticks of work left (`Phase::Work`).
    pub timer: u32,
    /// What it carries to the build, or the unused item it brings back.
    pub load: Stack,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Drones {
    pub list: Vec<Drone>,
}

impl Drones {
    /// How many drones of the port at `port` are out.
    pub fn out_of(&self, port: IVec3) -> u32 {
        self.list.iter().filter(|d| d.port == port).count() as u32
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.list.len());
        for d in &self.list {
            w.ivec3(d.port);
            w.vec3(d.pos);
            w.ivec3(d.target);
            w.u8(d.phase as u8);
            w.u32(d.timer);
            w.item(d.load.item);
            w.u32(d.load.count);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Drones> {
        let n = r.count()?;
        let mut list = Vec::new();
        for _ in 0..n {
            let (port, pos, target) = (r.ivec3()?, r.vec3()?, r.ivec3()?);
            let phase = match r.u8()? {
                0 => Phase::Out,
                1 => Phase::Work,
                2 => Phase::Home,
                _ => return None,
            };
            let (timer, item, count) = (r.u32()?, r.item()?, r.u32()?);
            if count > 1 || !pos.x.is_finite() || !pos.y.is_finite() || !pos.z.is_finite() {
                return None;
            }
            list.push(Drone { port, pos, prev: pos, target, phase, timer, load: Stack { item, count } });
        }
        Some(Drones { list })
    }
}

/// What a port found to do.
struct Job {
    /// The cell to work on: a ghost's anchor, or a site's cell.
    pos: IVec3,
    /// What it builds with (`ItemId::NONE`: it breaks) and the box that item comes from.
    item: ItemId,
    from: Option<IVec3>,
}

impl Sim {
    /// One tick of every drone: flights and work, landings, then launches from the ports.
    pub(crate) fn step_drones(&mut self) {
        let ports = self.factory.ports();
        for p in &ports {
            let (builders, couriers) = (self.drones.out_of(p.anchor), self.cargo.out_of(p.anchor));
            let kind = if couriers > 0 {
                CARGO_DRONE
            } else if builders > 0 {
                DRONE
            } else {
                ItemId::NONE
            };
            self.factory.set_port_flight(p.anchor, builders + couriers, kind);
        }
        if self.drones.list.is_empty() && ports.iter().all(|p| p.home == 0) {
            return;
        }
        let flying = std::mem::take(&mut self.drones.list);
        let mut kept = Vec::with_capacity(flying.len());
        for mut d in flying {
            d.prev = d.pos;
            if self.fly(&mut d) {
                kept.push(d);
            }
        }
        self.drones.list = kept;
        for p in self.factory.ports() {
            let stagger =
                (p.anchor.x.wrapping_mul(31).wrapping_add(p.anchor.z.wrapping_mul(17))).rem_euclid(LAUNCH_EVERY as i32);
            if p.home > 0 && p.powered && (self.tick + stagger as u64).is_multiple_of(LAUNCH_EVERY) {
                self.launch(&p);
            }
        }
    }

    /// Brings back every drone of the port at `port` that is out, as a stack of drone items (breaking the port).
    pub(crate) fn recall_drones(&mut self, port: IVec3) -> Option<Stack> {
        let before = self.drones.list.len();
        self.drones.list.retain(|d| d.port != port);
        let back = (before - self.drones.list.len()) as u32;
        (back > 0).then_some(Stack { item: DRONE, count: back })
    }

    /// Advances `d` one tick; false when it has landed or been lost (it is gone from the list).
    fn fly(&mut self, d: &mut Drone) -> bool {
        let step = SPEED * TICK;
        match d.phase {
            Phase::Out => {
                let goal = target_centre(d.target) + Vec3::new(0.0, HOVER, 0.0);
                let (pos, arrived) = approach(d.pos, goal, step);
                d.pos = pos;
                if arrived {
                    d.phase = Phase::Work;
                    d.timer = self.work_ticks(d.target);
                }
                true
            }
            Phase::Work => {
                d.timer = d.timer.saturating_sub(1);
                if d.timer == 0 {
                    self.finish_work(d);
                    d.phase = Phase::Home;
                }
                true
            }
            Phase::Home => {
                let Some(centre) = self.factory.port_centre(d.port) else {
                    self.lose(d);
                    return false;
                };
                let (pos, arrived) = approach(d.pos, centre + Vec3::new(0.0, HOVER * 0.5, 0.0), step);
                d.pos = pos;
                if arrived {
                    self.land(d);
                }
                !arrived
            }
        }
    }

    /// How long it takes to work on the cell at `target`: placing takes [`WORK_TICKS`], breaking as long as
    /// bare hands would, up to [`MAX_BREAK_TICKS`].
    fn work_ticks(&mut self, target: IVec3) -> u32 {
        if self.breaks(target) {
            let at = self.factory.footprint_at(target).map_or(target, |f| f.0);
            let time = block::def(self.world.block_anywhere_or_generate(at)).break_time.max(0.0);
            return ((time * TICK_RATE as f32) as u32).clamp(WORK_TICKS, MAX_BREAK_TICKS);
        }
        WORK_TICKS
    }

    /// The end of a drone's work on its target: place the block or break it.
    fn finish_work(&mut self, d: &mut Drone) {
        let Some(g) = self.ghosts.covering(d.target).copied().filter(|g| g.pos == d.target) else {
            return self.finish_site_cell(d);
        };
        if g.block == AIR {
            self.break_into_boxes(d.port, g.pos);
            self.ghosts.remove_at(g.pos);
        } else if self.put_block(CREDIT, g.pos, g.item(), g.facing, g.pos - IVec3::new(0, 1, 0)) {
            self.ghosts.built(g.pos, g.block);
            d.load = Stack::default();
        }
    }

    /// Breaks the block at `pos` by hand; what it leaves goes into the boxes at `port`, and spills when they are full.
    fn break_into_boxes(&mut self, port: IVec3, pos: IVec3) {
        if let Some((pos, drops)) = self.dismantle(CREDIT, pos, false) {
            let boxes = self.factory.port_boxes(port);
            for s in drops {
                let left = self.factory.store_in_boxes(&boxes, s);
                if left.count > 0 {
                    self.drop_stacks(pos, vec![left]);
                }
            }
        }
    }

    /// A drone reaches its pad: what it carries goes back in the boxes, and it lands.
    fn land(&mut self, d: &Drone) {
        if d.load.count > 0 {
            let boxes = self.factory.port_boxes(d.port);
            let left = self.factory.store_in_boxes(&boxes, d.load);
            if left.count > 0 {
                self.drop_stacks(d.pos.floor(), vec![left]);
            }
        }
        if !self.factory.port_land(d.port, DRONE) {
            self.lose(d);
        }
    }

    /// A drone that cannot come home (its port is gone or full): it and its load fall as items.
    fn lose(&mut self, d: &Drone) {
        let mut fallen = vec![Stack { item: DRONE, count: 1 }];
        fallen.extend(Some(d.load).filter(|s| s.count > 0));
        self.drop_stacks(d.pos.floor(), fallen);
    }

    /// Sends a drone from `port` to the nearest job it can do, if there is one.
    fn launch(&mut self, port: &PortInfo) {
        let Some(job) = self.pick_job(port) else { return };
        let mut load = Stack::default();
        if let Some(from) = job.from {
            let item = job.item;
            if !self.factory.box_take(from, item) {
                return;
            }
            load = Stack { item, count: 1 };
        }
        if !self.factory.port_take(port.anchor, DRONE) {
            if let Some(from) = job.from {
                self.factory.store_in_boxes(&[from], load);
            }
            return;
        }
        let pos = port.centre + Vec3::new(0.0, 0.4, 0.0);
        self.drones.list.push(Drone {
            port: port.anchor,
            pos,
            prev: pos,
            target: job.pos,
            phase: Phase::Out,
            timer: 0,
            load,
        });
    }

    /// The nearest ghost in reach that no drone is on and that can be worked now. Also clears marks on nothing.
    fn pick_job(&mut self, port: &PortInfo) -> Option<Job> {
        let boxes = self.factory.port_boxes(port.anchor);
        let reach2 = port.reach * port.reach;
        let mut best: Option<(f64, Job)> = None;
        let mut stale = Vec::new();
        for g in self.ghosts.iter() {
            let to = target_centre(g.pos) - port.centre;
            let dist2 = to.x * to.x + to.y * to.y + to.z * to.z;
            if dist2 > reach2 || best.as_ref().is_some_and(|b| b.0 <= dist2) {
                continue;
            }
            if self.drones.list.iter().any(|d| d.target == g.pos) {
                continue;
            }
            let from = if g.block == AIR {
                let at = self.factory.footprint_at(g.pos).map_or(g.pos, |f| f.0);
                let id = self.world.block_anywhere_or_generate(at);
                if id == AIR {
                    stale.push(g.pos);
                    continue;
                }
                if block::def(id).break_time < 0.0 {
                    continue;
                }
                None
            } else {
                let free = g.cells().into_iter().all(|c| block::replaceable(self.world.block_anywhere_or_generate(c)));
                let item = g.item();
                let Some(from) = boxes.iter().copied().find(|&b| self.factory.box_count(b, item) > 0).filter(|_| free)
                else {
                    continue;
                };
                Some(from)
            };
            let item = if from.is_some() { g.item() } else { ItemId::NONE };
            best = Some((dist2, Job { pos: g.pos, item, from }));
        }
        for pos in stale {
            self.ghosts.remove_at(pos);
        }
        let near = best.as_ref().map(|b| b.0);
        self.site_job(port, &boxes, near).or(best.map(|b| b.1))
    }
}

/// The middle of the cell at `cell`.
fn target_centre(cell: IVec3) -> Vec3 {
    cell.as_vec3() + Vec3::new(0.5, 0.5, 0.5)
}

/// `from` moved `step` towards `goal`, and whether it got there.
fn approach(from: Vec3, goal: Vec3, step: f64) -> (Vec3, bool) {
    let to = goal - from;
    let dist = to.length();
    if dist <= step {
        (goal, true)
    } else {
        (from + to * (step / dist), false)
    }
}
