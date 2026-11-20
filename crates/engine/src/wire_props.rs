//! Power wires as drawn, lifted over the ground: where the sagging wire between two poles would run through ground
//! (a hill between them, one standing much lower), a cosmetic prop stands on the ground under it and the wire hangs
//! over the prop instead. Props cost nothing and do nothing: no block, no grid, no save, nothing to aim at. The
//! factory lists the wires (`Factory::for_each_wire`); this draws every one.
//!
//! `route` is pure: the props of one span, from a ground test. Spans are routed against the loaded blocks (the render
//! cache), cached by their ends, at most `NEW_PER_FRAME` new ones a frame (the rest drawn straight until then), and
//! each re-checked after `RECHECK_FRAMES` so digging or filling under a wire moves its props.
//! Only spans between two real poles get props: a cable hangs into its shaft on purpose, a machine's lead is short.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::block::{self, tex};
use crate::chunk::CHUNK_SIZE;
use crate::factory::{self, push_box, sag_point, POLE_WIRE_Y};
use crate::math::{IVec3, Vec3};
use crate::world::{chunk_of, World};

/// Spans shorter than this across the ground get no props.
const MIN_RUN: f64 = 2.5;
/// Ground this near either end (the poles' own cells and footings) doesn't count.
const END_CLEAR: f64 = 1.0;
/// How far under the wire's centre a block counts as touching it.
const CLEARANCE: f64 = 0.15;
/// How many times a span is split at most (up to 2^n - 1 props).
const MAX_DEPTH: u32 = 5;
/// The furthest a hill is climbed from where the wire meets it, to find its surface.
const MAX_CLIMB: i32 = 48;
const NEW_PER_FRAME: usize = 24;
const RECHECK_FRAMES: u32 = 120;
const RECHECKS_PER_FRAME: usize = 8;
/// Most pole-to-pole spans scanned per frame while looking for due rechecks (keeps huge bases O(1) per frame).
const SCAN_PER_FRAME: usize = 128;

/// The tops of the props a wire from `a` to `b` (world points) needs so that it touches no ground, in order from `a`.
/// `ground` says whether a cell is ground a wire must not pass through.
pub fn route(a: Vec3, b: Vec3, ground: &impl Fn(IVec3) -> bool) -> Vec<Vec3> {
    let mut out = Vec::new();
    split(a, b, ground, MAX_DEPTH, &mut out);
    out
}

/// The cached props of every span, by its ends, plus the wire span lists cached from `Factory::layout_gen`.
pub(crate) struct WireProps {
    routes: FxHashMap<[i32; 6], Route>,
    pole_spans: Vec<(Vec3, Vec3, u16)>,
    leads: Vec<(Vec3, Vec3, u16)>,
    layout_gen: u64,
    version: u64,
    cursor: usize,
    frame: u32,
}

impl Default for WireProps {
    fn default() -> Self {
        Self {
            routes: FxHashMap::default(),
            pole_spans: Vec::new(),
            leads: Vec::new(),
            layout_gen: u64::MAX,
            version: 1,
            cursor: 0,
            frame: 0,
        }
    }
}

struct Route {
    props: Vec<Vec3>,
    /// The frame it was last routed.
    routed: u32,
}

impl WireProps {
    /// Monotonic version bumped only when wire geometry or props actually change.
    pub(crate) fn version(&self) -> u64 {
        self.version
    }

    /// Drops all cached wire lists and routes (after a co-op snapshot resync).
    pub(crate) fn reset(&mut self) {
        self.routes.clear();
        self.pole_spans.clear();
        self.leads.clear();
        self.layout_gen = u64::MAX;
        self.cursor = 0;
        self.version = self.version.wrapping_add(1);
    }

    /// Syncs span lists when the factory layout changes and incrementally routes/rechecks props near `eye`.
    pub(crate) fn step(&mut self, world: &World, factory: &factory::Factory, eye: Vec3) {
        self.frame = self.frame.wrapping_add(1);
        if factory.layout_gen != self.layout_gen {
            self.layout_gen = factory.layout_gen;
            self.pole_spans.clear();
            self.leads.clear();
            let mut live = FxHashSet::default();
            factory.for_each_wire(Vec3::ZERO, f64::INFINITY, |a, b, layer, poles| {
                if poles {
                    self.pole_spans.push((a, b, layer));
                    live.insert(key(a, b));
                } else {
                    self.leads.push((a, b, layer));
                }
            });
            self.routes.retain(|k, _| live.contains(k));
            self.cursor = 0;
            self.version = self.version.wrapping_add(1);
        }
        let n = self.pole_spans.len();
        if n == 0 {
            return;
        }
        let (frame, range2) = (self.frame, world.view_distance().powi(2));
        let (mut fresh, mut rechecks) = (NEW_PER_FRAME, RECHECKS_PER_FRAME);
        let ground = |c: IVec3| world.get_block(c).is_some_and(is_ground);
        let mut changed = false;
        let scan = n.min(SCAN_PER_FRAME);
        for _ in 0..scan {
            let idx = self.cursor % n;
            self.cursor = (idx + 1) % n;
            let (a, b, _) = self.pole_spans[idx];
            let mid = (a + b) * 0.5 - eye;
            if mid.x * mid.x + mid.y * mid.y + mid.z * mid.z > range2 {
                continue;
            }
            let k = key(a, b);
            match self.routes.get_mut(&k) {
                Some(r) => {
                    if frame.wrapping_sub(r.routed) > RECHECK_FRAMES && rechecks > 0 {
                        rechecks -= 1;
                        let next = route(a, b, &ground);
                        if next != r.props {
                            r.props = next;
                            changed = true;
                        }
                        r.routed = frame;
                    }
                }
                None if fresh > 0 => {
                    fresh -= 1;
                    let props = route(a, b, &ground);
                    if !props.is_empty() {
                        changed = true;
                    }
                    self.routes.insert(k, Route { props, routed: frame });
                }
                None => {}
            }
            if fresh == 0 && rechecks == 0 {
                break;
            }
        }
        if changed {
            self.version = self.version.wrapping_add(1);
        }
    }

    /// Emits all cached power wires and their props into per-chunk `wire_boxes` (static cache).
    pub(crate) fn emit_binned(&self, chunks: &mut FxHashMap<IVec3, factory::ChunkMembers>) {
        let mut scratch = Vec::new();
        for &(a, b, layer) in &self.pole_spans {
            let base = (chunk_of(a.floor()) * CHUNK_SIZE).as_vec3();
            scratch.clear();
            let mut from = a;
            if let Some(r) = self.routes.get(&key(a, b)) {
                for &top in &r.props {
                    factory::wire(&mut scratch, from - base, top - base, layer);
                    push_prop(&mut scratch, top - base);
                    from = top;
                }
            }
            factory::wire(&mut scratch, from - base, b - base, layer);
            factory::bin_boxes(&scratch, base, chunks, true);
        }
        for &(a, b, layer) in &self.leads {
            let base = (chunk_of(a.floor()) * CHUNK_SIZE).as_vec3();
            scratch.clear();
            factory::wire(&mut scratch, a - base, b - base, layer);
            factory::bin_boxes(&scratch, base, chunks, true);
        }
    }
}

/// Whether a block is ground a wire must clear: solid, and neither a tree (wires run through crowns as they do through
/// the air) nor a machine or its parts (a lead runs to them on purpose).
fn is_ground(b: block::BlockId) -> bool {
    block::SOLID[b as usize] && !block::is_tree(b) && b != block::MACHINE_PART && factory::machine(b).is_none()
}

/// A span's ends, to the nearest sixteenth of a block, as its cache key.
fn key(a: Vec3, b: Vec3) -> [i32; 6] {
    let q = |v: f64| (v * 16.0).round() as i32;
    [q(a.x), q(a.y), q(a.z), q(b.x), q(b.y), q(b.z)]
}

/// Adds the props of the span `a`..`b` to `out` in order, splitting it at the worst dip until it clears.
fn split(a: Vec3, b: Vec3, ground: &impl Fn(IVec3) -> bool, depth: u32, out: &mut Vec<Vec3>) {
    if depth == 0 {
        return;
    }
    let Some(top) = worst_dip(a, b, ground) else { return };
    split(a, top, ground, depth - 1, out);
    out.push(top);
    split(top, b, ground, depth - 1, out);
}

/// Where a prop must stand for the span `a`..`b`: under the point that is deepest in the ground, its top a pole's
/// height above that column's surface. `None` when the span clears the ground.
fn worst_dip(a: Vec3, b: Vec3, ground: &impl Fn(IVec3) -> bool) -> Option<Vec3> {
    let flat = |p: Vec3, q: Vec3| ((p.x - q.x).powi(2) + (p.z - q.z).powi(2)).sqrt();
    let run = flat(a, b);
    if run < MIN_RUN {
        return None;
    }
    let d = b - a;
    let samples = ((d.x * d.x + d.y * d.y + d.z * d.z).sqrt() * 3.0).ceil() as usize;
    let mut best: Option<(f64, IVec3)> = None;
    for i in 1..samples {
        let p = sag_point(a, b, i as f64 / samples as f64);
        if flat(p, a) < END_CLEAR || flat(p, b) < END_CLEAR {
            continue;
        }
        let cell = (p - Vec3::new(0.0, CLEARANCE, 0.0)).floor();
        if !ground(cell) {
            continue;
        }
        let mut surface = cell.y + 1;
        while surface < cell.y + MAX_CLIMB && ground(IVec3::new(cell.x, surface, cell.z)) {
            surface += 1;
        }
        let dip = f64::from(surface) - p.y;
        if best.is_none_or(|(deepest, _)| dip > deepest) {
            best = Some((dip, IVec3::new(cell.x, surface, cell.z)));
        }
    }
    best.map(|(_, foot)| foot.as_vec3() + Vec3::new(0.5, POLE_WIRE_Y, 0.5))
}

/// A prop: a slim wooden post from the ground to `top` (camera-relative) with a short crossarm and one insulator,
/// lighter than a pole so it doesn't read as one.
fn push_prop(out: &mut Vec<f32>, top: Vec3) {
    let post = POLE_WIRE_Y as f32;
    let foot = top - Vec3::new(0.0, POLE_WIRE_Y, 0.0);
    push_box(
        out,
        foot + Vec3::new(0.0, f64::from(post) / 2.0 - 0.04, 0.0),
        0.0,
        [0.1, post - 0.08, 0.1],
        0.0,
        [tex::PLANKS; 3],
        true,
    );
    push_box(out, top + Vec3::new(0.0, -0.12, 0.0), 0.0, [0.36, 0.06, 0.08], 0.0, [tex::PLANKS; 3], false);
    push_box(out, top + Vec3::new(0.0, -0.05, 0.0), 0.0, [0.07, 0.1, 0.07], 0.0, [tex::FLASK_GLASS; 3], false);
}

#[cfg(test)]
mod tests;
