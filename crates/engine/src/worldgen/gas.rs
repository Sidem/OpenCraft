//! Natural gas fields (generator version 13, CHEM-GAS): deep `GAS_SHALE` veins and lodes placed on their own
//! independent hash/RNG streams, completely separate from `ore_at` and `EXTRAS` (`geology.rs`).
//!
//! Invariant: upgrading a world's `deposits` layer from v6–v12 to v13 adds gas fields in unedited chunks without
//! moving, re-rolling or removing a single existing ore deposit (`worldgen/layers/tests.rs` verifies this).
//! Gas fields occur under Plains, Lowlands and Desert at least `GAS_FROM` blocks from spawn, buried 35–75 blocks
//! down (or near bedrock for lodes), and leave no surface stain — players scan for them with a Scanner.

use crate::block::GAS_SHALE;
use crate::chunk::CHUNK_SIZE;
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::math::{hash2, IVec3, Rng};

use super::ore::lode_shape;
use super::{Biome, WorldGen};

/// The generator `deposits` version that introduced natural gas fields.
pub const GAS_SINCE: u32 = 13;
/// How far from spawn (blocks) gas fields first appear.
pub const GAS_FROM: i32 = 220;
/// Depth band (blocks below local surface) for gas shale veins.
pub const GAS_BAND: (i32, i32) = (35, 75);
/// Distinct `DepositKey::index` so gas keys never collide with outcrop/vein/starter indices.
const GAS_INDEX: u16 = 200;
/// Chance per qualifying chunk column of seeding a gas vein (~1 per 16 qualifying columns).
const GAS_VEIN_CHANCE: f64 = 0.06;
/// Chance per qualifying chunk column of seeding a deep gas lode near bedrock.
const GAS_LODE_CHANCE: f64 = 0.008;

/// Whether `biome` at horizontal position `at` can host a natural gas field.
pub(super) fn holds_gas(biome: Biome, at: IVec3) -> bool {
    let far = (at.x as i64).pow(2) + (at.z as i64).pow(2) >= (GAS_FROM as i64).pow(2);
    far && matches!(biome, Biome::Plains | Biome::Lowlands | Biome::Desert)
}

impl WorldGen {
    /// Seeds version 13's independent gas fields for chunk column `(cx, cz)`.
    pub(super) fn seed_gas_fields(&self, cx: i32, cz: i32, out: &mut Vec<Deposit>) {
        if self.layers.deposits < GAS_SINCE {
            return;
        }
        let (x0, z0) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);

        let mut rng = Rng::new(hash2(self.seed ^ 0x6A5F, cx, cz) as u64);
        if rng.next_f64() < GAS_VEIN_CHANCE {
            let x = x0 + rng.below(32) as i32;
            let z = z0 + rng.below(32) as i32;
            let h = self.height_at(x, z);
            let biome = self.biome_at(x, z, h);
            if holds_gas(biome, IVec3::new(x, h, z)) {
                let depth = GAS_BAND.0 + rng.below((GAS_BAND.1 - GAS_BAND.0 + 1) as u32) as i32;
                let y = (h - depth).max(12);
                let major = rng.range(5.0, 7.5) as f32;
                let minor = rng.range(2.5, 3.8) as f32;
                let tall = rng.range(1.8, 2.6) as f32;
                let radii = if rng.below(2) == 0 { [major, tall, minor] } else { [minor, tall, major] };
                out.push(Deposit {
                    key: DepositKey { tier: Tier::Vein, cx, cz, ore: GAS_SHALE, index: GAS_INDEX },
                    center: IVec3::new(x, y, z),
                    radii,
                    seed: rng.next_u32(),
                });
            }
        }

        let mut rng = Rng::new(hash2(self.seed ^ 0x6A5E, cx, cz) as u64);
        if rng.next_f64() < GAS_LODE_CHANCE {
            let (center, radii, seed) = lode_shape(&mut rng, x0, z0);
            let h = self.height_at(center.x, center.z);
            let biome = self.biome_at(center.x, center.z, h);
            if holds_gas(biome, center) {
                out.push(Deposit {
                    key: DepositKey { tier: Tier::Lode, cx, cz, ore: GAS_SHALE, index: GAS_INDEX },
                    center,
                    radii,
                    seed,
                });
            }
        }
    }
}
