//! Moving water through pipework (core state, one tick at a time, from `Factory::update`).
//!
//! - A pump (powered, `PUMP_POWER`) lifts `PUMP_RATE` source blocks a second out of the water it
//!   touches: it searches from its six faces through water (sources and flows) up to `PUMP_RANGE`
//!   steps and takes the highest source, then the farthest (so a pool drains from its edges inward
//!   and what is left stays joined to the intake), then the lowest position. It holds up to
//!   `PUMP_HOLD` units and stops while full, so a network whose water nothing takes stands still. The
//!   sea refills what it takes (sim/water.rs), so its level never drops.
//! - An outlet pours up to `OUTLET_RATE` units a second, taking them from the pumps of its network in
//!   list order. Each unit becomes a source where it lands: straight down from the cell in front, or,
//!   when that lands on water, in the free cell of that water's edge that is lowest, then nearest
//!   (within `PUMP_RANGE`), then lowest in position, and resting on something (a block or a source).
//!   With nowhere to pour it waits.
//!
//! Every block it changes goes into `changed` (with the block it replaced), so `Sim::step` runs the
//! water rules on it. Searches read through the `*_anywhere` accessors.

use std::collections::VecDeque;

use rustc_hash::FxHashSet;

use crate::block::{flow_level, BlockId, AIR, LIQUID, WATER};
use crate::math::IVec3;
use crate::world::World;

use super::pipes::{Flow, Part, Pipework, UNIT};
use super::power::Power;
use super::{DIRS, FACES};

/// Source blocks a pump lifts a second at full power.
pub const PUMP_RATE: u32 = 2;
/// Units of water a pump holds for its outlets.
pub const PUMP_HOLD: u32 = 2;
/// Source blocks an outlet can pour a second.
pub const OUTLET_RATE: u32 = 4;
/// How far (in steps through water) pumps and outlets search.
pub const PUMP_RANGE: u32 = 16;

const UP: IVec3 = IVec3::new(0, 1, 0);

/// The best cell so far and its sort key (smaller wins).
type Pick = Option<((i32, u32, [i32; 3]), IVec3)>;

/// One tick of every pump, then every outlet. `pole` is each piece's power pole (pumps only).
pub(super) fn step_pipework(
    pieces: &mut [Pipework],
    pole: &[Option<u32>],
    power: &Power,
    world: &mut World,
    changed: &mut Vec<(IVec3, BlockId)>,
) {
    for (p, &pole) in pieces.iter_mut().zip(pole) {
        if p.part != Part::Pump {
            continue;
        }
        let speed = power.speed(pole);
        p.flow = match () {
            _ if p.held >= PUMP_HOLD => Flow::Full,
            _ if speed == 0 => Flow::NoPower,
            _ => Flow::Working,
        };
        if p.flow != Flow::Working {
            continue;
        }
        p.progress = (p.progress + PUMP_RATE * speed).min(UNIT);
        if p.progress < UNIT {
            continue;
        }
        match find_source(world, p.pos) {
            Some(s) => {
                world.set_block_anywhere(s, AIR);
                changed.push((s, WATER));
                p.held += 1;
                p.progress = 0;
            }
            None => p.flow = Flow::NoWater,
        }
    }
    for o in 0..pieces.len() {
        if pieces[o].part != Part::Outlet {
            continue;
        }
        let net = pieces[o].net;
        let Some(pump) = pieces.iter().position(|p| p.net == net && p.part == Part::Pump && p.held > 0) else {
            pieces[o].flow = Flow::Idle;
            continue;
        };
        let out = &mut pieces[o];
        out.progress = (out.progress + OUTLET_RATE * 1000).min(UNIT);
        out.flow = Flow::Working;
        if out.progress < UNIT {
            continue;
        }
        let Some(cell) = find_pour(world, out.pos + DIRS[out.facing as usize]) else {
            out.flow = Flow::Blocked;
            continue;
        };
        let old = world.block_anywhere_or_generate(cell);
        world.set_block_anywhere(cell, WATER);
        changed.push((cell, old));
        out.progress = 0;
        pieces[pump].held -= 1;
    }
}

/// The source a pump at `pos` takes next, if any is in reach.
fn find_source(world: &mut World, pos: IVec3) -> Option<IVec3> {
    let starts: Vec<IVec3> = FACES.iter().map(|&f| pos + f).collect();
    let mut best: Pick = None;
    search(world, &starts, |c, b, dist| {
        let key = (-c.y, u32::MAX - dist, [c.x, c.y, c.z]);
        if b == WATER && best.is_none_or(|k| key < k.0) {
            best = Some((key, c));
        }
    });
    best.map(|k| k.1)
}

/// Where a unit poured into the cell `front` ends up, if anywhere.
fn find_pour(world: &mut World, front: IVec3) -> Option<IVec3> {
    let b = world.block_anywhere_or_generate(front);
    let start = if fillable(b) {
        let mut c = front;
        while c.y > 0 && fillable(world.block_anywhere_or_generate(c - UP)) {
            c = c - UP;
        }
        if world.block_anywhere_or_generate(c - UP) != WATER {
            return Some(c);
        }
        c - UP
    } else if b == WATER {
        front
    } else {
        return None;
    };
    let mut best: Pick = None;
    let mut consider = |world: &mut World, c: IVec3, dist: u32| {
        let key = (c.y, dist, [c.x, c.y, c.z]);
        if best.is_none_or(|k| key < k.0) && fillable(world.block_anywhere_or_generate(c)) && rests(world, c) {
            best = Some((key, c));
        }
    };
    let mut edge = Vec::new();
    search(world, &[start], |c, _, dist| edge.push((c, dist)));
    for (c, dist) in edge {
        consider(world, c, dist);
        for f in FACES {
            consider(world, c + f, dist + 1);
        }
    }
    best.map(|k| k.1)
}

/// Visits every liquid cell reachable from `starts` through liquid within `PUMP_RANGE` steps, in
/// breadth-first order: `visit(cell, block, steps)`.
fn search(world: &mut World, starts: &[IVec3], mut visit: impl FnMut(IVec3, BlockId, u32)) {
    let mut seen: FxHashSet<IVec3> = FxHashSet::default();
    let mut queue = VecDeque::new();
    for &s in starts {
        if LIQUID[world.block_anywhere_or_generate(s) as usize] && seen.insert(s) {
            queue.push_back((s, 0));
        }
    }
    while let Some((c, dist)) = queue.pop_front() {
        visit(c, world.block_anywhere_or_generate(c), dist);
        if dist == PUMP_RANGE {
            continue;
        }
        for f in FACES {
            let n = c + f;
            if !seen.contains(&n) && LIQUID[world.block_anywhere_or_generate(n) as usize] {
                seen.insert(n);
                queue.push_back((n, dist + 1));
            }
        }
    }
}

/// Air or flowing water: where a poured unit can go.
fn fillable(b: BlockId) -> bool {
    b == AIR || flow_level(b).is_some()
}

/// Whether the cell under `c` holds water up: a block or a source, not air or flowing water.
fn rests(world: &mut World, c: IVec3) -> bool {
    !fillable(world.block_anywhere_or_generate(c - UP))
}
