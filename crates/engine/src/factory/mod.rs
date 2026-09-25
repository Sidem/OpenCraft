//! Factory machines: conveyor belts, miners and storage boxes.
//!
//! Machines occupy one voxel each (the chunk holds their block id, so collision, targeting and
//! breaking work unchanged) while their state lives here, keyed by position in `at`. Each kind is
//! a `Vec` in its own file (`belt.rs`, `miner.rs`, `storage.rs`); removal is `swap_remove` plus
//! fixing the moved entry's `at` slot. Machines keep running when their chunk is streamed out.
//!
//! `update` runs miners, then boxes, then belts (downstream first, see `links.rs`). Belt links and
//! the belt order are derived data, rebuilt by `relink` whenever `dirty` is set.
//!
//! To add a machine kind (Milestone 2 turns this into a registry, together with the smelter):
//! a file with its struct and `*_step` methods, a `Slot` variant, an `add_*` method, arms in
//! `remove`, `take_contents`, `relink`, `describe` and `write_instances`, and a block id.

mod belt;
mod describe;
mod links;
mod miner;
mod render;
mod storage;

use rustc_hash::FxHashMap;

use crate::block::BlockId;
use crate::deposits::{DepositKey, Deposits};
use crate::inventory::{add_to_slots, Stack, MAX_STACK};
use crate::math::{IVec3, Vec3};
use crate::sound::Sounds;
use crate::world::World;

use belt::{belt_step, Belt};
use miner::Miner;
use storage::Storage;

pub use describe::fmt_int;
pub use miner::{MinerStatus, MINER_RECOVERY};
#[cfg(test)]
pub use miner::MINER_BUFFER;
pub use render::{push_box, INSTANCE_FLOATS};

/// Horizontal directions in player-yaw quarter turns: 0 = -Z (north), 1 = +X, 2 = +Z, 3 = -X.
pub const DIRS: [IVec3; 4] = [IVec3::new(0, 0, -1), IVec3::new(1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(-1, 0, 0)];
/// Block face normals in mesher order: +X, -X, +Y, -Y, +Z, -Z.
pub const FACES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// The horizontal direction a player facing `yaw` looks along.
pub fn dir_from_yaw(yaw: f64) -> u8 {
    ((yaw / std::f64::consts::FRAC_PI_2).round() as i32).rem_euclid(4) as u8
}

pub fn face_of(v: IVec3) -> Option<u8> {
    FACES.iter().position(|&f| f == v).map(|i| i as u8)
}

#[derive(Default)]
pub struct Factory {
    belts: Vec<Belt>,
    miners: Vec<Miner>,
    storages: Vec<Storage>,
    at: FxHashMap<IVec3, Slot>,
    /// Belt indices, downstream first.
    order: Vec<u32>,
    dirty: bool,
    pub deposits: Deposits,
    tick: u64,
}

impl Factory {
    pub fn belt_count(&self) -> usize {
        self.belts.len()
    }

    pub fn miner_count(&self) -> usize {
        self.miners.len()
    }

    pub fn storage_count(&self) -> usize {
        self.storages.len()
    }

    pub fn add_belt(&mut self, pos: IVec3, dir: u8) {
        self.remove(pos);
        self.at.insert(pos, Slot::Belt(self.belts.len() as u32));
        self.belts.push(Belt::new(pos, dir));
        self.dirty = true;
    }

    pub fn add_miner(&mut self, pos: IVec3, drill: u8, deposit: Option<DepositKey>) {
        self.remove(pos);
        self.at.insert(pos, Slot::Miner(self.miners.len() as u32));
        self.miners.push(Miner::new(pos, drill, deposit));
        self.dirty = true;
    }

    pub fn add_storage(&mut self, pos: IVec3) {
        self.remove(pos);
        self.at.insert(pos, Slot::Storage(self.storages.len() as u32));
        self.storages.push(Storage::new(pos));
        self.dirty = true;
    }

    /// Removes the machine at `pos`, returning whatever it was holding or carrying.
    pub fn remove(&mut self, pos: IVec3) -> Vec<Stack> {
        let Some(slot) = self.at.remove(&pos) else { return Vec::new() };
        self.dirty = true;
        match slot {
            Slot::Belt(i) => {
                let b = self.belts.swap_remove(i as usize);
                if let Some(moved) = self.belts.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Belt(i));
                }
                stacks_of(b.items.iter().map(|it| it.item))
            }
            Slot::Miner(i) => {
                let m = self.miners.swap_remove(i as usize);
                if let Some(moved) = self.miners.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Miner(i));
                }
                if m.held > 0 {
                    vec![Stack { item: m.ore, count: m.held }]
                } else {
                    Vec::new()
                }
            }
            Slot::Storage(i) => {
                let s = self.storages.swap_remove(i as usize);
                if let Some(moved) = self.storages.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Storage(i));
                }
                s.slots.into_iter().filter(|s| !s.is_empty()).collect()
            }
        }
    }

    /// Right-click on a box or miner: moves its contents into `take(item, count)`, which returns
    /// how many it accepted. Returns false if there is no such machine at `pos`.
    pub fn take_contents(&mut self, pos: IVec3, mut take: impl FnMut(BlockId, u32) -> u32) -> bool {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => {
                for s in self.storages[*i as usize].slots.iter_mut().filter(|s| !s.is_empty()) {
                    s.count -= take(s.item, s.count).min(s.count);
                    if s.count == 0 {
                        *s = Stack::default();
                    }
                }
                true
            }
            Some(Slot::Miner(i)) => {
                let m = &mut self.miners[*i as usize];
                if m.held > 0 {
                    m.held -= take(m.ore, m.held).min(m.held);
                }
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, dt: f64, world: &mut World, eye: Vec3, sounds: &mut Sounds) {
        if self.dirty {
            self.relink();
        }
        self.tick += 1;
        let tick = self.tick;
        let Factory { belts, miners, storages, deposits, order, .. } = self;
        for m in miners.iter_mut() {
            let drawn = m.draw_step(deposits, world, tick, dt);
            m.output_step(belts, storages);
            m.sound_step(drawn, dt, eye, sounds);
        }
        for s in storages.iter_mut() {
            s.output_step(belts);
        }
        belt_step(belts, storages, order, dt);
    }

    #[cfg(test)]
    pub fn storage_count_at(&self, pos: IVec3, item: BlockId) -> u32 {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => {
                self.storages[*i as usize].slots.iter().filter(|s| s.item == item).map(|s| s.count).sum()
            }
            _ => 0,
        }
    }

    #[cfg(test)]
    fn belt_at(&self, pos: IVec3) -> &Belt {
        match self.at.get(&pos) {
            Some(Slot::Belt(i)) => &self.belts[*i as usize],
            _ => panic!("no belt at {pos:?}"),
        }
    }

    #[cfg(test)]
    pub fn miner_at(&self, pos: IVec3) -> &Miner {
        match self.at.get(&pos) {
            Some(Slot::Miner(i)) => &self.miners[*i as usize],
            _ => panic!("no miner at {pos:?}"),
        }
    }
}

/// Where a belt, miner or box delivers to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Link {
    None,
    /// `mid`: joining from the side, so the item enters halfway along the target belt.
    Belt {
        belt: u32,
        mid: bool,
    },
    Storage(u32),
}

/// What occupies a position: an index into the matching machine `Vec`.
#[derive(Clone, Copy, Debug)]
enum Slot {
    Belt(u32),
    Miner(u32),
    Storage(u32),
}

#[inline]
fn opposite(dir: u8) -> u8 {
    (dir + 2) % 4
}

/// Hands one item to a link. `overflow` is how far past the end of the source belt it already is.
fn deliver(belts: &mut [Belt], storages: &mut [Storage], link: Link, item: BlockId, overflow: f32) -> bool {
    match link {
        Link::None => false,
        Link::Belt { belt, mid } => belts[belt as usize].accept(item, mid, overflow),
        Link::Storage(s) => add_to_slots(&mut storages[s as usize].slots, item, 1) == 0,
    }
}

/// Merges loose items into stacks (for dropping a machine's contents).
fn stacks_of(items: impl Iterator<Item = BlockId>) -> Vec<Stack> {
    let mut out: Vec<Stack> = Vec::new();
    for item in items {
        match out.iter_mut().find(|s| s.item == item && s.count < MAX_STACK) {
            Some(s) => s.count += 1,
            None => out.push(Stack { item, count: 1 }),
        }
    }
    out
}

#[cfg(test)]
mod tests;
