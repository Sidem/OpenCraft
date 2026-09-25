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
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::{hash3, unit, IVec3};
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

    /// The deposit owning the ore (or spent rock) block at `p`, starting to track it if needed.
    pub fn lookup(&mut self, world: &mut World, p: IVec3) -> Option<DepositKey> {
        let b = world.get_block(p)?;
        if !block::is_ore(b) && b != SPENT_ROCK {
            return None;
        }
        let d = world.generator_mut().deposit_at(p)?;
        if b != SPENT_ROCK && d.ore() != b {
            return None;
        }
        self.ensure(world, d);
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

    fn ensure(&mut self, world: &mut World, d: Deposit) {
        if self.states.contains_key(&d.key) {
            return;
        }
        let members = members_of(world, &d);
        let ore = d.ore();
        let remaining = members.iter().filter(|&&p| world.block_anywhere(p).unwrap_or(ore) == ore).count() as u32;
        self.states.insert(
            d.key,
            DepositState {
                deposit: d,
                initial_blocks: members.len() as u32,
                remaining_blocks: remaining,
                members,
                partial: 0.0,
                budget: 0.0,
                budget_tick: u64::MAX,
            },
        );
    }
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
mod tests {
    use super::*;
    use crate::block::IRON_ORE;

    fn deposit(tier: Tier, r: f32) -> Deposit {
        Deposit {
            key: DepositKey { tier, cx: 0, cz: 0, ore: IRON_ORE, index: 0 },
            center: IVec3::new(0, 40, 0),
            radii: [r, r, r],
            seed: 7,
        }
    }

    #[test]
    fn shape_stays_inside_bounds() {
        let d = deposit(Tier::Vein, 4.3);
        let (lo, hi) = d.bounds();
        let mut inside = 0;
        for y in lo.y - 3..=hi.y + 3 {
            for z in lo.z - 3..=hi.z + 3 {
                for x in lo.x - 3..=hi.x + 3 {
                    let p = IVec3::new(x, y, z);
                    if d.contains(p) {
                        inside += 1;
                        assert!(p.x >= lo.x && p.x <= hi.x && p.y >= lo.y && p.y <= hi.y && p.z >= lo.z && p.z <= hi.z);
                    }
                }
            }
        }
        // Roughly a ball of radius 4.3 (volume ~333), give or take the ragged edge.
        assert!((200..500).contains(&inside), "{inside} blocks");
    }

    #[test]
    fn ownership_prefers_bigger_tiers() {
        let lode = DepositKey { tier: Tier::Lode, cx: 5, cz: 5, ore: IRON_ORE, index: 0 };
        let outcrop = DepositKey { tier: Tier::Outcrop, cx: -5, cz: -5, ore: IRON_ORE, index: 0 };
        assert!(lode < outcrop);
    }

    /// First generated ore block of `tier` found scanning chunks near the origin.
    fn find_ore(world: &mut World, tier: Tier) -> (IVec3, Deposit) {
        for cz in -3..=3 {
            for cx in -3..=3 {
                for cy in 0..WORLD_HEIGHT_CHUNKS {
                    let c = IVec3::new(cx, cy, cz);
                    let chunk = world.original_chunk(c);
                    for i in 0..crate::chunk::CHUNK_VOLUME {
                        let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
                        if !block::is_ore(chunk.get(x, y, z)) {
                            continue;
                        }
                        let p = IVec3::new(cx * 32 + x as i32, cy * 32 + y as i32, cz * 32 + z as i32);
                        let d = world.generator_mut().deposit_at(p).expect("every ore block has an owner");
                        if d.tier() == tier {
                            return (p, d);
                        }
                    }
                }
            }
        }
        panic!("no {tier:?} near the origin");
    }

    #[test]
    fn members_are_exactly_the_blocks_a_deposit_owns() {
        let mut world = World::new(11, 2);
        for tier in [Tier::Outcrop, Tier::Vein] {
            let (p, d) = find_ore(&mut world, tier);
            let mut deps = Deposits::default();
            deps.ensure(&mut world, d);
            let st = deps.get(&d.key).unwrap();
            assert!(st.members.contains(&p));
            assert_eq!(st.remaining_blocks, st.initial_blocks, "nothing has been mined yet");
            for &m in &st.members {
                assert_eq!(world.generator_mut().deposit_at(m).unwrap().key, d.key);
            }
            println!("{} has {} blocks", d.name(), st.initial_blocks);
        }
    }

    #[test]
    fn drawing_converts_the_nearest_block_even_while_unloaded() {
        let mut world = World::new(11, 2);
        let (p, d) = find_ore(&mut world, Tier::Outcrop);
        let mut deps = Deposits::default();
        deps.ensure(&mut world, d);
        let blocks = deps.get(&d.key).unwrap().remaining_blocks;
        let grade = Tier::Outcrop.grade() as f64;
        let cap = Tier::Outcrop.draw_cap();

        // A generous miner is still held to the outcrop's draw cap.
        let mut drawn = 0.0;
        for tick in 0..(grade as u64 + 5) {
            drawn += deps.draw(&mut world, &d.key, 50.0, p, tick, 1.0);
        }
        assert!((drawn - cap * (grade + 5.0)).abs() < 1e-6, "drawn {drawn}");
        let st = deps.get(&d.key).unwrap();
        assert_eq!(st.remaining_blocks, blocks - 1);
        assert_eq!(world.block_anywhere(p), Some(SPENT_ROCK), "the block at the drill goes first");
    }

    #[test]
    fn taper_slows_the_last_fifth() {
        let mut st = DepositState {
            deposit: deposit(Tier::Outcrop, 2.0),
            members: Vec::new(),
            initial_blocks: 100,
            remaining_blocks: 100,
            partial: 0.0,
            budget: 0.0,
            budget_tick: 0,
        };
        assert_eq!(st.taper(), 1.0);
        st.remaining_blocks = 20;
        assert!((st.taper() - 1.0).abs() < 1e-9);
        st.remaining_blocks = 10;
        assert!((st.taper() - 0.5).abs() < 1e-9);
        st.remaining_blocks = 1;
        assert_eq!(st.taper(), TAPER_FLOOR);
    }
}
