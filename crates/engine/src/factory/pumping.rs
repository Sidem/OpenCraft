//! Moving water through pipework (core state, one tick at a time, from `Factory::update`).
//!
//! - **Head, pressure & energy conservation:** Every water pipe network has a peak height `peak_y` (highest piece in
//!   `link_pipework`). A pump at `pump.y` must overcome `lift = (peak_y - pump.y).max(0)` blocks. Each tier has a
//!   maximum lift (`max_head`): pressure drops linearly from 100% at 0 lift to 25% at `max_head`, and hits 0%
//!   (`Flow::TooHigh`) above `max_head`. Lift power scales strictly with both flow rate and climb height
//!   (`power + lift * lift_power`, where `lift_power = 7 * rate` kW/block), guaranteeing that lifting water always
//!   costs more electricity than water wheels can ever recover from its fall (`<= 5.71 kW` per block/s·block).
//! - **Pumps:** Powered pumps with head lift source blocks out of water within `PUMP_RANGE` steps (highest, then
//!   farthest) into their `hold` buffer, scaled by both power share and head pressure. The sea refills at or below
//!   `SEA_LEVEL`.
//! - **Outlets & mass-flow splitting:** Active pump flow on a pipe network divides evenly among all open outlets on
//!   that network (`per_outlet` milli-blocks/s), setting each outlet's live voxel stream level (`FLOW_1..=FLOW_7`,
//!   preserved through vertical falls in `sim/water.rs`) and 3D jet pressure. Under brownout or over-splitting, flow
//!   and stream level drop smoothly — so a wheel-powered pump loop naturally spirals down to 0 kW without any
//!   artificial water tagging. Completed pumped units deposit `WATER` sources 1-to-1 at `find_pour`.

use std::collections::VecDeque;

use rustc_hash::FxHashSet;

use crate::block::{flow, flow_level, BlockId, AIR, LIQUID, WATER};
use crate::math::IVec3;
use crate::world::World;
use crate::worldgen::SEA_LEVEL;

use super::pipes::{Flow, Part, Pipework, UNIT};
use super::power::Power;
use super::{DIRS, FACES};

/// What a pump tier does, Mk1 first.
pub struct PumpTier {
    /// Source blocks lifted a second at full power and 0 lift (a divisor of 60, so the ticks are whole).
    pub rate: u32,
    /// Units of water it holds for its outlets.
    pub hold: u32,
    /// Base kW at 0 lift (flat or downhill).
    pub power: u32,
    /// Extra kW per block of vertical lift (`peak_y - pump.y`); `7 * rate` ensures `P_pump > P_hydro` at every height.
    pub lift_power: u32,
    /// Maximum vertical lift (in blocks) above the pump before pressure hits zero.
    pub max_head: u32,
}

pub const PUMP_TIERS: [PumpTier; 3] = [
    PumpTier { rate: 2, hold: 2, power: 5, lift_power: 14, max_head: 12 },
    PumpTier { rate: 4, hold: 4, power: 10, lift_power: 28, max_head: 24 },
    PumpTier { rate: 6, hold: 6, power: 20, lift_power: 42, max_head: 40 },
];
/// Source blocks an outlet can pour a second at 100% pressure.
pub const OUTLET_RATE: u32 = 4;
/// How far (in steps through water) pumps and outlets search.
pub const PUMP_RANGE: u32 = 16;
/// How far sideways from the cell in front of it an outlet may place a source.
pub const POUR_REACH: i32 = 3;
/// Static pressure recovered (in permille) per block an outlet sits below its pipe network's peak.
const DROP_RECOVERY_PER_BLOCK: u32 = 40;
/// Milli-blocks/s required per voxel flow level (`FLOW_1` = 0.25/s .. `FLOW_7` = 1.75/s).
const FLOW_UNIT_MILLI: u32 = 250;

const UP: IVec3 = IVec3::new(0, 1, 0);

/// The best cell so far and its sort key (smaller wins).
type Pick = Option<((i32, u32, [i32; 3]), IVec3)>;

/// Hydraulic pressure (0..=1000‰) a pump at `pump_y` delivers over a pipe network peaking at `peak_y`:
/// 1000 at 0 lift, linearly down to 250 at `tier.max_head`, and 0 when `lift > tier.max_head`.
pub fn pump_pressure(pump_y: i32, peak_y: i32, tier: &PumpTier) -> u16 {
    let lift = (peak_y - pump_y).max(0) as u32;
    if lift > tier.max_head {
        0
    } else {
        (1000 - (lift * 750) / tier.max_head) as u16
    }
}

/// Voxel flow level (0..=7) sustained by `milli_rate` (milli-blocks/s) exiting an outlet.
pub fn stream_level_for_rate(milli_rate: u32) -> u8 {
    (milli_rate / FLOW_UNIT_MILLI).min(7) as u8
}

/// One tick of every pump, then every outlet. `pole` is each piece's power pole (pumps only).
pub(super) fn step_pipework(
    pieces: &mut [Pipework],
    pole: &[Option<u32>],
    power: &Power,
    world: &mut World,
    changed: &mut Vec<(IVec3, BlockId)>,
) {
    // Pass 1: step every pump and record its effective delivery rate (milli-blocks/s).
    let mut pump_flow = vec![0u32; pieces.len()];
    for (i, (p, &pole)) in pieces.iter_mut().zip(pole).enumerate() {
        if p.part != Part::Pump {
            continue;
        }
        let tier = p.pump_stats();
        p.pressure = pump_pressure(p.pos.y, p.peak_y, tier);
        let speed = power.speed(pole);
        p.flow = match () {
            _ if p.pressure == 0 => Flow::TooHigh,
            _ if p.held >= tier.hold => Flow::Full,
            _ if speed == 0 => Flow::NoPower,
            _ => Flow::Working,
        };
        if p.flow == Flow::Working {
            // Effective delivery scales with BOTH head pressure and electric power share.
            let eff = tier.rate * speed * u32::from(p.pressure) / 1000;
            pump_flow[i] = eff;
            let step = eff.max(1);
            p.progress = (p.progress + step).min(UNIT);
            if p.progress >= UNIT {
                match find_source(world, p.pos) {
                    Some(s) => {
                        world.set_block_anywhere_later(s, AIR);
                        changed.push((s, WATER));
                        p.held += 1;
                        p.progress = 0;
                    }
                    None => {
                        p.flow = Flow::NoWater;
                        pump_flow[i] = 0;
                    }
                }
            }
        } else if p.flow == Flow::Full && speed > 0 {
            // A full pump waiting on an outlet can still deliver its rated flow as soon as the outlet draws.
            pump_flow[i] = tier.rate * speed * u32::from(p.pressure) / 1000;
        }
    }

    // Pass 2: count open (pourable) outlets per network so pumped flow splits evenly across them.
    let net_count = pieces.iter().map(|p| p.net as usize + 1).max().unwrap_or(0);
    let mut open_outlets = vec![0u32; net_count];
    let mut pour_target: Vec<Option<IVec3>> = vec![None; pieces.len()];
    for o in 0..pieces.len() {
        if pieces[o].part != Part::Outlet {
            continue;
        }
        let front = pieces[o].pos + DIRS[pieces[o].facing as usize];
        let target = find_pour(world, front);
        pour_target[o] = target;
        if target.is_some() {
            open_outlets[pieces[o].net as usize] += 1;
        }
    }

    // Pass 3: step every outlet using its network's split pump flow and pressure.
    for o in 0..pieces.len() {
        if pieces[o].part != Part::Outlet {
            continue;
        }
        let net = pieces[o].net;
        let front = pieces[o].pos + DIRS[pieces[o].facing as usize];
        let cur_front = world.block_anywhere_or_generate(front);
        let was_pouring = pieces[o].pouring > 0 || flow_level(cur_front).is_some();
        let mut any_pump = false;
        let mut any_head = false;
        let mut active_pumping = false;
        let mut pump: Option<usize> = None;
        let mut best_p: u16 = 0;
        let mut net_flow: u32 = 0;
        for (i, p) in pieces.iter().enumerate() {
            if p.net == net && p.part == Part::Pump {
                any_pump = true;
                if p.pressure > 0 {
                    any_head = true;
                    net_flow += pump_flow[i];
                    if p.held > 0 {
                        if pump.is_none() {
                            pump = Some(i);
                        }
                        best_p = best_p.max(p.pressure);
                    } else if was_pouring && p.flow == Flow::Working {
                        active_pumping = true;
                        best_p = best_p.max(p.pressure);
                    }
                }
            }
        }
        if pump.is_none() && !active_pumping {
            let out = &mut pieces[o];
            out.flow = if any_pump && !any_head { Flow::TooHigh } else { Flow::Idle };
            out.pressure = 0;
            out.pouring = 0;
            if was_pouring {
                let cur = world.block_anywhere_or_generate(front);
                if flow_level(cur).is_some() {
                    changed.push((front, cur));
                }
            }
            continue;
        }
        let drop = (pieces[o].peak_y - pieces[o].pos.y).max(0) as u32;
        let head_p = (u32::from(best_p) + drop * DROP_RECOVERY_PER_BLOCK).min(1000);
        let Some(cell) = pour_target[o] else {
            let out = &mut pieces[o];
            out.flow = Flow::Blocked;
            out.pressure = head_p as u16;
            out.pouring = 0;
            if was_pouring {
                let cur = world.block_anywhere_or_generate(front);
                if flow_level(cur).is_some() {
                    changed.push((front, cur));
                }
            }
            continue;
        };
        let split = open_outlets[net as usize].max(1);
        let per_outlet = net_flow / split;
        let lvl = stream_level_for_rate(per_outlet);
        // Visual jet pressure reflects both head pressure and actual flow share (diminishes under brownout or splitting).
        let flow_p = (per_outlet * 1000 / (FLOW_UNIT_MILLI * 7)).min(1000);
        let outlet_p = head_p.min(flow_p) as u16;
        let out = &mut pieces[o];
        out.pressure = outlet_p;
        out.flow = Flow::Working;
        out.pouring = lvl;
        let cur_front = world.block_anywhere_or_generate(front);
        if lvl > 0 {
            let target_front = flow(lvl);
            if fillable(cur_front) && cur_front != target_front {
                world.set_block_anywhere_later(front, target_front);
                changed.push((front, cur_front));
            }
        } else if was_pouring && flow_level(cur_front).is_some() {
            changed.push((front, cur_front));
        }
        let Some(pump) = pump else {
            continue;
        };
        let step = (OUTLET_RATE * u32::from(outlet_p.max(100))).max(1);
        out.progress = (out.progress + step).min(UNIT);
        if out.progress < UNIT {
            continue;
        }
        let old = world.block_anywhere_or_generate(cell);
        world.set_block_anywhere_later(cell, WATER);
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
