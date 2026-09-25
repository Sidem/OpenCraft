//! Dropped item entities: physics, magnet pickup and GPU instance data.
//!
//! Items fall and slide only while their position is in a loaded chunk (elsewhere they wait), are
//! pulled towards the nearest [`Collector`] (a player) in reach once their pickup delay has passed and
//! that collector has room, and despawn after [`DESPAWN_SECONDS`]. Drawn as small boxes through
//! `factory::push_box`.

use crate::block::{BlockId, AIR, FACE_BOTTOM, FACE_SIDE, FACE_TEX, FACE_TOP};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::push_box;
use crate::inventory::Inventory;
use crate::math::Vec3;
use crate::physics::{move_axis, Aabb};

const HALF: f64 = 0.125;
const GRAVITY: f64 = 22.0;
const DESPAWN_SECONDS: f32 = 300.0;
const MAGNET_RADIUS: f64 = 2.4;
const PICKUP_RADIUS: f64 = 0.7;

pub struct ItemEntity {
    pub pos: Vec3,
    pub vel: Vec3,
    pub item: BlockId,
    pub count: u32,
    pub age: f32,
    pub pickup_delay: f32,
    on_ground: bool,
}

#[derive(Default)]
pub struct Items {
    pub list: Vec<ItemEntity>,
}

/// Something items fly to: a player's body centre and a scratch copy of its inventory, which the
/// pickups fill so later items see the room that is left.
pub struct Collector {
    pub center: Vec3,
    pub room: Inventory,
}

impl Items {
    pub fn spawn(&mut self, pos: Vec3, vel: Vec3, item: BlockId, count: u32, pickup_delay: f32) {
        self.list.push(ItemEntity { pos, vel, item, count, age: 0.0, pickup_delay, on_ground: false });
    }

    /// What a save keeps of each item (`on_ground` is recomputed every step).
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.list.len());
        for e in &self.list {
            w.vec3(e.pos);
            w.vec3(e.vel);
            w.u8(e.item);
            w.u32(e.count);
            w.f32(e.age);
            w.f32(e.pickup_delay);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Items> {
        let mut items = Items::default();
        for _ in 0..r.count()? {
            let (pos, vel, item, count, age, delay) = (r.vec3()?, r.vec3()?, r.block()?, r.u32()?, r.f32()?, r.f32()?);
            if item == AIR || count == 0 {
                return None;
            }
            items.spawn(pos, vel, item, count, delay);
            items.list.last_mut()?.age = age;
        }
        Some(items)
    }

    /// Steps all items. `collected(collector, item, n)` is called for every successful pickup, with the
    /// collector's index; ties in distance go to the lower index.
    pub fn update(
        &mut self,
        dt: f64,
        collectors: &mut [Collector],
        solid: &mut impl FnMut(i32, i32, i32) -> bool,
        loaded: &impl Fn(Vec3) -> bool,
        mut collected: impl FnMut(usize, BlockId, u32),
    ) {
        let mut i = 0;
        while i < self.list.len() {
            let e = &mut self.list[i];
            e.age += dt as f32;
            if e.age > DESPAWN_SECONDS {
                self.list.swap_remove(i);
                continue;
            }

            let mut nearest: Option<(usize, f64)> = None;
            if e.age >= e.pickup_delay {
                for (c, col) in collectors.iter().enumerate() {
                    let dist = (col.center - e.pos).length();
                    if dist < MAGNET_RADIUS && nearest.is_none_or(|(_, d)| dist < d) && col.room.space_for(e.item) > 0 {
                        nearest = Some((c, dist));
                    }
                }
            }
            if let Some((c, dist)) = nearest {
                let col = &mut collectors[c];
                let to_player = col.center - e.pos;
                if dist < PICKUP_RADIUS {
                    let left = col.room.add(e.item, e.count);
                    let taken = e.count - left;
                    if taken > 0 {
                        collected(c, e.item, taken);
                    }
                    if left == 0 {
                        self.list.swap_remove(i);
                        continue;
                    }
                    e.count = left;
                }
                // Fly straight at the player, accelerating as it gets closer.
                let speed = 5.0 + (MAGNET_RADIUS - dist) * 8.0;
                e.vel = to_player * (speed / dist.max(1e-3));
                e.pos += e.vel * dt;
                e.on_ground = false;
            } else if loaded(e.pos) {
                e.vel.y = (e.vel.y - GRAVITY * dt).max(-40.0);
                let mut bb = Aabb::centered(e.pos, HALF);
                let want_y = e.vel.y * dt;
                let dy = move_axis(&mut bb, 1, want_y, solid);
                e.on_ground = want_y < 0.0 && (dy - want_y).abs() > 1e-9;
                if (dy - want_y).abs() > 1e-9 {
                    e.vel.y = 0.0;
                }
                for axis in [0, 2] {
                    let want = e.vel.get(axis) * dt;
                    if (move_axis(&mut bb, axis, want, solid) - want).abs() > 1e-9 {
                        e.vel.set(axis, 0.0);
                    }
                }
                if e.on_ground {
                    let f = 1.0 - (10.0 * dt).min(1.0);
                    e.vel.x *= f;
                    e.vel.z *= f;
                }
                e.pos = bb.center();
            }
            i += 1;
        }
    }

    /// Appends camera-relative box instances (see `factory::INSTANCE_FLOATS`) for the renderer.
    pub fn write_instances(&self, out: &mut Vec<f32>, eye: Vec3) {
        let size = (HALF * 2.0) as f32;
        for e in &self.list {
            let faces = FACE_TEX[e.item as usize];
            let bob = (e.age as f64 * 2.6).sin() * 0.05 + 0.05;
            let tex = [faces[FACE_TOP], faces[FACE_SIDE], faces[FACE_BOTTOM]];
            push_box(out, e.pos - eye + Vec3::new(0.0, bob, 0.0), e.age * 1.7, [size; 3], 0.0, tex, false);
        }
    }
}

#[cfg(test)]
mod tests;
