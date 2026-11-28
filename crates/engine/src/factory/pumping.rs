//! Moving water through pipework (core state, one tick at a time, from `Factory::update`).
//!
//! - **Head, pressure & energy conservation:** Every water pipe network has a peak height `peak_y` (highest piece in
//!   `link_pipework`). A pump at `pump.y` must overcome `lift = (peak_y - pump.y).max(0)` blocks. Each tier has a
//!   maximum lift (`max_head`): pressure drops linearly from 100% at 0 lift to 25% at `max_head`, and hits 0%
//!   (`Flow::TooHigh`) above `max_head`. Lift power scales strictly with both flow rate and climb height
//!   (`power + lift * lift_power`, where `lift_power = 7 * rate` kW/block), guaranteeing that lifting water always
//!   costs more electricity than water wheels can ever recover from its fall (`<= 5.71 kW` per block/s·block).
//! - **Strict mass conservation (pumps):** Powered pumps with head lift exactly 1 block (8 eighths) per cycle out of
//!   reachable liquid within `PUMP_RANGE` steps (highest, then farthest — draining full `WATER` blocks or accumulating
//!   partial `FLOW_1..=FLOW_7` cells up to 8 eighths) into `hold`. The sea refills at or below `SEA_LEVEL`.
//! - **Strict mass conservation (outlets):** Outlets never create phantom water. As `out.progress` advances through
//!   each `EIGHTH` (`UNIT / 8`) of a claimed unit, the outlet injects real finite eighths (`1..=8`) directly into its
//!   `front` cell, where cellular water (`sim/water.rs`) drops them naturally down cliffs and over water wheels.

use std::collections::VecDeque;

use rustc_hash::FxHashSet;

use crate::block::{block_of_wet, flow_level, wet_of, BlockId, AIR, ICE, LIQUID, WATER};
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
/// How far sideways from the cell in front of it an outlet checks for basin room before reporting `Blocked`.
pub const POUR_REACH: i32 = 3;
/// Progress per 1/8th of a water block emitted by an outlet (`60_000 / 8 = 7_500`, exact).
const EIGHTH: u32 = UNIT / 8;
/// Static pressure recovered (in permille) per block an outlet sits below its pipe network's peak.
const DROP_RECOVERY_PER_BLOCK: u32 = 40;
/// Milli-blocks/s required per visual jet level (`1` = 0.25/s .. `7` = 1.75/s).
const FLOW_UNIT_MILLI: u32 = 250;

const UP: IVec3 = IVec3::new(0, 1, 0);

/// A candidate liquid cell with its deterministic priority key: `(key, pos, block, mass)`.
type WaterCell = ((i32, u32, [i32; 3]), IVec3, BlockId, u8);

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

/// Visual jet flow level (0..=7) sustained by `milli_rate` (milli-blocks/s) exiting an outlet.
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
                match take_water_unit(world, p.pos, changed) {
                    Intake::Full => {
                        p.held += 1;
                        p.progress = 0;
                    }
                    Intake::Dregs => {
                        p.progress = 0;
                    }
                    Intake::Empty => {
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
    let mut can_pour = vec![false; pieces.len()];
    for o in 0..pieces.len() {
        if pieces[o].part != Part::Outlet {
            continue;
        }
        let front = pieces[o].pos + DIRS[pieces[o].facing as usize];
        if can_pour_into(world, front) {
            can_pour[o] = true;
            open_outlets[pieces[o].net as usize] += 1;
        }
    }

    // Pass 3: step every outlet, emitting conserved eighths directly into `front`.
    for o in 0..pieces.len() {
        if pieces[o].part != Part::Outlet {
            continue;
        }
        let net = pieces[o].net;
        let front = pieces[o].pos + DIRS[pieces[o].facing as usize];
        let claimed = pieces[o].progress >= EIGHTH;
        let mut any_pump = false;
        let mut any_head = false;
        let mut pump: Option<usize> = None;
        let mut best_p: u16 = 0;
        let mut net_flow: u32 = 0;
        for (i, p) in pieces.iter().enumerate() {
            if p.net == net && p.part == Part::Pump {
                any_pump = true;
                if p.pressure > 0 {
                    any_head = true;
                    net_flow += pump_flow[i];
                    best_p = best_p.max(p.pressure);
                    if p.held > 0 && pump.is_none() {
                        pump = Some(i);
                    }
                }
            }
        }
        if !claimed && pump.is_none() {
            let out = &mut pieces[o];
            out.flow = if any_pump && !any_head { Flow::TooHigh } else { Flow::Idle };
            out.pressure = 0;
            out.pouring = 0;
            out.progress = 0;
            continue;
        }
        let drop = (pieces[o].peak_y - pieces[o].pos.y).max(0) as u32;
        let head_p = (u32::from(best_p.max(if claimed { 250 } else { 0 })) + drop * DROP_RECOVERY_PER_BLOCK).min(1000);
        if !can_pour[o] {
            let out = &mut pieces[o];
            out.flow = Flow::Blocked;
            out.pressure = head_p as u16;
            out.pouring = 0;
            continue;
        }

        let split = open_outlets[net as usize].max(1);
        let per_outlet = net_flow / split;
        let capacity = (OUTLET_RATE * head_p.max(100)).max(1);
        let buffered = pump.is_some_and(|i| pieces[i].held >= 2);
        let step = if per_outlet > 0 && !buffered { per_outlet.min(capacity).max(1) } else { capacity };
        let eff_rate = if per_outlet > 0 { per_outlet } else { step };
        let lvl = stream_level_for_rate(eff_rate).max(1);
        let flow_p = (eff_rate * 1000 / (FLOW_UNIT_MILLI * 7)).min(1000);
        let outlet_p = head_p.min(flow_p).max(100) as u16;

        let prev_e = (pieces[o].progress / EIGHTH).min(8);
        let next_p = (pieces[o].progress + step).min(UNIT);
        let next_e = (next_p / EIGHTH).min(8);

        if next_e > prev_e {
            let want = (next_e - prev_e) as u8;
            let target = find_pour_cell(world, front);
            let Some((dst, cur_b, cur_m)) = target else {
                let out = &mut pieces[o];
                out.progress = prev_e * EIGHTH + EIGHTH - 1;
                out.flow = Flow::Blocked;
                out.pressure = outlet_p;
                out.pouring = 0;
                continue;
            };
            let emit = want.min(8 - cur_m);
            if prev_e == 0 {
                let Some(pi) = pump else {
                    pieces[o].flow = Flow::Idle;
                    pieces[o].pouring = 0;
                    pieces[o].progress = 0;
                    continue;
                };
                pieces[pi].held -= 1;
            }
            let new_b = block_of_wet(cur_m + emit);
            world.set_block_anywhere_later(dst, new_b);
            changed.push((dst, cur_b));
            let done_e = prev_e + u32::from(emit);
            let out = &mut pieces[o];
            out.progress = if done_e >= 8 {
                0
            } else if emit == want {
                next_p
            } else {
                done_e * EIGHTH
            };
            out.flow = Flow::Working;
            out.pressure = outlet_p;
            out.pouring = lvl;
        } else {
            let out = &mut pieces[o];
            out.progress = next_p;
            out.flow = Flow::Working;
            out.pressure = outlet_p;
            out.pouring = lvl;
        }
    }
}

enum Intake {
    /// Lifted a full 8-eighth unit into the pump's buffer.
    Full,
    /// Cleared leftover `< 8` eighth sump dregs without conjuring a full unit.
    Dregs,
    /// No liquid within reach.
    Empty,
}

/// Removes up to 8 eighths (1 full unit) of liquid water reachable from `pos` within `PUMP_RANGE` steps,
/// taking highest and farthest cells first (draining full `WATER` blocks or accumulating partial `FLOW_1..=FLOW_7`
/// cells). If fewer than 8 eighths remain in total, clears the final dregs cleanly without adding a phantom unit to
/// `held`. A single frozen `ICE` cap directly below the pump still lets it reach liquid underneath.
fn take_water_unit(world: &mut World, pos: IVec3, changed: &mut Vec<(IVec3, BlockId)>) -> Intake {
    let mut starts: Vec<IVec3> = FACES.iter().map(|&f| pos + f).collect();
    if world.block_anywhere_or_generate(pos - UP) == ICE {
        starts.push(pos - UP - UP);
    }
    let mut cells: Vec<WaterCell> = Vec::new();
    let mut total = 0u32;
    search(world, &starts, |c, b, dist| {
        let m = wet_of(b);
        if m > 0 {
            let key = (-c.y, u32::MAX - dist, [c.x, c.y, c.z]);
            cells.push((key, c, b, m));
            total += u32::from(m);
        }
    });
    if total == 0 {
        return Intake::Empty;
    }
    if total < 8 {
        for (_, c, old_b, _) in cells {
            world.set_block_anywhere_later(c, AIR);
            changed.push((c, old_b));
        }
        return Intake::Dregs;
    }
    cells.sort_unstable_by_key(|e| e.0);
    let mut need = 8u8;
    for (_, c, old_b, m) in cells {
        if need == 0 {
            break;
        }
        let take = need.min(m);
        world.set_block_anywhere_later(c, block_of_wet(m - take));
        changed.push((c, old_b));
        need -= take;
    }
    Intake::Full
}

/// Whether an outlet facing `front` has somewhere safe to pour.
fn can_pour_into(world: &mut World, front: IVec3) -> bool {
    find_pour_cell(world, front).is_some()
}

/// Finds the cell to inject pumped eighths into: `front` itself whenever it has room (`< 8`), or when submerged
/// (`front == WATER`), the lowest/nearest connected basin cell within `POUR_REACH` sideways at `y <= front.y` that
/// still has room (`< 8`). Refuses to start a source in dry air at or below `SEA_LEVEL`.
fn find_pour_cell(world: &mut World, front: IVec3) -> Option<(IVec3, BlockId, u8)> {
    let b = world.block_anywhere_or_generate(front);
    if b != AIR && !LIQUID[b as usize] {
        return None;
    }
    // Trace straight down through air/partial water to check the dry-air below SEA_LEVEL safety guard.
    let mut c = front;
    while c.y > 0 && fillable(world.block_anywhere_or_generate(c - UP)) {
        c = c - UP;
    }
    if c.y <= SEA_LEVEL
        && world.block_anywhere_or_generate(c - UP) != WATER
        && !FACES.iter().any(|&f| f.y >= 0 && world.block_anywhere_or_generate(c + f) == WATER)
    {
        return None;
    }
    let m = wet_of(b);
    if m < 8 {
        return Some((front, b, m));
    }
    // Submerged outlet (`front == WATER`): hydraulic pressure pushes water into the lowest/nearest `< 8` cell
    // reachable in the basin within `POUR_REACH` sideways at `y <= front.y`.
    let mut best: Option<WaterCell> = None;
    let mut consider = |p: IVec3, pb: BlockId, dist: u32| {
        if (p.x - front.x).abs() > POUR_REACH || (p.z - front.z).abs() > POUR_REACH || p.y > front.y {
            return;
        }
        if !fillable(pb) {
            return;
        }
        let key = (p.y, dist, [p.x, p.y, p.z]);
        if best.is_none_or(|b| key < b.0) {
            best = Some((key, p, pb, wet_of(pb)));
        }
    };
    let mut visited = Vec::new();
    search(world, &[front], |p, pb, dist| {
        if (p.x - front.x).abs() <= POUR_REACH && (p.z - front.z).abs() <= POUR_REACH && p.y <= front.y {
            consider(p, pb, dist);
            visited.push((p, dist));
        }
    });
    for (p, dist) in visited {
        for f in FACES {
            let n = p + f;
            let nb = world.block_anywhere_or_generate(n);
            consider(n, nb, dist + 1);
        }
    }
    best.map(|(_, p, pb, pm)| (p, pb, pm))
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

/// Air or flowing water (`< 8` eighths): a cell with liquid room.
fn fillable(b: BlockId) -> bool {
    b == AIR || flow_level(b).is_some()
}
