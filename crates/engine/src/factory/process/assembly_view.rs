//! The assembler's work cell in motion (presentation only; `Look::Work` in `model.rs`). In each batch its two robot
//! arms take the recipe's inputs one at a time off the back half of the conveyor and set them on the fixture, taking
//! turns; a cartoon work cloud then hides the pile while the overhead hammer pounds, and the product comes out of it
//! and rides the conveyor out of the front. Everything is a pure function of the batch's share done (`batch_share`,
//! blended between ticks) and the time, read from the core and never written: nothing to save or sync. A batch
//! quicker than `MIN_CYCLE` is shown at that pace on the clock instead. A stalled machine freezes mid-move; an idle
//! one rests its arms.
//! Invariants (`assembly_view/tests.rs`): every input kind of every recipe is carried, an arm always reaches its
//! wrist, an item never jumps (conveyor to gripper to fixture), and the cloud is thick when the inputs become the
//! product. To change the choreography: the share constants and positions below.

use std::f64::consts::TAU;

use crate::block::tex;
use crate::item::ItemId;
use crate::math::Vec3;
use crate::recipes::MachineRecipe;

use super::super::render::{push_box, push_round, push_shape, Axis, Form};
use super::display::{draw_item, NEAR};
use super::model::{batch_seconds, batch_share, local};
use super::{Processor, Status};

#[cfg(test)]
mod tests;

/// Shares of a batch: the arms gather the inputs until `GATHER`, the cloud works until `CLOUD`, then the product rides
/// out. The inputs become the product halfway through the cloud (`SWAP`); the cloud swells and clears over `PUFF_IN`.
const GATHER: f64 = 0.6;
const CLOUD: f64 = 0.85;
const SWAP: f64 = (GATHER + CLOUD) / 2.0;
const PUFF_IN: f64 = 0.04;
/// Seconds the shortest show takes: a quicker batch is shown at this pace instead, on the clock.
const MIN_CYCLE: f64 = 3.0;
/// Picks a batch shows at most: one per unit while they fit, else one per input kind.
const MAX_PICKS: usize = 6;
/// Shares of a pick's window: the gripper closes on the input, then lets it go on the fixture.
const GRAB: f64 = 0.42;
const LET_GO: f64 = 0.92;
/// The arms (left, right): each waist's foot on the table (machine frame, front +z), the shoulder's height over it,
/// and the upper arm's and forearm's lengths.
const BASES: [[f64; 3]; 2] = [[-0.55, -0.06, -0.2], [0.55, -0.06, 0.05]];
const SHOULDER: f64 = 0.22;
const UPPER: f64 = 0.44;
const FORE: f64 = 0.4;
/// Where each arm's inputs wait (its lane of the conveyor's back half), where they come in, where the product leaves,
/// and the fixture's top. The product steps down off the fixture (`FIXTURE_HALF` wide each way) over `STEP`.
const PICKUP: [[f64; 3]; 2] = [[-0.1, 0.0, -0.5], [0.1, 0.0, -0.5]];
const ENTRY_Z: f64 = -0.94;
const EXIT_Z: f64 = 0.94;
const FIXTURE: [f64; 3] = [0.0, 0.08, 0.0];
const FIXTURE_HALF: f64 = 0.15;
const STEP: f64 = 0.1;
/// A wrist stands `GRIP` over the floor of what it grips and carries it `HOVER` higher.
const GRIP: f64 = 0.2;
const HOVER: f64 = 0.16;
/// The slots an input and the product fill (across and high).
const ITEM: f32 = 0.13;
const PRODUCT: f32 = 0.2;
/// The hammer hangs from the carriage's foot; its head's height raised, its deepest stroke, its pace (radians a
/// second of the stroke's sine).
const CARRIAGE_FOOT: f64 = 0.86;
const HAMMER_UP: f64 = 0.59;
const HAMMER_STROKE: f64 = 0.28;
const HAMMER_PACE: f64 = 14.0;

const ROBOT: [u16; 3] = [tex::MAT_EMBER; 3];
const JOINT: [u16; 3] = [tex::MAT_GUNMETAL; 3];

/// Draws `p`'s work cell, its footprint's centre at camera-relative `centre`, the machine turned by `yaw`.
pub(super) fn draw(p: &Processor, out: &mut Vec<f32>, centre: Vec3, yaw: f32, time: f64) {
    let at = |v: [f64; 3]| local(centre, yaw, v);
    let recipe = p.batch_recipe();
    let plan = recipe.map(plan);
    // The shaking and pounding run on the clock, so they stop where they are when the machine does.
    let clock = if p.status == Status::Working { time } else { 0.0 };
    let t = match batch_seconds(p) {
        // Too quick to follow: one show every `MIN_CYCLE` stands for the batches in it.
        Some(s) if s < MIN_CYCLE && clock > 0.0 => (clock / MIN_CYCLE).fract(),
        _ => batch_share(p, time).unwrap_or(0.0),
    };
    for arm in 0..2 {
        let (wrist, holding) = match &plan {
            Some(plan) => {
                (wrist(plan, arm, t, clock), plan.of(arm).any(|k| input_at(plan, k, t, clock).is_some_and(|i| i.1)))
            }
            None => (rest(arm), false),
        };
        draw_arm(out, &at, arm, wrist, holding);
    }
    let puff = if plan.is_some() { cloud(t) } else { 0.0 };
    let head = HAMMER_UP - puff * (clock * HAMMER_PACE).sin().abs() * HAMMER_STROKE;
    push_box(out, at([0.0, head, 0.0]), yaw, [0.2, 0.08, 0.2], 0.0, [tex::STEEL; 3], false);
    let rod = CARRIAGE_FOOT - (head + 0.04);
    push_box(out, at([0.0, head + 0.04 + rod / 2.0, 0.0]), yaw, [0.08, rod as f32, 0.08], 0.0, [tex::DRILL; 3], false);
    let (Some(plan), Some(recipe)) = (plan, recipe) else { return };
    if centre.length() > NEAR {
        return;
    }
    for pick in &plan.picks[..plan.n] {
        if let Some((floor, _)) = input_at(&plan, pick, t, clock) {
            draw_item(out, pick.item, at(floor), yaw, [ITEM, ITEM], false);
        }
    }
    if let Some(floor) = product_at(t) {
        draw_item(out, recipe.main().0, at(floor), yaw, [PRODUCT, PRODUCT], false);
    }
    if puff > 0.0 {
        draw_cloud(out, &at, puff, clock);
    }
}

/// One input carried in a batch: what, by which arm, its window of the batch (start and length, shares) and where
/// it is set down on the fixture.
#[derive(Clone, Copy)]
struct Pick {
    item: ItemId,
    arm: usize,
    start: f64,
    len: f64,
    spot: [f64; 3],
}

/// A batch's picks, in order: the arms take turns, both at work at once.
struct Plan {
    picks: [Pick; MAX_PICKS],
    n: usize,
}

impl Plan {
    fn of(&self, arm: usize) -> impl Iterator<Item = &Pick> {
        self.picks[..self.n].iter().filter(move |k| k.arm == arm)
    }
}

/// What a batch of `r` carries: every unit while they fit in `MAX_PICKS`, else one of each kind.
fn plan(r: &MachineRecipe) -> Plan {
    let units: u32 = r.inputs.iter().map(|&(_, n)| n).sum();
    let each = units as usize <= MAX_PICKS;
    let blank = Pick { item: ItemId(0), arm: 0, start: 0.0, len: 0.0, spot: FIXTURE };
    let mut plan = Plan { picks: [blank; MAX_PICKS], n: 0 };
    for &(item, count) in r.inputs {
        for _ in 0..if each { count } else { 1 } {
            if plan.n < MAX_PICKS {
                plan.picks[plan.n].item = item;
                plan.n += 1;
            }
        }
    }
    let len = GATHER / plan.n.div_ceil(2).max(1) as f64;
    for (i, pick) in plan.picks[..plan.n].iter_mut().enumerate() {
        // The first sits in the middle of the fixture, the rest in a small heap round it.
        let (a, r) = (i as f64 * 2.4, if i == 0 { 0.0 } else { 0.07 });
        pick.arm = i % 2;
        (pick.start, pick.len) = ((i / 2) as f64 * len, len);
        pick.spot = [FIXTURE[0] + a.cos() * r, FIXTURE[1], FIXTURE[2] + a.sin() * r];
    }
    plan
}

/// Where `arm`'s wrist rests between batches: over the middle of its reach.
fn rest(arm: usize) -> [f64; 3] {
    let mid = lerp(PICKUP[arm], FIXTURE, 0.5);
    [mid[0], mid[1] + GRIP + HOVER + 0.08, mid[2]]
}

/// Where `arm`'s wrist is at batch share `t`; `clock` (seconds, 0 when stalled) shakes it as it works by the cloud.
fn wrist(plan: &Plan, arm: usize, t: f64, clock: f64) -> [f64; 3] {
    let up = |q: [f64; 3], h: f64| [q[0], q[1] + h, q[2]];
    let mut from = rest(arm);
    for k in plan.of(arm) {
        let (over_pick, at_pick) = (up(PICKUP[arm], GRIP + HOVER), up(PICKUP[arm], GRIP));
        let (over_spot, at_spot) = (up(k.spot, GRIP + HOVER), up(k.spot, GRIP));
        if t < k.start + k.len {
            let keys = [
                (0.0, from),
                (0.3, over_pick),
                (GRAB, at_pick),
                (0.55, over_pick),
                (0.82, over_spot),
                (LET_GO, at_spot),
            ];
            return path(
                &[keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], (1.0, over_spot)],
                (t - k.start) / k.len,
            );
        }
        from = over_spot;
    }
    // Every input set down: work beside the cloud, then back to rest for the next batch.
    let side = if arm == 0 { -1.0 } else { 1.0 };
    let phase = clock * 19.0 + arm as f64 * 1.7;
    let work = [FIXTURE[0] + side * 0.22 + 0.03 * phase.sin(), FIXTURE[1] + 0.3 + 0.03 * phase.cos(), FIXTURE[2]];
    path(&[(GATHER, from), (GATHER + PUFF_IN, work), (CLOUD, work), (1.0, rest(arm))], t)
}

/// Where the floor of `pick`'s input is at share `t`, and whether a gripper holds it: None before it comes in on the
/// conveyor and once the cloud has made it into the product.
fn input_at(plan: &Plan, pick: &Pick, t: f64, clock: f64) -> Option<([f64; 3], bool)> {
    let appear = (pick.start - 0.5 * pick.len).max(0.0);
    let arrive = pick.start + 0.3 * pick.len;
    if t < appear || t >= SWAP {
        return None;
    }
    let lane = PICKUP[pick.arm];
    let f = (t - pick.start) / pick.len;
    Some(if t < arrive {
        let s = (t - appear) / (arrive - appear);
        ([lane[0], lane[1], ENTRY_Z + (lane[2] - ENTRY_Z) * s], false)
    } else if f < GRAB {
        (lane, false)
    } else if f < LET_GO {
        let w = wrist(plan, pick.arm, t, clock);
        ([w[0], w[1] - GRIP, w[2]], true)
    } else {
        (pick.spot, false)
    })
}

/// Where the product's floor is at share `t`: on the fixture from the swap, under the cloud, then out along the belt.
fn product_at(t: f64) -> Option<[f64; 3]> {
    let leave = CLOUD + 0.02;
    if t < SWAP {
        return None;
    }
    let z = FIXTURE[2] + (EXIT_Z - FIXTURE[2]) * ((t - leave) / (1.0 - leave)).max(0.0);
    let y = FIXTURE[1] * ((FIXTURE_HALF + STEP - z) / STEP).clamp(0.0, 1.0);
    Some([FIXTURE[0], y, z])
}

/// How thick the work cloud is at share `t` (0 to 1).
fn cloud(t: f64) -> f64 {
    if !(GATHER..CLOUD).contains(&t) {
        return 0.0;
    }
    ((t - GATHER) / PUFF_IN).min((CLOUD - t) / PUFF_IN).min(1.0)
}

/// The elbow of an arm whose shoulder is at `s` reaching for wrist `w` (elbow up), and the wrist it reaches: `w`, or
/// the nearest point the arm can reach.
fn elbow(s: [f64; 3], w: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let (dx, dz, h) = (w[0] - s[0], w[2] - s[2], w[1] - s[1]);
    let d = dx.hypot(dz);
    let r = d.hypot(h).clamp((UPPER - FORE).abs() + 1e-3, UPPER + FORE - 1e-3);
    let (a, b) = (h.atan2(d), ((UPPER * UPPER + r * r - FORE * FORE) / (2.0 * UPPER * r)).clamp(-1.0, 1.0).acos());
    let (ux, uz) = if d > 1e-9 { (dx / d, dz / d) } else { (1.0, 0.0) };
    let point =
        |len: f64, angle: f64| [s[0] + len * angle.cos() * ux, s[1] + len * angle.sin(), s[2] + len * angle.cos() * uz];
    (point(UPPER, a + b), point(r, a))
}

/// An arm reaching for `wrist` (machine frame): its waist, shoulder and elbow joints, the two limbs and a gripper whose
/// fingers close on what it holds.
fn draw_arm(out: &mut Vec<f32>, at: &impl Fn([f64; 3]) -> Vec3, arm: usize, wrist: [f64; 3], holding: bool) {
    let base = BASES[arm];
    let shoulder = [base[0], base[1] + SHOULDER, base[2]];
    let (elbow, wrist) = elbow(shoulder, wrist);
    let (s, e, w) = (at(shoulder), at(elbow), at(wrist));
    // Turned to face the wrist: a box's own x then lies across the arm's plane.
    let facing = (s.x - w.x).atan2(w.z - s.z) as f32;
    push_round(out, at([base[0], base[1] + 0.08, base[2]]), facing, Axis::Up, [0.24, 0.16, 0.24], 1.0, ROBOT, false);
    push_round(out, s, facing, Axis::X, [0.22, 0.13, 0.13], 1.0, JOINT, false);
    limb(out, s, e, 0.11);
    push_round(out, e, facing, Axis::X, [0.18, 0.12, 0.12], 1.0, JOINT, false);
    limb(out, e, w, 0.09);
    push_box(out, w - Vec3::new(0.0, 0.04, 0.0), facing, [0.12, 0.05, 0.08], 0.0, JOINT, false);
    let across = Vec3::new(f64::from(facing.cos()), 0.0, f64::from(facing.sin()));
    let gap = if holding { 0.08 } else { 0.11 };
    for side in [-1.0, 1.0] {
        let finger = w + across * (side * gap) - Vec3::new(0.0, 0.11, 0.0);
        push_box(out, finger, facing, [0.02, 0.1, 0.05], 0.0, JOINT, false);
    }
}

/// A limb `thick` across from `a` to `b` (camera-relative): a box leaned so its length runs along them.
fn limb(out: &mut Vec<f32>, a: Vec3, b: Vec3, thick: f32) {
    let d = if b.y >= a.y { b - a } else { a - b };
    let flat = d.x.hypot(d.z);
    let yaw = if flat > 1e-9 { (-d.x).atan2(d.z) as f32 } else { 0.0 };
    let pitch = flat.atan2(d.y) as f32;
    push_shape(out, Form::Lean(pitch), (a + b) * 0.5, yaw, [thick, d.length() as f32, thick], ROBOT, false);
}

/// The cartoon work cloud over the fixture, `thick` (0 to 1): puffs swirling round a big one, a few sparks flying.
fn draw_cloud(out: &mut Vec<f32>, at: &impl Fn([f64; 3]) -> Vec3, thick: f64, clock: f64) {
    let c = [FIXTURE[0], FIXTURE[1] + 0.14, FIXTURE[2]];
    let mut puff = |v: [f64; 3], size: f64| {
        let p = at(v);
        push_box(out, p, p.x.atan2(p.z) as f32, [(size * thick) as f32; 3], 0.0, [tex::SMOKE; 3], false);
    };
    puff([c[0], c[1] + 0.04, c[2]], 0.28);
    for k in 0..6 {
        let a = k as f64 * TAU / 6.0 + clock * 2.5;
        let bob = (clock * 7.0 + k as f64 * 1.9).sin();
        let lift = if k % 2 == 0 { 0.05 } else { -0.02 };
        puff([c[0] + a.cos() * 0.14, c[1] + lift + 0.03 * bob, c[2] + a.sin() * 0.14], 0.18 + 0.03 * bob);
    }
    for k in 0..3 {
        let beat = clock * 5.0 + k as f64 * 0.37;
        if beat.fract() < 0.35 {
            let a = beat.floor() * 2.3 + k as f64;
            let p = at([c[0] + a.cos() * 0.24, c[1] + 0.12 + 0.05 * (a * 1.3).sin(), c[2] + a.sin() * 0.24]);
            push_box(out, p, 0.0, [(0.04 * thick) as f32; 3], 0.0, [tex::LAMP_YELLOW; 3], false);
        }
    }
}

/// Eases `keys` (shares and points, in order) at share `t`: still before the first and after the last.
fn path(keys: &[(f64, [f64; 3])], t: f64) -> [f64; 3] {
    let mut prev = keys[0];
    if t <= prev.0 {
        return prev.1;
    }
    for &key in &keys[1..] {
        if t < key.0 {
            let s = ((t - prev.0) / (key.0 - prev.0)).clamp(0.0, 1.0);
            return lerp(prev.1, key.1, s * s * (3.0 - 2.0 * s));
        }
        prev = key;
    }
    prev.1
}

fn lerp(a: [f64; 3], b: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s, a[2] + (b[2] - a[2]) * s]
}
