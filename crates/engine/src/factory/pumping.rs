//! Moving water through pipework (core state, one tick at a time, from `Factory::update`).
//!
//! - A pump (powered) lifts source blocks out of the water it touches, as fast as its tier says
//!   (`PUMP_TIERS`: rate, hold and kW): it searches from its six faces through water (sources and flows) up to `PUMP_RANGE`
//!   steps and takes the highest source, then the farthest (so a pool drains from its edges inward
//!   and what is left stays joined to the intake), then the lowest position. It holds up to
//!   its `hold` units and stops while full, so a network whose water nothing takes stands still. The
//!   sea refills what it takes (sim/water.rs), so its level never drops.
//! - An outlet pours up to `OUTLET_RATE` units a second, taking them from the pumps of its network in
//!   list order. Each unit becomes a source where it lands: straight down from the cell in front, or,
//!   when that lands on water, in the free cell of that water's edge that is lowest, then nearest
//!   (within `PUMP_RANGE`), then lowest in position, and resting on something (a block or a source).
//!   It never places a source above the cell in front, more than `POUR_REACH` cells from it sideways,
//!   or at or below `SEA_LEVEL` away from existing water (a lone source there floods all connected air).
//!   With nowhere to pour it waits (`Flow::Blocked`), so a full hole stops the outlet.
//!
//! Every block it changes goes into `changed` (with the block it replaced), so `Sim::step` runs the
//! water rules on it. Searches read through the `*_anywhere` accessors.

use std::collections::VecDeque;

use rustc_hash::FxHashSet;

use crate::block::{flow_level, BlockId, AIR, LIQUID, WATER};
use crate::math::IVec3;
use crate::world::World;
use crate::worldgen::SEA_LEVEL;

use super::pipes::{Flow, Part, Pipework, UNIT};
use super::power::Power;
use super::{DIRS, FACES};

/// What a pump tier does, Mk1 first.
pub struct PumpTier {
    /// Source blocks lifted a second at full power (a divisor of 60, so the ticks are whole).
    pub rate: u32,
    /// Units of water it holds for its outlets.
    pub hold: u32,
    /// kW while it has room for water.
    pub power: u32,
}

pub const PUMP_TIERS: [PumpTier; 3] = [
    PumpTier { rate: 2, hold: 2, power: 5 },
    PumpTier { rate: 4, hold: 4, power: 10 },
    PumpTier { rate: 6, hold: 6, power: 20 },
];
/// Source blocks an outlet can pour a second.
pub const OUTLET_RATE: u32 = 4;
/// How far (in steps through water) pumps and outlets search.
pub const PUMP_RANGE: u32 = 16;
/// How far sideways from the cell in front of it an outlet may place a source.
pub const POUR_REACH: i32 = 3;

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
            _ if p.held >= p.pump_stats().hold => Flow::Full,
            _ if speed == 0 => Flow::NoPower,
            _ => Flow::Working,
        };
        if p.flow != Flow::Working {
            continue;
        }
        p.progress = (p.progress + p.pump_stats().rate * speed).min(UNIT);
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

/// Where a unit poured into the cell `front` ends up, if anywhere: the column below `front`, or the
/// edge of the water it lands on, but never higher than `front` or more than `POUR_REACH` cells from
/// it sideways (so a full hole blocks the outlet rather than flooding the land around it).
fn find_pour(world: &mut World, front: IVec3) -> Option<IVec3> {
    let b = world.block_anywhere_or_generate(front);
    let start = if fillable(b) {
        let mut c = front;
        while c.y > 0 && fillable(world.block_anywhere_or_generate(c - UP)) {
            c = c - UP;
        }
        if world.block_anywhere_or_generate(c - UP) != WATER {
            return pourable(world, c).then_some(c);
        }
        c - UP
    } else if b == WATER {
        front
    } else {
        return None;
    };
    let mut best: Pick = None;
    let mut consider = |world: &mut World, c: IVec3, dist: u32| {
        let near = (c.x - front.x).abs() <= POUR_REACH && (c.z - front.z).abs() <= POUR_REACH;
        let key = (c.y, dist, [c.x, c.y, c.z]);
        if best.is_none_or(|k| key < k.0)
            && near
            && c.y <= front.y
            && fillable(world.block_anywhere_or_generate(c))
            && rests(world, c)
            && pourable(world, c)
        {
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

/// Whether a source may be placed at `c`. At or below `SEA_LEVEL` a source floods everything connected
/// (sim/water.rs), so there it must join water that is already there.
fn pourable(world: &mut World, c: IVec3) -> bool {
    c.y > SEA_LEVEL || FACES.iter().any(|&f| f.y >= 0 && world.block_anywhere_or_generate(c + f) == WATER)
}

/// Whether the cell under `c` holds water up: a block or a source, not air or flowing water.
fn rests(world: &mut World, c: IVec3) -> bool {
    !fillable(world.block_anywhere_or_generate(c - UP))
}
