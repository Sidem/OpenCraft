//! Ore deposits: every ore block belongs to one deposit, and a deposit is one pooled reserve.
//!
//! - **Geometry** is derived from world-generation hashes ([`Deposit`]), so nothing is stored for a
//!   deposit nobody has touched. Worldgen stamps the same shapes it seeds, and [`Deposit::contains`]
//!   answers "which deposit owns this block" by re-running the shape test.
//! - **Grade** is the number of ore units one block holds. It depends on the tier: small outcrops at
//!   the surface, larger veins further down, and huge lodes near bedrock.
//! - **Hand mining** keeps a fixed handful per block ([`HAND_YIELD`]) and destroys the block's whole
//!   share, so the richer the deposit, the more a pickaxe wastes.
//! - **Miners** draw from the pool. Every time the pool loses one block's worth of ore, the ore block
//!   nearest the drawing miner turns into spent rock, so extraction visibly eats the deposit.
//! - Each deposit has a **draw cap** shared by all its miners, and output **tapers** over the last
//!   part of the reserve instead of stopping dead.

use rustc_hash::FxHashMap;

use crate::block::{self, BlockId, SPENT_ROCK};
use crate::bytes::ByteWriter;
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::{hash3, sort_small_by_key, unit, IVec3};
use crate::world::World;
use crate::worldgen::WORLD_HEIGHT_CHUNKS;

/// Ore items kept when an ore block is mined by hand, whatever the deposit's grade.
pub const HAND_YIELD: u32 = 3;
/// Output starts falling once less than this fraction of the reserve is left...
const TAPER_START: f64 = 0.2;
/// ...down to this fraction of full speed for the final stretch.
const TAPER_FLOOR: f64 = 0.25;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum Tier {
    Lode = 0,
    Vein = 1,
    Outcrop = 2,
}

impl Tier {
    pub fn name(self) -> &'static str {
        match self {
            Tier::Lode => "lode",
            Tier::Vein => "vein",
            Tier::Outcrop => "outcrop",
        }
    }

    /// Ore units per block.
    pub fn grade(self) -> u32 {
        match self {
            Tier::Lode => 2000,
            Tier::Vein => 1000,
            Tier::Outcrop => 100,
        }
    }

    /// Most ore units per second all miners on one deposit can draw together.
    pub fn draw_cap(self) -> f64 {
        match self {
            Tier::Lode => 20.0,
            Tier::Vein => 4.0,
            Tier::Outcrop => 1.0,
        }
    }

    pub fn from_u8(v: u8) -> Option<Tier> {
        match v {
            0 => Some(Tier::Lode),
            1 => Some(Tier::Vein),
            2 => Some(Tier::Outcrop),
            _ => None,
        }
    }
}

/// Identifies a deposit by where world generation seeded it. The field order is also the
/// ownership order: where shapes overlap, the smaller key owns the block (so lodes win over veins,
/// and veins over outcrops).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct DepositKey {
    pub tier: Tier,
    /// Chunk column the deposit was seeded from.
    pub cx: i32,
    pub cz: i32,
    pub ore: BlockId,
    pub index: u16,
}

impl DepositKey {
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u8(self.tier as u8);
        w.i32(self.cx);
        w.i32(self.cz);
        w.u8(self.ore);
        w.u16(self.index);
    }
}

/// Deposit geometry: a ragged ellipsoid.
#[derive(Clone, Copy, Debug)]
pub struct Deposit {
    pub key: DepositKey,
    pub center: IVec3,
    pub radii: [f32; 3],
    pub seed: u32,
}

/// How far past the nominal radius the ragged edge can reach (sqrt of the 1.4 threshold below).
const EDGE_REACH: f32 = 1.19;

impl Deposit {
    pub fn ore(&self) -> BlockId {
        self.key.ore
    }

    pub fn tier(&self) -> Tier {
        self.key.tier
    }

    /// Inclusive bounding box of every block the shape can contain.
    pub fn bounds(&self) -> (IVec3, IVec3) {
        let r = |i: usize| (self.radii[i] * EDGE_REACH).ceil() as i32;
        let e = IVec3::new(r(0), r(1), r(2));
        (self.center - e, self.center + e)
    }

    pub fn intersects(&self, lo: IVec3, hi: IVec3) -> bool {
        let (a, b) = self.bounds();
        a.x <= hi.x && b.x >= lo.x && a.y <= hi.y && b.y >= lo.y && a.z <= hi.z && b.z >= lo.z
    }

    /// Shape test. The per-block threshold jitter makes the surface ragged instead of a smooth ball.
    #[inline]
    pub fn contains(&self, p: IVec3) -> bool {
        let d = p - self.center;
        let q = |v: i32, r: f32| {
            let t = v as f32 / r;
            t * t
        };
        let dist = q(d.x, self.radii[0]) + q(d.y, self.radii[1]) + q(d.z, self.radii[2]);
        if dist > 1.4 {
            return false;
        }
        dist <= 0.6 + 0.8 * unit(hash3(self.seed, p.x, p.y, p.z)) as f32
    }

    /// Display name, e.g. "Iron vein".
    pub fn name(&self) -> String {
        format!("{} {}", block::ore_label(self.ore()), self.tier().name())
    }
}

/// Tracked state of a deposit that has been looked at, mined or drilled.
pub struct DepositState {
    pub deposit: Deposit,
    /// Every position that was generated as this deposit's ore.
    members: Vec<IVec3>,
    pub initial_blocks: u32,
    pub remaining_blocks: u32,
    /// Units drawn towards converting the next block into spent rock.
    partial: f64,
    /// Draw budget left in the current factory tick, shared by all miners on this deposit.
    budget: f64,
    budget_tick: u64,
}

impl DepositState {
    /// The deposit's figures as the world stands now, computed without tracking it. Readouts use this
    /// for deposits nobody has touched; it costs a generation pass over the deposit's chunks.
    pub fn survey(world: &mut World, d: Deposit) -> DepositState {
        let members = members_of(world, &d);
        let ore = d.ore();
        let remaining = members.iter().filter(|&&p| world.block_anywhere(p).unwrap_or(ore) == ore).count() as u32;
        DepositState {
            deposit: d,
            initial_blocks: members.len() as u32,
            remaining_blocks: remaining,
            members,
            partial: 0.0,
            budget: 0.0,
            budget_tick: u64::MAX,
        }
    }

    pub fn grade(&self) -> u32 {
        self.deposit.tier().grade()
    }

    pub fn remaining_units(&self) -> f64 {
        (self.remaining_blocks as f64 * self.grade() as f64 - self.partial).max(0.0)
    }

    pub fn initial_units(&self) -> f64 {
        self.initial_blocks as f64 * self.grade() as f64
    }

    pub fn exhausted(&self) -> bool {
        self.remaining_blocks == 0
    }

    /// Output multiplier: 1 until the last [`TAPER_START`] of the reserve, then falling to [`TAPER_FLOOR`].
    pub fn taper(&self) -> f64 {
        let initial = self.initial_units();
        if initial <= 0.0 {
            return 0.0;
        }
        (self.remaining_units() / initial / TAPER_START).clamp(TAPER_FLOOR, 1.0)
    }

    /// Removes up to `want` units (already tapered) within this tick's shared budget, turning blocks
    /// nearest `near` into spent rock as whole shares are used up. Returns the units drawn.
    fn draw(&mut self, world: &mut World, want: f64, near: IVec3, tick: u64, dt: f64) -> f64 {
        if self.exhausted() || want <= 0.0 {
            return 0.0;
        }
        if self.budget_tick != tick {
            self.budget_tick = tick;
            self.budget = self.deposit.tier().draw_cap() * self.taper() * dt;
        }
        let amount = want.min(self.budget).min(self.remaining_units());
        if amount <= 0.0 {
            return 0.0;
        }
        self.budget -= amount;
        self.partial += amount;
        let grade = self.grade() as f64;
        while self.partial >= grade && self.remaining_blocks > 0 {
            self.partial -= grade;
            self.remaining_blocks -= 1;
            match self.nearest_ore(world, near) {
                Some(p) => {
                    world.set_block_anywhere(p, SPENT_ROCK);
                }
                // The blocks no longer match the count (edited some other way): trust the world.
                None => self.remaining_blocks = 0,
            }
        }
        if self.remaining_blocks == 0 {
            self.partial = 0.0;
        }
        amount
    }

    fn nearest_ore(&self, world: &World, near: IVec3) -> Option<IVec3> {
        let ore = self.deposit.ore();
        self.members.iter().copied().filter(|&p| world.block_anywhere(p).unwrap_or(ore) == ore).min_by_key(|&p| {
            let d = p - near;
            d.x * d.x + d.y * d.y + d.z * d.z
        })
    }
}

#[derive(Default)]
pub struct Deposits {
    states: FxHashMap<DepositKey, DepositState>,
}

impl Deposits {
    pub fn get(&self, key: &DepositKey) -> Option<&DepositState> {
        self.states.get(key)
    }

    pub fn tracked(&self) -> usize {
        self.states.len()
    }

    /// Core state: every tracked deposit, sorted by key, with what is left of it. Its geometry and
    /// members follow from generation; the draw budget only lives within a tick.
    pub fn write_state(&self, w: &mut ByteWriter) {
        let mut states: Vec<&DepositState> = self.states.values().collect();
        sort_small_by_key(&mut states, |s| s.deposit.key);
        w.count(states.len());
        for s in states {
            s.deposit.key.write_state(w);
            w.u32(s.remaining_blocks);
            w.f64(s.partial);
        }
    }

    /// The deposit owning the ore (or spent rock) block at `p`, starting to track it if needed.
    pub fn lookup(&mut self, world: &mut World, p: IVec3) -> Option<DepositKey> {
        let d = owner_of(world, p)?;
        self.states.entry(d.key).or_insert_with(|| DepositState::survey(world, d));
        Some(d.key)
    }

    /// Hand mining: the block at `p` is about to be broken and its whole share is lost.
    /// Call before removing the block.
    pub fn hand_mined(&mut self, world: &mut World, p: IVec3) {
        if let Some(key) = self.lookup(world, p) {
            let st = self.states.get_mut(&key).expect("lookup tracks the deposit");
            st.remaining_blocks = st.remaining_blocks.saturating_sub(1);
            if st.remaining_blocks == 0 {
                st.partial = 0.0;
            }
        }
    }

    /// Draws for a miner at `near` wanting `rate` units per second for `dt` seconds.
    pub fn draw(&mut self, world: &mut World, key: &DepositKey, rate: f64, near: IVec3, tick: u64, dt: f64) -> f64 {
        match self.states.get_mut(key) {
            Some(st) => {
                let want = rate * st.taper() * dt;
                st.draw(world, want, near, tick, dt)
            }
            None => 0.0,
        }
    }
}

/// The deposit owning the ore (or spent rock) block at `p`, loaded or not. Tracks nothing, so
/// queries can use it; `&mut` only for the world generator's caches.
pub fn owner_of(world: &mut World, p: IVec3) -> Option<Deposit> {
    let b = world.block_anywhere_or_generate(p);
    if !block::is_ore(b) && b != SPENT_ROCK {
        return None;
    }
    let d = world.generator_mut().deposit_at(p)?;
    if b != SPENT_ROCK && d.ore() != b {
        return None;
    }
    Some(d)
}

/// Every block world generation made into this deposit's ore: inside the shape, generated as this
/// ore, and not claimed by an overlapping deposit with a smaller key.
fn members_of(world: &mut World, d: &Deposit) -> Vec<IVec3> {
    let (lo, hi) = d.bounds();
    let rivals: Vec<Deposit> =
        world.generator_mut().deposits_touching(lo, hi).into_iter().filter(|o| o.key < d.key).collect();
    let mut out = Vec::new();
    let c0 = IVec3::new(lo.x >> CHUNK_SHIFT, (lo.y >> CHUNK_SHIFT).max(0), lo.z >> CHUNK_SHIFT);
    let c1 = IVec3::new(hi.x >> CHUNK_SHIFT, (hi.y >> CHUNK_SHIFT).min(WORLD_HEIGHT_CHUNKS - 1), hi.z >> CHUNK_SHIFT);
    for cy in c0.y..=c1.y {
        for cz in c0.z..=c1.z {
            for cx in c0.x..=c1.x {
                let c = IVec3::new(cx, cy, cz);
                let original = world.original_chunk(c);
                let base = IVec3::new(cx * CHUNK_SIZE, cy * CHUNK_SIZE, cz * CHUNK_SIZE);
                let from = IVec3::new(lo.x.max(base.x), lo.y.max(base.y), lo.z.max(base.z));
                let to = IVec3::new(
                    hi.x.min(base.x + CHUNK_SIZE - 1),
                    hi.y.min(base.y + CHUNK_SIZE - 1),
                    hi.z.min(base.z + CHUNK_SIZE - 1),
                );
                for y in from.y..=to.y {
                    for z in from.z..=to.z {
                        for x in from.x..=to.x {
                            let p = IVec3::new(x, y, z);
                            let local = p - base;
                            if original.get(local.x as usize, local.y as usize, local.z as usize) == d.ore()
                                && d.contains(p)
                                && !rivals.iter().any(|o| o.contains(p))
                            {
                                out.push(p);
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
