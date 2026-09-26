//! Loose items in co-op: only the host runs item physics and pickups
//! (`entities.rs`, through authority.rs); a client never spawns an item and draws what the host sends.
//! Ten times a second (`ITEM_TICKS`) the host notes, for each peer, the items within `VIEW_RADIUS` of
//! that peer's body (`take_item_view`). The client keeps the latest list (`push_items`) and glides each
//! item, matched by id, towards where the host last saw it.
//!
//! Bytes: a count, then per item its id (`u32`), item (`u16`) and position (3 × `f32`); at most
//! `VIEW_ITEMS`. A damaged buffer is refused whole. None of this touches the core.

use super::Role;
use crate::bytes::{ByteReader, ByteWriter};
use crate::entities::push_item_box;
use crate::item::ItemId;
use crate::math::Vec3;
use crate::sim::PlayerId;
use crate::Game;

/// Body ticks between item views: 10 a second.
pub const ITEM_TICKS: u32 = 6;
/// A peer sees the items within this many blocks of its body.
const VIEW_RADIUS: f64 = 48.0;
const VIEW_ITEMS: usize = 256;
/// How fast a drawn item closes the gap to its latest position (per second), and the gap (in blocks)
/// beyond which it jumps instead.
const GLIDE_RATE: f64 = 20.0;
const SNAP: f64 = 4.0;

/// A client's items, as drawn.
#[derive(Default)]
pub(crate) struct ItemView {
    list: Vec<Shown>,
}

struct Shown {
    id: u32,
    item: ItemId,
    pos: Vec3,
    target: Vec3,
    /// Seconds since this client first saw it (spin and bob).
    age: f32,
}

impl Game {
    /// Host: notes each peer's view (from `net_tick`).
    pub(crate) fn write_item_views(&mut self) {
        let Role::Host(h) = &mut self.role else { return };
        h.views.clear();
        for &peer in &h.peers {
            let Some(Some(body)) = self.bodies.get(peer.0 as usize) else { continue };
            let near = self.items.list.iter().filter(|e| (e.pos - body.pos).length() < VIEW_RADIUS).take(VIEW_ITEMS);
            let near: Vec<_> = near.collect();
            let mut w = ByteWriter::default();
            w.count(near.len());
            for e in near {
                w.u32(e.id);
                w.item(e.item);
                for v in [e.pos.x, e.pos.y, e.pos.z] {
                    w.f32(v as f32);
                }
            }
            h.views.push((peer, w.bytes));
        }
    }

    /// Host: peer `id`'s latest view, once; empty when nothing new was noted.
    pub(crate) fn item_view_for(&mut self, id: PlayerId) -> Vec<u8> {
        let Role::Host(h) = &mut self.role else { return Vec::new() };
        match h.views.iter().position(|v| v.0 == id) {
            Some(i) => h.views.swap_remove(i).1,
            None => Vec::new(),
        }
    }

    /// Client: the host's latest view. False (and nothing changed) if the bytes are damaged.
    pub(crate) fn push_item_view(&mut self, bytes: &[u8]) -> bool {
        let Role::Client(c) = &mut self.role else { return false };
        let mut r = ByteReader::new(bytes);
        let Some(list) = read_view(&mut r).filter(|_| r.is_done()) else { return false };
        let old = std::mem::take(&mut c.items.list);
        c.items.list = list
            .into_iter()
            .map(|(id, item, target)| match old.iter().find(|o| o.id == id) {
                Some(o) => Shown { id, item, pos: o.pos, target, age: o.age },
                None => Shown { id, item, pos: target, target, age: 0.0 },
            })
            .collect();
        true
    }

    /// Box instances for the loose items: a client's view, glided on by `dt`, or this game's own.
    pub(crate) fn write_item_instances(&mut self, dt: f64, eye: Vec3) {
        let Role::Client(c) = &mut self.role else {
            self.items.write_instances(&mut self.instances, eye);
            return;
        };
        let k = 1.0 - (-GLIDE_RATE * dt).exp();
        for s in &mut c.items.list {
            let gap = s.target - s.pos;
            s.pos = if gap.length() > SNAP { s.target } else { s.pos + gap * k };
            s.age += dt as f32;
            push_item_box(&mut self.instances, s.item, s.pos - eye, s.age);
        }
    }
}

fn read_view(r: &mut ByteReader) -> Option<Vec<(u32, ItemId, Vec3)>> {
    let mut list = Vec::new();
    for _ in 0..r.count()? {
        let (id, item) = (r.u32()?, r.item()?);
        let pos = Vec3::new(r.f32()? as f64, r.f32()? as f64, r.f32()? as f64);
        let finite = pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite();
        if item == ItemId::NONE || !finite {
            return None;
        }
        list.push((id, item, pos));
    }
    Some(list)
}
