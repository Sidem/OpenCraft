//! Conveyor belts, simulated per cell. Each belt holds a few items with a progress value `p` in
//! 0..1 along its length, front (highest `p`) first, kept at least [`ITEM_SPACING`] apart.
//!
//! `belt_step` walks belts downstream first (the `order` from `links.rs`), so a moving line never
//! stalls for a tick at cell borders. A belt hands its front item to whatever is in front of it:
//! another belt (entering at its start, or in its middle when joining from the side) or a box.
//! Ramps, lifts and underpasses are belts with a `shape` (`belt_shape.rs`); a fast belt is a flat belt
//! with `fast` set, moving at [`FAST_BELT_SPEED`].

use std::f32::consts::FRAC_PI_2;

use crate::block::{self, tex};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};

use super::belt_shape::Shape;
use super::links::{deliver, Link, Sinks};
use super::render::push_box;
use super::{Factory, Machine, DIRS};

const DIR_NAMES: [&str; 4] = ["north", "east", "south", "west"];

/// Belt speed in blocks per second.
pub const BELT_SPEED: f32 = 1.0;
pub const FAST_BELT_SPEED: f32 = 2.0;
/// Minimum distance between item centres on a belt, in blocks.
pub const ITEM_SPACING: f32 = 0.35;
pub const ITEM_SIZE: f32 = 0.25;
/// Where items wait on a belt with nothing in front of it (still fully on the belt).
pub const END_STOP: f32 = 1.0 - ITEM_SIZE * 0.5;
pub const BELT_HEIGHT: f32 = 0.18;

#[derive(Clone, Copy, Debug)]
pub struct BeltItem {
    pub item: ItemId,
    pub p: f32,
}

pub struct Belt {
    pub pos: IVec3,
    pub dir: u8,
    /// Front (highest `p`) first.
    pub items: Vec<BeltItem>,
    pub out: Link,
    /// Set when the only thing feeding this belt comes in from one side: items then enter from that
    /// side and turn the corner at the centre.
    pub curve_from: Option<u8>,
    pub shape: Shape,
    /// A lift that takes items from the lift below / hands them to the lift above (derived).
    pub lift_below: bool,
    pub lift_above: bool,
    pub fast: bool,
}

impl Belt {
    pub fn new(pos: IVec3, dir: u8, shape: Shape, fast: bool) -> Belt {
        let (items, out, curve_from) = (Vec::new(), Link::None, None);
        Belt { pos, dir: dir % 4, items, out, curve_from, shape, lift_below: false, lift_above: false, fast }
    }

    /// Blocks per second.
    pub fn speed(&self) -> f32 {
        if self.fast {
            FAST_BELT_SPEED
        } else {
            BELT_SPEED
        }
    }

    /// Item offset from the cell centre (horizontal) at progress `p`.
    pub fn offset(&self, p: f32) -> (f32, f32) {
        let (d, t) = match self.curve_from {
            Some(side) if p < 0.5 => (DIRS[side as usize], 0.5 - p),
            _ => (DIRS[self.dir as usize], p - 0.5),
        };
        (d.x as f32 * t, d.z as f32 * t)
    }

    pub fn mid_free(&self) -> bool {
        self.items.iter().all(|it| (it.p - 0.5).abs() >= ITEM_SPACING)
    }

    /// Accepts an item at the start (with `overflow` progress already travelled) or in the middle.
    pub fn accept(&mut self, item: ItemId, mid: bool, overflow: f32) -> bool {
        if mid {
            if !self.mid_free() {
                return false;
            }
            let i = self.items.iter().position(|it| it.p < 0.5).unwrap_or(self.items.len());
            self.items.insert(i, BeltItem { item, p: 0.5 });
            return true;
        }
        let p = match self.items.last() {
            Some(rear) if rear.p < ITEM_SPACING => return false,
            Some(rear) => overflow.min(rear.p - ITEM_SPACING),
            None => overflow.min(END_STOP),
        };
        self.items.push(BeltItem { item, p: p.max(0.0) });
        true
    }
}

/// Moves every belt's items forward by `dt` and hands front items on, downstream belts first.
pub fn belt_step(belts: &mut [Belt], sinks: &mut Sinks, order: &[u32], dt: f64) {
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
                if sinks.can_accept(slot, front.item) {
                    f32::INFINITY
                } else {
                    1.0
                }
            }
        };
        for it in belts[bi].items.iter_mut() {
            it.p = (it.p + step).min(limit).max(it.p);
            limit = it.p - ITEM_SPACING;
        }
        while let Some(front) = belts[bi].items.first().copied() {
            if front.p < 1.0 {
                break;
            }
            if deliver(belts, sinks, out, front.item, front.p - 1.0) {
                belts[bi].items.remove(0);
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
    /// are rebuilt by `relink`). Saves before version 6 have only flat belts, before 9 only slow ones.
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.dir);
        w.u8(self.shape as u8);
        w.bool(self.fast);
        w.count(self.items.len());
        for it in &self.items {
            w.item(it.item);
            w.f32(it.p);
        }
    }

    fn read_state(r: &mut ByteReader) -> Option<Belt> {
        let (pos, dir) = (r.ivec3()?, r.u8()?);
        let shape = if r.version >= 6 { Shape::from_u8(r.u8()?)? } else { Shape::Flat };
        let fast = r.version >= 9 && r.bool()?;
        let mut belt = Belt::new(pos, dir, shape, fast);
        for _ in 0..r.count()? {
            belt.items.push(BeltItem { item: r.item()?, p: r.f32()? });
        }
        (dir < 4).then_some(belt)
    }

    /// The carried items, merged into stacks.
    fn contents(&self) -> Vec<Stack> {
        let mut out: Vec<Stack> = Vec::new();
        for it in &self.items {
            match out.iter_mut().find(|s| s.item == it.item && s.count < item::stack_size(it.item)) {
                Some(s) => s.count += 1,
                None => out.push(Stack { item: it.item, count: 1 }),
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
                    " · no underpass exit facing the same way within 5 cells ahead".to_string()
                }
                Link::None => " · nothing in front, items wait at the end".to_string(),
                Link::Belt { mid: true, .. } => " · joins the next belt from the side".to_string(),
                Link::Belt { .. } => String::new(),
                Link::Machine(slot) => format!(" · delivers into the {}", block::def(slot.kind().def().block).name),
            }
        };
        format!("{load} · heading {}{}{end}", DIR_NAMES[self.dir as usize], self.shape.words())
    }

    /// A scrolling rubber top between two rails (or the shape's model), and the items riding on it.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let base = rel - Vec3::new(0.0, 0.5, 0.0);
        let scroll = (time * self.speed() as f64).fract() as f32;
        let yaw = self.dir as f32 * FRAC_PI_2;
        let (s, c) = yaw.sin_cos();
        let at = |x: f32, y: f32, z: f32| base + Vec3::new((c * x - s * z) as f64, y as f64, (s * x + c * z) as f64);
        if matches!(self.shape, Shape::Flat | Shape::Entry | Shape::Exit) {
            let top = [if self.fast { tex::FAST_BELT_TOP } else { tex::BELT_TOP }, tex::FRAME, tex::FRAME];
            push_box(out, at(0.0, BELT_HEIGHT * 0.5, 0.0), yaw, [0.84, BELT_HEIGHT, 1.0], scroll, top, true);
            for side in [-0.46, 0.46] {
                push_box(out, at(side, 0.13, 0.0), yaw, [0.08, 0.26, 1.0], 0.0, [tex::FRAME; 3], true);
            }
        }
        self.shape_model(out, &at, yaw, scroll);
        for it in self.items.iter().filter(|it| self.shows(it.p)) {
            let Some(def) = item::def(it.item) else { continue };
            let (x, y, z) = self.item_at(it.p);
            let size = def.size.map(|s| s * ITEM_SIZE);
            let pos = base + Vec3::new(x as f64, (y + BELT_HEIGHT + size[1] * 0.5) as f64, z as f64);
            push_box(out, pos, yaw, size, 0.0, def.tex, false);
        }
    }
}
