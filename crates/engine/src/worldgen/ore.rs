//! Ore deposit placement: seeding outcrops, veins and lodes per chunk column, stamping their ore
//! into generated chunks, and the ownership lookups built on the same seeding.
//!
//! Invariants: a deposit's shape stays within one column of where it was seeded, so a chunk only
//! needs its 3x3 column neighbourhood; deposits are stamped in ownership order (`DepositKey`
//! order) and never overwrite ore, so where shapes overlap the smallest key owns the block, both
//! when stamping and in [`WorldGen::deposit_at`]. Runtime deposit state lives in `crate::deposits`.
//! To add an ore: a row in [`ORE_GEN`] plus its block in `block.rs`.

use crate::block::{BlockId, COAL_ORE, COPPER_ORE, DIRT, GRASS, IRON_ORE, STONE};
use crate::chunk::{index, CHUNK_SIZE};
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::math::{hash2, hash3, sort_small_by_key, IVec3, Rng};

use super::{WorldGen, WORLD_HEIGHT};

/// No surface outcrops this close to spawn, so the player starts on plain ground.
const ORE_SPAWN_CLEARING: i32 = 10;
/// Per ore: (block, outcrops per chunk column, chance of a vein per column, weight when picking a lode's ore).
const ORE_GEN: [(BlockId, u32, f64, u32); 3] =
    [(COAL_ORE, 4, 0.5, 1), (IRON_ORE, 4, 0.45, 2), (COPPER_ORE, 3, 0.35, 1)];
/// Chance that a chunk column seeds a lode (roughly one per 40 columns, i.e. per ~200 x 200 blocks).
const LODE_CHANCE: f64 = 1.0 / 40.0;

impl WorldGen {
    /// The deposit that owns the ore block at `p`: the first one in ownership order whose shape
    /// contains it (the same one that stamped it during generation).
    pub fn deposit_at(&mut self, p: IVec3) -> Option<Deposit> {
        let col = self.column(p.x >> 5, p.z >> 5);
        col.deposits.iter().find(|d| d.contains(p)).copied()
    }

    /// Every deposit whose bounds overlap the box `lo..=hi`, in ownership order.
    pub fn deposits_touching(&mut self, lo: IVec3, hi: IVec3) -> Vec<Deposit> {
        let mut out: Vec<Deposit> = Vec::new();
        for cz in (lo.z >> 5)..=(hi.z >> 5) {
            for cx in (lo.x >> 5)..=(hi.x >> 5) {
                let col = self.column(cx, cz);
                out.extend(col.deposits.iter().filter(|d| d.intersects(lo, hi)));
            }
        }
        sort_small_by_key(&mut out, |d| d.key);
        out.dedup_by_key(|d| d.key);
        out
    }

    /// Nearest deposit of `tier` seeded within `radius` chunk columns of `near` (a prospecting aid).
    pub fn find_deposit(&self, near: IVec3, tier: Tier, radius: i32) -> Option<Deposit> {
        let (pcx, pcz) = (near.x >> 5, near.z >> 5);
        let mut seeded = Vec::new();
        let mut best: Option<(i64, Deposit)> = None;
        for cz in pcz - radius..=pcz + radius {
            for cx in pcx - radius..=pcx + radius {
                seeded.clear();
                self.seed_deposits(cx, cz, &mut seeded);
                for d in seeded.iter().filter(|d| d.tier() == tier) {
                    let v = d.center - near;
                    let dist = (v.x as i64).pow(2) + (v.y as i64).pow(2) + (v.z as i64).pow(2);
                    if best.as_ref().is_none_or(|(b, _)| dist < *b) {
                        best = Some((dist, *d));
                    }
                }
            }
        }
        best.map(|(_, d)| d)
    }

    /// Deposits from this column's 3x3 neighbourhood that reach into it, in ownership order.
    pub(super) fn deposits_for_column(&self, cx: i32, cz: i32) -> Vec<Deposit> {
        let mut all = Vec::new();
        for dz in -1..=1 {
            for dx in -1..=1 {
                self.seed_deposits(cx + dx, cz + dz, &mut all);
            }
        }
        let lo = IVec3::new(cx * CHUNK_SIZE, 0, cz * CHUNK_SIZE);
        let hi = IVec3::new(lo.x + CHUNK_SIZE - 1, WORLD_HEIGHT - 1, lo.z + CHUNK_SIZE - 1);
        all.retain(|d| d.intersects(lo, hi));
        sort_small_by_key(&mut all, |d| d.key);
        all
    }

    /// Every deposit seeded in one chunk column.
    fn seed_deposits(&self, cx: i32, cz: i32, out: &mut Vec<Deposit>) {
        let (x0, z0) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);
        for (oi, &(ore, outcrops, vein_chance, _)) in ORE_GEN.iter().enumerate() {
            let mut rng = Rng::new(hash3(self.seed ^ 0xDE90, cx, oi as i32, cz) as u64);
            let key = |tier, index| DepositKey { tier, cx, cz, ore, index };
            for i in 0..outcrops {
                let x = x0 + rng.below(32) as i32;
                let z = z0 + rng.below(32) as i32;
                let surface = self.height_at(x, z);
                // Half sit right at the surface (visible patches), half are pockets deeper down.
                let y = if rng.next_f64() < 0.5 {
                    surface - rng.below(4) as i32
                } else {
                    8 + rng.below((surface - 14).max(1) as u32) as i32
                };
                let r = rng.range(1.3, 2.3) as f32;
                let squash = rng.range(0.7, 1.0) as f32;
                let seed = rng.next_u32();
                if x.abs() < ORE_SPAWN_CLEARING && z.abs() < ORE_SPAWN_CLEARING {
                    continue;
                }
                out.push(Deposit {
                    key: key(Tier::Outcrop, i as u16),
                    center: IVec3::new(x, y, z),
                    radii: [r, r * squash, r],
                    seed,
                });
            }
            if rng.next_f64() < vein_chance {
                let x = x0 + rng.below(32) as i32;
                let z = z0 + rng.below(32) as i32;
                let y = (self.height_at(x, z) - rng.range(20.0, 60.0) as i32).max(12);
                let major = rng.range(4.5, 7.0) as f32;
                let minor = rng.range(2.0, 3.0) as f32;
                let tall = rng.range(1.8, 2.6) as f32;
                let radii = if rng.below(2) == 0 { [major, tall, minor] } else { [minor, tall, major] };
                out.push(Deposit { key: key(Tier::Vein, 0), center: IVec3::new(x, y, z), radii, seed: rng.next_u32() });
            }
        }

        let mut rng = Rng::new(hash2(self.seed ^ 0x10DE, cx, cz) as u64);
        if rng.next_f64() < LODE_CHANCE {
            let total: u32 = ORE_GEN.iter().map(|o| o.3).sum();
            let mut pick = rng.below(total);
            let mut ore = ORE_GEN[0].0;
            for &(o, _, _, weight) in &ORE_GEN {
                if pick < weight {
                    ore = o;
                    break;
                }
                pick -= weight;
            }
            let x = x0 + rng.below(32) as i32;
            let z = z0 + rng.below(32) as i32;
            let y = 12 + rng.below(14) as i32;
            let r = rng.range(7.0, 8.5) as f32;
            out.push(Deposit {
                key: DepositKey { tier: Tier::Lode, cx, cz, ore, index: 0 },
                center: IVec3::new(x, y, z),
                radii: [r, r * 0.75, r],
                seed: rng.next_u32(),
            });
        }
    }
}

/// Writes a deposit's ore into the chunk at `base`. Deposits are stamped in ownership order and never
/// overwrite ore, so where shapes overlap the first (smallest key) wins, matching [`WorldGen::deposit_at`].
pub(super) fn stamp_deposit(d: &Deposit, base: IVec3, b: &mut [BlockId]) {
    let (lo, hi) = d.bounds();
    let from = |v: i32, o: i32| (v - o).max(0);
    let to = |v: i32, o: i32| (v - o).min(CHUNK_SIZE - 1);
    for y in from(lo.y, base.y)..=to(hi.y, base.y) {
        for z in from(lo.z, base.z)..=to(hi.z, base.z) {
            for x in from(lo.x, base.x)..=to(hi.x, base.x) {
                let i = index(x as usize, y as usize, z as usize);
                if ore_replaceable(b[i]) && d.contains(base + IVec3::new(x, y, z)) {
                    b[i] = d.ore();
                }
            }
        }
    }
}

/// Blocks a deposit may turn into ore. Outcrops that reach the surface replace grass and dirt, which
/// is what makes them visible.
#[inline]
fn ore_replaceable(b: BlockId) -> bool {
    matches!(b, STONE | DIRT | GRASS)
}
