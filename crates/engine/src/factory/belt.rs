//! Conveyor belts, simulated per cell. Each belt holds a few items with a progress value `p` in
//! 0..1 along its length, front (highest `p`) first, kept at least [`ITEM_SPACING`] apart.
//!
//! `belt_step` walks belts downstream first (the `order` from `links.rs`), so a moving line never
//! stalls for a tick at cell borders. A belt hands its front item to whatever is in front of it:
//! another belt (entering at its start, or in its middle when joining from the side) or a box.
//! Ramps, lifts and underpasses are belts with a `shape` (`belt_shape.rs`); every shape comes in the
//! tiers of [`BELT_TIERS`] (`tier` indexes it; `tiers.rs` names the item of each). A body standing on
//! a belt is carried along at its speed (`Factory::conveyor_at`, used by `authority.rs`). Each belt counts the items it
//! hands on per [`FLOW_SECS`] window and averages the last [`FLOW_WINDOWS`] (its readout's flow, against
//! [`most_a_minute`]; derived, not saved). Drawing it: `belt_model.rs`.

use std::f32::consts::FRAC_PI_2;

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::entities::push_item_model;
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};

use super::belt_curve::{bend_of, bend_words, BENDS, BEND_STRAIGHT};
use super::belt_shape::Shape;
use super::links::{deliver, Link, Sinks, Slot};
use super::provenance::Tag;
use super::render::push_box;
use super::{Factory, Machine, DIRS};

const DIR_NAMES: [&str; 4] = ["north", "east", "south", "west"];

/// A belt tier's numbers: speed in blocks per second and the texture of its moving top.
pub struct BeltTier {
    pub speed: f32,
    pub top: u16,
}

/// Mk1 first. Speeds: TECH_ERAS section 1.
pub const BELT_TIERS: [BeltTier; 4] = [
    BeltTier { speed: 1.0, top: tex::BELT_TOP },
    BeltTier { speed: 2.0, top: tex::FAST_BELT_TOP },
    BeltTier { speed: 4.0, top: tex::BELT_MK3_TOP },
    BeltTier { speed: 8.0, top: tex::BELT_MK4_TOP },
];
/// Minimum distance between item centres on a belt, in blocks.
pub const ITEM_SPACING: f32 = 0.35;
pub const ITEM_SIZE: f32 = 0.25;
/// Beyond this many blocks from the camera, an item riding a belt is one plain box, not its model.
pub const ITEM_DETAIL_RANGE: f64 = 24.0;
/// Where items wait on a belt with nothing in front of it (still fully on the belt).
pub const END_STOP: f32 = 1.0 - ITEM_SIZE * 0.5;
pub const BELT_HEIGHT: f32 = 0.18;

#[derive(Clone, Copy, Debug)]
pub struct BeltItem {
    pub item: ItemId,
    /// 1, except a tool: its uses left (it rides as one piece, `tools::lot`).
    pub n: u32,
    pub p: f32,
    /// Where it was put on a belt and how long it has ridden (`provenance.rs`).
    pub tag: Tag,
}

pub struct Belt {
    pub pos: IVec3,
    pub dir: u8,
    /// Front (highest `p`) first.
    pub items: Vec<BeltItem>,
    pub out: Link,
    /// The side items come in from when they turn a corner here (elt_curve.rs): the player's bend, or else set when the
    /// only thing feeding this belt comes in from one side (derived).
    pub curve_from: Option<u8>,
    /// The player's bend (elt_curve.rs: straight, or taking items in from the left or right), relative to dir.
    /// Only flat belts bend; core state.
    pub bend: u8,
    pub shape: Shape,
    /// A lift that takes items from the lift below / hands them to the lift above (derived).
    pub lift_below: bool,
    pub lift_above: bool,
    /// Index into [`BELT_TIERS`] (0 is Mk1).
    pub tier: u8,
    /// Items handed on so far this [`FLOW_SECS`] window, in the last [`FLOW_WINDOWS`] whole ones (newest first), and
    /// how many of those have closed (derived, not saved: the readout's flow).
    pub passed: u16,
    pub recent: [u16; FLOW_WINDOWS],
    pub windows: u8,
}

/// Seconds a belt counts the items it hands on before its readout's flow changes.
pub const FLOW_SECS: u32 = 10;
/// Windows the flow averages: one alone counts 6 or 7 items where a machine makes 40 a minute.
pub const FLOW_WINDOWS: usize = 3;

/// Items a minute a belt of `tier` carries at most: one enters when the one before has gone [`ITEM_SPACING`],
/// counted in whole ticks as a box or machine feeds it (the same sums as `belt_step`).
pub fn most_a_minute(tier: u8) -> u32 {
    let step = BELT_TIERS[tier as usize].speed * crate::TICK as f32;
    let (mut p, mut ticks) = (0.0f32, 0);
    while p < ITEM_SPACING {
        (p, ticks) = (p + step, ticks + 1);
    }
    60 * crate::TICK_RATE / ticks
}

/// A belt's model where none stands yet (the line tool's preview, `belt_line.rs`), without items. `facing` carries a
/// flat belt's bend (`belt_facing`).
pub fn belt_preview(out: &mut Vec<f32>, pos: IVec3, facing: u8, shape: Shape, tier: u8, rel: Vec3, time: f64) {
    let mut belt = Belt::new(pos, facing, shape, tier);
    if shape == Shape::Flat && bend_of(facing) != BEND_STRAIGHT {
        belt.bend = bend_of(facing);
        belt.curve_from = Some(belt.entry_side());
    }
    belt.model(out, rel, time);
}

impl Belt {
    pub fn new(pos: IVec3, dir: u8, shape: Shape, tier: u8) -> Belt {
        let (items, out, curve_from) = (Vec::new(), Link::None, None);
        let (lift_below, lift_above) = (false, false);
        let (passed, recent, windows) = (0, [0; FLOW_WINDOWS], 0);
        let bend = BEND_STRAIGHT;
        Belt {
            pos,
            dir: dir % 4,
            items,
            out,
            curve_from,
            bend,
            shape,
            lift_below,
            lift_above,
            tier,
            passed,
            recent,
            windows,
        }
    }

    /// Items a minute it handed on over its closed windows (the last [`FLOW_WINDOWS`]), if one has closed.
    pub fn flow_a_minute(&self) -> Option<u32> {
        let n = usize::from(self.windows);
        let items: u32 = self.recent[..n].iter().map(|&c| u32::from(c)).sum();
        (n > 0).then(|| items * 60 / (FLOW_SECS * n as u32))
    }

    /// Blocks per second.
    pub fn speed(&self) -> f32 {
        BELT_TIERS[self.tier as usize].speed
    }

    /// Item offset from the cell centre (horizontal) at progress `p`.
    pub fn offset(&self, p: f32) -> (f32, f32) {
        if let Some(side) = self.curve_from {
            return self.curve_offset(side, p);
        }
        let d = DIRS[self.dir as usize];
        (d.x as f32 * (p - 0.5), d.z as f32 * (p - 0.5))
    }

    pub fn mid_free(&self) -> bool {
        self.items.iter().all(|it| (it.p - 0.5).abs() >= ITEM_SPACING)
    }

    /// Accepts an item (`n` of it: 1, or a tool's uses) at the start (with `overflow` progress already travelled)
    /// or in the middle.
    pub fn accept(&mut self, item: ItemId, n: u32, tag: Tag, mid: bool, overflow: f32) -> bool {
        if mid {
            if !self.mid_free() {
                return false;
            }
            let i = self.items.iter().position(|it| it.p < 0.5).unwrap_or(self.items.len());
            self.items.insert(i, BeltItem { item, n, p: 0.5, tag });
            return true;
        }
        let p = match self.items.last() {
            Some(rear) if rear.p < ITEM_SPACING => return false,
            Some(rear) => overflow.min(rear.p - ITEM_SPACING),
            None => overflow.min(END_STOP),
        };
        self.items.push(BeltItem { item, n, p: p.max(0.0), tag });
        true
    }
}

impl Factory {
    /// How fast the belt under a body's feet carries it, in blocks per second (zero off belts, in the
    /// air and on lifts, which are climbed). Belts aren't solid: a body stands on the ground in the
    /// belt's cell, so the feet must be within the belt's own height of the cell's floor.
    pub fn conveyor_at(&self, feet: Vec3) -> Vec3 {
        let cell = Vec3::new(feet.x, feet.y + 1e-3, feet.z).floor();
        if feet.y - cell.y as f64 > f64::from(BELT_HEIGHT) + 0.05 {
            return Vec3::ZERO;
        }
        match self.at.get(&cell) {
            Some(&Slot::Belt(i)) => {
                let belt = &self.belts[i as usize];
                let d = DIRS[belt.dir as usize];
                let speed = if belt.shape == Shape::Lift { 0.0 } else { f64::from(belt.speed()) };
                Vec3::new(d.x as f64 * speed, 0.0, d.z as f64 * speed)
            }
            _ => Vec3::ZERO,
        }
    }
}

/// Moves every belt's items forward by `dt` and hands front items on, downstream belts first. On every
/// [`FLOW_SECS`]th second of `tick` each belt's count of items handed on closes a window of its flow.
pub fn belt_step(belts: &mut [Belt], sinks: &mut Sinks, order: &[u32], dt: f64, tick: u64) {
    if tick > 0 && tick.is_multiple_of(u64::from(FLOW_SECS * crate::TICK_RATE)) {
        for b in belts.iter_mut() {
            b.recent.rotate_right(1);
            b.recent[0] = std::mem::take(&mut b.passed);
            b.windows = (b.windows + 1).min(FLOW_WINDOWS as u8);
        }
    }
    for &bi in order.iter() {
        let bi = bi as usize;
        let step = belts[bi].speed() * dt as f32;
        let out = belts[bi].out;
        let Some(front) = belts[bi].items.first().copied() else { continue };
        let mut limit = match out {
            Link::None => END_STOP,
            Link::Belt { belt, mid: false } => {
                belts[belt as usize].items.last().map_or(f32::INFINITY, |r| 1.0 + r.p - ITEM_SPACING)
            }
            Link::Belt { belt, mid: true } => {
                if belts[belt as usize].mid_free() {
                    f32::INFINITY
                } else {
                    1.0
                }
            }
            Link::Machine(slot) => {
                if sinks.can_accept(slot, front.item, front.n) {
                    f32::INFINITY
                } else {
                    1.0
                }
            }
        };
        for it in belts[bi].items.iter_mut() {
            it.p = (it.p + step).min(limit).max(it.p);
            limit = it.p - ITEM_SPACING;
            it.tag.age();
        }
        while let Some(front) = belts[bi].items.first().copied() {
            if front.p < 1.0 {
                break;
            }
            if deliver(belts, sinks, out, (front.item, front.n, front.tag), front.p - 1.0) {
                belts[bi].items.remove(0);
                belts[bi].passed = belts[bi].passed.saturating_add(1);
            } else {
                belts[bi].items[0].p = 1.0;
                break;
            }
        }
    }
}

impl Machine for Belt {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: position, direction, shape, speed and items (`out`, `curve_from` and the lift flags
    /// are rebuilt by `relink`). Saves before version 6 have only flat belts, before 9 only Mk1 ones (the
    /// tier byte was a `fast` bool then, so tiers 0 and 1 are byte-identical to it), before 62 only straight ones.
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.dir);
        w.u8(self.shape as u8);
        w.u8(self.tier);
        w.u8(self.bend);
        w.count(self.items.len());
        for it in &self.items {
            w.item(it.item);
            w.u32(it.n);
            w.f32(it.p);
            it.tag.write_state(w);
        }
    }

    fn read_state(r: &mut ByteReader) -> Option<Belt> {
        let (pos, dir) = (r.ivec3()?, r.u8()?);
        let shape = if r.version >= 6 { Shape::from_u8(r.u8()?)? } else { Shape::Flat };
        let tier = if r.version >= 9 { r.u8()? } else { 0 };
        let mut belt = Belt::new(pos, dir, shape, tier);
        belt.bend = if r.version >= 62 { r.u8()? } else { BEND_STRAIGHT };
        for _ in 0..r.count()? {
            let item = r.item()?;
            // Before version 39 a tool on a belt was one use per item.
            let n = if r.version >= 39 { r.u32()? } else { 1 };
            let p = r.f32()?;
            belt.items.push(BeltItem { item, n, p, tag: Tag::read_state(r)? });
        }
        let bend_ok = belt.bend == BEND_STRAIGHT || (belt.bend < BENDS && shape == Shape::Flat);
        (dir < 4 && (tier as usize) < BELT_TIERS.len() && bend_ok).then_some(belt)
    }

    /// The carried items, merged into stacks.
    fn contents(&self) -> Vec<Stack> {
        let mut out: Vec<Stack> = Vec::new();
        for it in &self.items {
            match out.iter_mut().find(|s| s.item == it.item && s.count + it.n <= item::stack_size(it.item)) {
                Some(s) => s.count += it.n,
                None => out.push(Stack { item: it.item, count: it.n }),
            }
        }
        out
    }

    /// Load, heading and what it delivers into (skipped until the links are rebuilt).
    fn describe(&self, f: &Factory) -> String {
        let load = match self.items.len() {
            0 => "Empty".to_string(),
            1 => "Carrying 1 item".to_string(),
            n => format!("Carrying {n} items"),
        };
        let end = if f.dirty {
            String::new()
        } else {
            match self.out {
                Link::None if self.shape == Shape::Entry => {
                    let blocks = super::underpass::span(self.tier);
                    format!(" · needs another underpass facing the same way, at most {blocks} blocks ahead")
                }
                Link::None => " · nothing in front, items wait at the end".to_string(),
                Link::Belt { mid: true, .. } => " · joins the next belt from the side".to_string(),
                Link::Belt { .. } => String::new(),
                Link::Machine(slot) => format!(" · delivers into the {}", f.name_of(slot)),
            }
        };
        let most = most_a_minute(self.tier);
        let flow = match self.flow_a_minute() {
            Some(n) => format!("Moves {n} items a minute, {}% of the {most} it can carry", n * 100 / most),
            None => format!("Carries up to {most} items a minute"),
        };
        let (heading, bend) = (DIR_NAMES[self.dir as usize], bend_words(self.bend));
        format!("{load} · heading {heading}{bend}{}{end}\n{flow}", self.shape.words())
    }

    /// A scrolling rubber top between two rails (or the shape's model), and the items riding on it.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        self.frame(out, rel, time);
        let base = rel - Vec3::new(0.0, 0.5, 0.0);
        let yaw = self.dir as f32 * FRAC_PI_2;
        for it in self.items.iter().filter(|it| self.shows(it.p)) {
            let Some((pos, size)) = self.item_pose(it, base) else { continue };
            // Only near the camera: a modelled item is many boxes, and far belts hold thousands of them.
            if rel.length() > ITEM_DETAIL_RANGE || !push_item_model(out, it.item, pos, yaw, ITEM_SIZE as f64) {
                push_box(out, pos, yaw, size, 0.0, item::def(it.item).map_or([0; 3], |d| d.tex), false);
            }
        }
    }
}
