//! Version 3 deposit seeding (Milestone 5): as many outcrops, veins and lodes as version 2, but ore is
//! rare at the surface and every ore keeps to its own depth.
//!
//! - An outcrop slot shows at the surface only with `EXPOSED_CHANCE`, and only where rock is bare
//!   ([`WorldGen::bare_rock`]: cliffs, bare mountain tops, deserts, basalt fields); every other slot is a
//!   pocket buried in its ore's band. Veins sit in their ore's band too ([`ORE_DEPTH`], below the local
//!   surface); lodes stay near bedrock. The ore is picked first (by biome, `geology.rs`), then the depth.
//! - A starter set ([`WorldGen::starter_outcrops`]): one exposed coal, iron and copper outcrop 40–80
//!   blocks from spawn on dry, gentle ground, seeded in the column it falls in with key index
//!   `STARTER_INDEX + i`, so `deposit_by_key` finds it again.
//!
//! Version 4 (a play-test found starter ore too hard to find) makes the same draws with other numbers:
//! shallower bands (`ORE_DEPTH_V4`, so a stain's vein is in reach of a hand-dug shaft), coal, iron and
//! copper exposed more often (`EXPOSED_CHANCE_METALS`), and two starter patches of each (`STARTERS_V4`).
//!
//! Invariants: a pure function of the seed and the column, like versions 1 and 2; keys stay unique per
//! (column, tier, slot). Lodes use version 2's draws exactly. To tune: the constants below (version 4's
//! only: version 3 is released).

use crate::block::{BlockId, BAUXITE_ORE, COAL_ORE, COPPER_ORE, IRON_ORE, LIMESTONE, QUARTZ_ORE};
use crate::chunk::CHUNK_SIZE;
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::math::{hash2, IVec3, Rng};

use super::ore::{lode_shape, LODE_CHANCE, ORE_SPAWN_CLEARING};
use super::{Biome, WorldGen, CLIFF_SLOPE, ROCK_LEVEL, SEA_LEVEL};

/// Outcrop slots per chunk column (as in version 2).
const OUTCROPS: u32 = 11;
/// Chance of each vein slot per column (as in version 2).
const VEIN_CHANCES: [f64; 2] = [1.0, 0.3];
/// Chance that an outcrop slot on bare rock shows at the surface (about 0.3 exposed per column).
const EXPOSED_CHANCE: f64 = 0.28;
/// Per ore: the band (blocks below the local surface) its buried pockets and veins sit in.
pub(super) const ORE_DEPTH: [(BlockId, i32, i32); 5] =
    [(LIMESTONE, 8, 25), (COAL_ORE, 10, 35), (IRON_ORE, 20, 50), (COPPER_ORE, 35, 70), (QUARTZ_ORE, 35, 70)];
/// Bauxite (version 5) lies shallow, in the laterite just under a far desert or basalt field.
pub(super) const ORE_DEPTH_V4: [(BlockId, i32, i32); 6] = [
    (LIMESTONE, 6, 20),
    (COAL_ORE, 6, 22),
    (IRON_ORE, 8, 28),
    (COPPER_ORE, 12, 34),
    (QUARTZ_ORE, 25, 50),
    (BAUXITE_ORE, 4, 16),
];
/// Version 4: the chance for a coal, iron or copper outcrop slot on bare rock to show.
const EXPOSED_CHANCE_METALS: f64 = 0.42;
/// Nothing buried is placed below this height (bedrock lies under it).
const DEEPEST: i32 = 8;
/// The starter set: its ores, how far from spawn (blocks) each lies, and its radii.
type Starter = (BlockId, (i32, i32));
const STARTERS: [Starter; 3] = [(COAL_ORE, (40, 80)), (IRON_ORE, (40, 80)), (COPPER_ORE, (40, 80))];
const STARTER_RADII: [f32; 3] = [2.2, 1.8, 2.2];
const STARTERS_V4: [Starter; 6] = [
    (COAL_ORE, (28, 60)),
    (IRON_ORE, (28, 60)),
    (COPPER_ORE, (28, 60)),
    (IRON_ORE, (60, 110)),
    (COPPER_ORE, (60, 110)),
    (COAL_ORE, (60, 110)),
];
const STARTER_RADII_V4: [f32; 3] = [2.6, 2.0, 2.6];
pub(super) const STARTER_INDEX: u16 = 100;
/// Directions the starter search tries, as (x, z) in thousandths: 16 around the circle.
pub(super) const DIRS16: [(i32, i32); 16] = [
    (1000, 0),
    (924, 383),
    (707, 707),
    (383, 924),
    (0, 1000),
    (-383, 924),
    (-707, 707),
    (-924, 383),
    (-1000, 0),
    (-924, -383),
    (-707, -707),
    (-383, -924),
    (0, -1000),
    (383, -924),
    (707, -707),
    (924, -383),
];

impl WorldGen {
    /// Version 3's deposits seeded in one chunk column.
    pub(super) fn seed_deposits_v3(&self, cx: i32, cz: i32, out: &mut Vec<Deposit>) {
        let (x0, z0) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);
        let key = |tier, ore, index: u32| DepositKey { tier, cx, cz, ore, index: index as u16 };
        let mut rng = Rng::new(hash2(self.seed ^ 0x6E03, cx, cz) as u64);
        for i in 0..OUTCROPS {
            let (x, z) = (x0 + rng.below(32) as i32, z0 + rng.below(32) as i32);
            let h = self.height_at(x, z);
            let ore = self.ore_at(&mut rng, IVec3::new(x, h, z));
            let exposed = rng.next_f64() < self.exposed_chance(ore) && self.bare_rock(x, z);
            let r = rng.range(1.3, 2.3) as f32;
            let squash = rng.range(0.7, 1.0) as f32;
            let depth = band_depth(&mut rng, self.ore_band(ore));
            let seed = rng.next_u32();
            if x.abs() < ORE_SPAWN_CLEARING && z.abs() < ORE_SPAWN_CLEARING {
                continue;
            }
            // An exposed outcrop's top block always shows (see `Deposit::contains`).
            let (y, ry) =
                if exposed { (h - (seed & 1) as i32, (r * squash).max(1.3)) } else { (buried(h, depth), r * squash) };
            out.push(Deposit { key: key(Tier::Outcrop, ore, i), center: IVec3::new(x, y, z), radii: [r, ry, r], seed });
        }
        for (i, &chance) in VEIN_CHANCES.iter().enumerate() {
            if rng.next_f64() < chance {
                let (x, z) = (x0 + rng.below(32) as i32, z0 + rng.below(32) as i32);
                let h = self.height_at(x, z);
                let ore = self.ore_at(&mut rng, IVec3::new(x, h, z));
                let y = buried(h, band_depth(&mut rng, self.ore_band(ore))).max(12);
                let major = rng.range(4.5, 7.0) as f32;
                let minor = rng.range(2.0, 3.0) as f32;
                let tall = rng.range(1.8, 2.6) as f32;
                let radii = if rng.below(2) == 0 { [major, tall, minor] } else { [minor, tall, major] };
                let center = IVec3::new(x, y, z);
                out.push(Deposit { key: key(Tier::Vein, ore, i as u32), center, radii, seed: rng.next_u32() });
            }
        }
        // Lodes exactly as version 2 draws them.
        let mut rng = Rng::new(hash2(self.seed ^ 0x10DF, cx, cz) as u64);
        if rng.next_f64() < LODE_CHANCE {
            let (center, radii, seed) = lode_shape(&mut rng, x0, z0);
            let ore = self.ore_at(&mut rng, center);
            out.push(Deposit { key: key(Tier::Lode, ore, 0), center, radii, seed });
        }
        // The starter set only ever lies in the few columns around spawn.
        if cx.abs() <= 4 && cz.abs() <= 4 {
            out.extend(self.starter_outcrops().into_iter().filter(|d| d.key.cx == cx && d.key.cz == cz));
        }
    }

    /// Whether the column at (x, z) shows bare rock (or desert sand over it) on dry land, where an
    /// outcrop may show: a cliff, a mountain top above `ROCK_LEVEL`, a desert or a basalt field.
    pub(super) fn bare_rock(&self, x: i32, z: i32) -> bool {
        let h = self.height_at(x, z);
        h > SEA_LEVEL + 1
            && (self.slope_at(x, z) >= CLIFF_SLOPE
                || h > ROCK_LEVEL
                || matches!(self.biome_at(x, z, h), Biome::Desert | Biome::BasaltFields))
    }

    /// The band (blocks below the local surface) buried pockets and veins of `ore` sit in. Versions 1
    /// and 2 have no bands: their veins lie 20 to 60 blocks down whatever the ore.
    pub(crate) fn ore_band(&self, ore: BlockId) -> (i32, i32) {
        let table = match self.version {
            ..=2 => return (20, 60),
            3 => &ORE_DEPTH[..],
            _ => &ORE_DEPTH_V4[..],
        };
        table.iter().find(|b| b.0 == ore).map_or((10, 40), |b| (b.1, b.2))
    }

    /// How far from spawn (blocks) the starter set lies, nearest and farthest.
    pub(crate) fn starter_reach(&self) -> (i32, i32) {
        let starters = if self.version >= 4 { &STARTERS_V4[..] } else { &STARTERS[..] };
        starters.iter().fold((i32::MAX, 0), |(lo, hi), s| (lo.min(s.1 .0), hi.max(s.1 .1)))
    }

    /// The chance that an outcrop slot of `ore` on bare rock shows at the surface.
    fn exposed_chance(&self, ore: BlockId) -> f64 {
        let metal = matches!(ore, COAL_ORE | IRON_ORE | COPPER_ORE | BAUXITE_ORE);
        if self.version >= 4 && metal {
            EXPOSED_CHANCE_METALS
        } else {
            EXPOSED_CHANCE
        }
    }

    /// The starter outcrops: for each row of `STARTERS` (`STARTERS_V4`), the first dry, gentle column
    /// found walking out through its ring, each starting a third of a turn on from the last.
    pub(super) fn starter_outcrops(&self) -> Vec<Deposit> {
        let (starters, radii) =
            if self.version >= 4 { (&STARTERS_V4[..], STARTER_RADII_V4) } else { (&STARTERS[..], STARTER_RADII) };
        let turn = (hash2(self.seed ^ 0x57A7, 0, 0) % 16) as usize;
        let mut out: Vec<Deposit> = Vec::new();
        for (i, &(ore, ring)) in starters.iter().enumerate() {
            'search: for dist in (ring.0..=ring.1).step_by(4) {
                for step in 0..16 {
                    let (dx, dz) = DIRS16[(turn + i * 5 + step) % 16];
                    let (x, z) = (dx * dist / 1000, dz * dist / 1000);
                    let h = self.height_at(x, z);
                    let apart = out.iter().all(|d| (d.center.x - x).abs() + (d.center.z - z).abs() > 12);
                    if h > SEA_LEVEL + 1 && self.slope_at(x, z) <= 2 && apart {
                        let key = DepositKey {
                            tier: Tier::Outcrop,
                            cx: x >> 5,
                            cz: z >> 5,
                            ore,
                            index: STARTER_INDEX + i as u16,
                        };
                        let seed = hash2(self.seed ^ 0x57A8, x, z);
                        out.push(Deposit { key, center: IVec3::new(x, h - 1, z), radii, seed });
                        break 'search;
                    }
                }
            }
        }
        out
    }
}

/// A depth in a band (`WorldGen::ore_band`).
fn band_depth(rng: &mut Rng, (lo, hi): (i32, i32)) -> i32 {
    lo + rng.below((hi - lo + 1) as u32) as i32
}

/// The height of something buried `depth` below a surface at `h`, kept above bedrock.
fn buried(h: i32, depth: i32) -> i32 {
    (h - depth).max(DEEPEST)
}

#[cfg(test)]
mod tests;
