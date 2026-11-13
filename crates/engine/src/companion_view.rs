//! Drawing every player's personal drone (presentation only; its errands are core state: `helpers/drone/`). It hovers
//! off the player's right shoulder, or flies its trip: eased legs that climb over what lies between, a dip over each
//! box while it loads, the item it carries hanging under it on a line.
//!
//! The motion is a spring that tracks the target with its velocity fed forward (no lag at cruise), and the model is a
//! quad-rotor posed as a real one flies: its rotors push along its up, so up points along acceleration plus gravity
//! plus drag ([`tilt`]), turning toward where it goes; the props of each diagonal pair spin the same way, the pairs
//! opposite, faster as it speeds. Red (left) and green (right) lights on the front arms, a white strobe at the back.
//! Never feeds the core. To change the look: `MODEL` (parts in the drone's own frame) and `push_flyer`.

use std::f64::consts::{FRAC_PI_4, PI, TAU};

use crate::block::tex;
use crate::entities::push_item_model;
use crate::factory::{push_box, push_framed, Framed};
use crate::helpers::drone::{Trip, DWELL};
use crate::item::{self, ItemId, PERSONAL_DRONE};
use crate::math::{IVec3, Vec3};
use crate::player::Player;
use crate::{Game, TICK};

#[cfg(test)]
mod tests;

/// Drones further than this from the eye are not drawn.
const DRAW_RANGE: f64 = 160.0;
/// The spring's stiffness while it hovers at the shoulder and while it flies a trip, in 1/s.
const HOVER_OMEGA: f64 = 7.0;
const TRIP_OMEGA: f64 = 14.0;
/// Further than this from its target it jumps there (a teleport, a world loaded).
const SNAP: f64 = 48.0;
/// Hovering height over a box's top, and how far it dips while it loads.
const HOVER: f64 = 0.75;
const DIP: f64 = 0.35;
/// Gravity and the drag that tilts it into its speed (blocks, seconds), and the most it ever leans (radians).
const GRAVITY: f64 = 9.8;
const DRAG: f64 = 0.9;
const MAX_LEAN: f64 = 0.6;
/// Rotor speed hovering, and more per block a second of speed (radians a second; slow enough not to strobe).
const SPIN: f64 = 26.0;
const SPIN_PER_SPEED: f64 = 0.6;

/// One drone as drawn: where it is, how it moves and how it is turned (never saved).
#[derive(Clone, Copy, Default)]
pub struct Flyer {
    pos: Vec3,
    vel: Vec3,
    up: Vec3,
    heading: f64,
    spin: f64,
    time: f64,
    live: bool,
}

/// Every player's drone, indexed by `PlayerId`.
#[derive(Default)]
pub struct Companions {
    list: Vec<Flyer>,
}

impl Game {
    /// Moves and draws every player's personal drone near the eye; `time` is game seconds at this frame.
    pub(crate) fn write_companions(&mut self, eye: Vec3, alpha: f64, time: f64) {
        let local_eye = self.prev_eye + (self.body().eye() - self.prev_eye) * alpha;
        let ticks = self.sim.tick as f64 + alpha;
        for (i, core) in self.sim.players.iter().enumerate() {
            let Some(core) = core.as_ref().filter(|c| c.inventory.count(PERSONAL_DRONE) > 0) else { continue };
            let Some(body) = self.bodies.get(i).and_then(Option::as_ref) else { continue };
            let head = if i == self.local.0 as usize { local_eye } else { body.eye() };
            if self.companions.list.len() <= i {
                self.companions.list.resize(i + 1, Flyer::default());
            }
            let f = &mut self.companions.list[i];
            let home = |t: f64| shoulder(body, head, t);
            let trip = core.helpers.drone.trip.as_ref();
            let target = |t: f64| match trip {
                Some(trip) => flight(trip, ticks + (t - time) / TICK, home(t)),
                None => (home(t), None),
            };
            let carried = f.fly(time, target, trip.is_some());
            if (f.pos - eye).length() <= DRAW_RANGE {
                push_flyer(&mut self.instances, f, eye, carried, time);
            }
        }
    }
}

impl Flyer {
    /// Moves toward `target(t)` (where it should be at game time `t`, and what it carries) up to `time`; what it
    /// carries now.
    fn fly(&mut self, time: f64, target: impl Fn(f64) -> (Vec3, Option<ItemId>), trip: bool) -> Option<ItemId> {
        let (goal, carried) = target(time);
        if !self.live || (goal - self.pos).length() > SNAP || time < self.time {
            *self = Flyer { pos: goal, up: Vec3::new(0.0, 1.0, 0.0), time, live: true, ..*self };
            return carried;
        }
        let dt = (time - self.time).min(0.1);
        self.time = time;
        let omega = if trip { TRIP_OMEGA } else { HOVER_OMEGA };
        let steps = (dt * 240.0).ceil().max(1.0);
        let h = dt / steps;
        let mut accel = Vec3::ZERO;
        for k in 0..steps as u32 {
            let t = time - dt + h * f64::from(k + 1);
            let (p, _) = target(t);
            let (q, _) = target(t - 1.0 / 60.0);
            let want = (p - q) * 60.0;
            accel = (p - self.pos) * (omega * omega) + (want - self.vel) * (2.0 * omega);
            self.vel += accel * h;
            self.pos += self.vel * h;
        }
        // Thrust along up: acceleration, gravity and drag; eased so a jolt does not flick it.
        let lean = tilt(accel, self.vel);
        let k = 1.0 - (-dt * 8.0).exp();
        self.up = unit(self.up + (lean - self.up) * k);
        let flat = Vec3::new(self.vel.x, 0.0, self.vel.z);
        if flat.length() > 1.5 {
            let want = flat.z.atan2(flat.x);
            let turn = wrap(want - self.heading);
            self.heading += turn.clamp(-5.0 * dt, 5.0 * dt);
        }
        self.spin = (self.spin + dt * (SPIN + SPIN_PER_SPEED * self.vel.length())) % TAU;
        carried
    }
}

/// Where a drone points its up when it accelerates by `accel` at `vel`: thrust balances gravity, drag and the push,
/// leaning at most [`MAX_LEAN`].
pub fn tilt(accel: Vec3, vel: Vec3) -> Vec3 {
    let mut side = Vec3::new(accel.x + DRAG * vel.x, 0.0, accel.z + DRAG * vel.z);
    let lift = (GRAVITY + accel.y + DRAG * vel.y).max(GRAVITY * 0.3);
    let most = lift * MAX_LEAN.tan();
    if side.length() > most {
        side = side * (most / side.length());
    }
    unit(Vec3::new(side.x, lift, side.z))
}

/// The spot off the player's right shoulder, a little behind (out of view until they turn), swaying slowly.
fn shoulder(body: &Player, head: Vec3, t: f64) -> Vec3 {
    let (sy, cy) = body.yaw.sin_cos();
    // The body's own frame (`player.rs`): forward is (sin yaw, 0, -cos yaw).
    let ahead = Vec3::new(sy, 0.0, -cy);
    let right = Vec3::new(cy, 0.0, sy);
    let sway = Vec3::new(0.0, 0.08 * (t * 1.3).sin(), 0.0) + right * (0.06 * (t * 0.7).sin());
    head - ahead * 0.5 + right * 1.0 + Vec3::new(0.0, 0.2, 0.0) + sway
}

/// Where the drone is `t` ticks into `trip` and what hangs under it: lifting off from `home`, over each stop in turn
/// (hovering there through its dwell), back to `home`.
pub fn flight(trip: &Trip, t: f64, home: Vec3) -> (Vec3, Option<ItemId>) {
    let over = |c: IVec3| c.as_vec3() + Vec3::new(0.5, 1.0 + HOVER, 0.5);
    let half = f64::from(DWELL) / 2.0;
    let (mut from, mut left) = (home, 0.0);
    for s in &trip.stops {
        let (arrive, go) = (f64::from(s.tick) - half, f64::from(s.tick) + half);
        let to = over(s.at);
        if t < arrive {
            return (leg(from, to, (t - left) / (arrive - left).max(1.0)), carried(trip, t));
        }
        if t < go {
            let u = (t - arrive) / (go - arrive);
            return (to - Vec3::new(0.0, DIP * (u * PI).sin(), 0.0), carried(trip, t));
        }
        (from, left) = (to, go);
    }
    let s = (t - left) / (f64::from(trip.total) - left).max(1.0);
    (leg(from, home, s), carried(trip, t))
}

/// A leg from `a` to `b` at share `s`: eased in and out, arcing up over the way (more on a longer leg).
fn leg(a: Vec3, b: Vec3, s: f64) -> Vec3 {
    let s = s.clamp(0.0, 1.0);
    let e = s * s * (3.0 - 2.0 * s);
    let climb = ((b - a).length() * 0.15).clamp(0.5, 5.0);
    a + (b - a) * e + Vec3::new(0.0, climb * (s * PI).sin(), 0.0)
}

/// The item under the drone at `t`: on a fetch, what the last stop passed gave; on a store, what the next stop takes.
fn carried(trip: &Trip, t: f64) -> Option<ItemId> {
    let mut stops = trip.stops.iter();
    let s = if trip.store { stops.find(|s| f64::from(s.tick) > t) } else { stops.rfind(|s| f64::from(s.tick) <= t) };
    s.map(|s| s.item)
}

fn wrap(a: f64) -> f64 {
    (a + PI).rem_euclid(TAU) - PI
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

fn unit(v: Vec3) -> Vec3 {
    v * (1.0 / v.length().max(1e-9))
}

/// What a part of the model is: a box or an upright cylinder.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Box,
    Round,
}

/// One part in the drone's own frame (x to its left, y up, z forward; blocks): centre, size, turn about up, layer, kind.
type Piece = ([f64; 3], [f32; 3], f64, u16, Kind);

/// Where the four motors sit (x, z) and which way each prop turns.
const ARM: f64 = 0.215;
const MOTORS: [(f64, f64, f64); 4] = [(ARM, ARM, 1.0), (-ARM, ARM, -1.0), (ARM, -ARM, -1.0), (-ARM, -ARM, 1.0)];

const SHELL: u16 = tex::DRONE_SHELL;
const ACCENT: u16 = tex::MAT_AETHER;
const CARBON: u16 = tex::DRONE_CARBON;
const DARK: u16 = tex::MAT_GUNMETAL;
const METAL: u16 = tex::MAT_METAL;
const BLADE: u16 = tex::MAT_RUBBER;

/// The body: a shell over a dark belly, a battery on its back, skids, the camera on its gimbal under the nose and a
/// lamp in the belly. Arms, motors, props and lights come from `MOTORS`.
const MODEL: [Piece; 15] = [
    ([0.0, 0.0, 0.0], [0.2, 0.075, 0.3], 0.0, SHELL, Kind::Box),
    ([0.0, -0.045, 0.0], [0.17, 0.02, 0.26], 0.0, DARK, Kind::Box),
    ([0.0, 0.05, 0.02], [0.15, 0.03, 0.2], 0.0, ACCENT, Kind::Box),
    ([0.0, 0.07, -0.05], [0.11, 0.03, 0.11], 0.0, DARK, Kind::Box),
    ([0.0, 0.088, -0.05], [0.04, 0.006, 0.08], 0.0, tex::LAMP_GREEN, Kind::Box),
    ([0.0, 0.0, 0.155], [0.12, 0.05, 0.02], 0.0, SHELL, Kind::Box),
    ([0.0, -0.075, 0.12], [0.03, 0.04, 0.03], 0.0, DARK, Kind::Box),
    ([0.0, -0.1, 0.13], [0.055, 0.045, 0.055], 0.0, DARK, Kind::Round),
    ([0.0, -0.1, 0.16], [0.03, 0.03, 0.012], 0.0, tex::CAB_GLASS, Kind::Box),
    ([0.0, -0.058, 0.0], [0.04, 0.008, 0.04], 0.0, tex::LAMP, Kind::Box),
    ([0.07, -0.09, 0.07], [0.012, 0.08, 0.012], 0.0, DARK, Kind::Box),
    ([-0.07, -0.09, 0.07], [0.012, 0.08, 0.012], 0.0, DARK, Kind::Box),
    ([0.07, -0.09, -0.07], [0.012, 0.08, 0.012], 0.0, DARK, Kind::Box),
    ([-0.07, -0.09, -0.07], [0.012, 0.08, 0.012], 0.0, DARK, Kind::Box),
    ([0.0, -0.135, 0.0], [0.18, 0.012, 0.012], 0.0, DARK, Kind::Box),
];

/// Pushes one drone at `f` (camera-relative to `eye`), with `carried` hanging under it.
fn push_flyer(out: &mut Vec<f32>, f: &Flyer, eye: Vec3, carried: Option<ItemId>, time: f64) {
    let up = f.up;
    let ahead = Vec3::new(f.heading.cos(), 0.0, f.heading.sin());
    let fwd = unit(ahead - up * dot(ahead, up));
    let left = cross(up, fwd);
    let at = f.pos - eye;
    let mut put = |(c, size, yaw, layer, kind): Piece| {
        let centre = at + left * c[0] + up * c[1] + fwd * c[2];
        let (s, co) = yaw.sin_cos();
        let axes = [left * co + fwd * s, up, fwd * co - left * s];
        let look = Framed { taper: (kind == Kind::Round).then_some(1.0), slide: None };
        push_framed(out, centre, axes, size, [layer; 3], look);
    };
    MODEL.iter().for_each(|&p| put(p));
    let strobe = (time * 0.8).fract() < 0.08;
    for (i, &(x, z, turn)) in MOTORS.iter().enumerate() {
        let diag = z.atan2(x);
        // The arm from the body out to the motor, and the motor with its bell and prop on top.
        put(([x * 0.55, 0.005, z * 0.55], [0.2, 0.026, 0.034], diag, CARBON, Kind::Box));
        put(([x, 0.015, z], [0.055, 0.05, 0.055], 0.0, DARK, Kind::Round));
        put(([x, 0.048, z], [0.045, 0.018, 0.045], 0.0, METAL, Kind::Round));
        let spin = f.spin * turn + i as f64 * FRAC_PI_4;
        for b in 0..3 {
            let yaw = spin + f64::from(b) * TAU / 3.0;
            let (s, c) = yaw.sin_cos();
            let mid = [x + 0.075 * c, 0.062, z + 0.075 * s];
            put((mid, [0.15, 0.004, 0.032], yaw, BLADE, Kind::Box));
        }
        let light = match i {
            0 => Some(tex::LAMP_RED),
            1 => Some(tex::LAMP_GREEN),
            _ if strobe => Some(tex::LAMP),
            _ => None,
        };
        if let Some(layer) = light {
            put(([x, -0.02, z], [0.022, 0.014, 0.022], 0.0, layer, Kind::Box));
        }
    }
    if let Some(item) = carried {
        // A line down from the belly to the load, which hangs level and turns slowly.
        put(([0.0, -0.2, 0.0], [0.006, 0.12, 0.006], 0.0, DARK, Kind::Box));
        let (below, yaw) = (at - up * 0.36, (time * 0.6) as f32);
        if !push_item_model(out, item, below, yaw, 0.22) {
            let def = item::def(item).map_or(([0.6; 3], [tex::FRAME; 3]), |d| (d.size, d.tex));
            push_box(out, below, yaw, def.0.map(|s| s * 0.22), 0.0, def.1, false);
        }
    }
}
