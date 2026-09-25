//! Conveyor belts, simulated per cell. Each belt holds a few items with a progress value `p` in
//! 0..1 along its length, front (highest `p`) first, kept at least [`ITEM_SPACING`] apart.
//!
//! `belt_step` walks belts downstream first (the `order` from `links.rs`), so a moving line never
//! stalls for a tick at cell borders. A belt hands its front item to whatever is in front of it:
//! another belt (entering at its start, or in its middle when joining from the side) or a box.

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;

use super::storage::Storage;
use super::{deliver, Link, DIRS};

/// Belt speed in blocks per second.
pub const BELT_SPEED: f32 = 1.0;
/// Minimum distance between item centres on a belt, in blocks.
pub const ITEM_SPACING: f32 = 0.35;
pub const ITEM_SIZE: f32 = 0.25;
/// Where items wait on a belt with nothing in front of it (still fully on the belt).
pub const END_STOP: f32 = 1.0 - ITEM_SIZE * 0.5;
pub const BELT_HEIGHT: f32 = 0.18;

#[derive(Clone, Copy, Debug)]
pub struct BeltItem {
    pub item: BlockId,
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
}

impl Belt {
    pub fn new(pos: IVec3, dir: u8) -> Belt {
        Belt { pos, dir: dir % 4, items: Vec::new(), out: Link::None, curve_from: None }
    }

    /// Core state: position, direction and items (`out` and `curve_from` are rebuilt by `relink`).
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.dir);
        w.count(self.items.len());
        for it in &self.items {
            w.u8(it.item);
            w.f32(it.p);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Belt> {
        let (pos, dir) = (r.ivec3()?, r.u8()?);
        let mut belt = Belt::new(pos, dir);
        for _ in 0..r.count()? {
            belt.items.push(BeltItem { item: r.block()?, p: r.f32()? });
        }
        (dir < 4).then_some(belt)
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
    pub fn accept(&mut self, item: BlockId, mid: bool, overflow: f32) -> bool {
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
pub fn belt_step(belts: &mut [Belt], storages: &mut [Storage], order: &[u32], dt: f64) {
    let step = BELT_SPEED * dt as f32;
    for &bi in order.iter() {
        let bi = bi as usize;
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
            Link::Storage(k) => {
                if storages[k as usize].can_accept(front.item) {
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
            if deliver(belts, storages, out, front.item, front.p - 1.0) {
                belts[bi].items.remove(0);
            } else {
                belts[bi].items[0].p = 1.0;
                break;
            }
        }
    }
}
