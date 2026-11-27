//! Seasonal phenology for every tree species, grass, ground cover and fallen leaf litter (SEASON-TREES).
//!
//! Presentation only: driven purely by [`crate::daytime::year_fraction`] (a pure function of the core tick), so
//! co-op peers agree without saved state and nothing ever remeshes as seasons pass. Each species owns one cyclic
//! keyframe row over the year (`0.0` spring day 1, `0.25` summer, `0.50` autumn, `0.75` winter); [`pack_foliage`]
//! and [`pack_ground`] evaluate the current year fraction into flat float slices the WebGL2 renderer uploads as
//! uniforms each frame.
//!
//! Invariants: evergreen species (pine, redwood, eucalyptus, palm, bamboo) always report `cover = 1.0` and
//! `litter_cover = 0.0`; ore-indicator plants are never tagged in [`season_layers`] so they stay readable year-round.
//! To add a species: append a `SPECIES_*` id, add a match arm in [`species_of`], and append its [`Phenology`] row.

use crate::block::{
    tex, BlockId, ACACIA_LEAVES, BAMBOO_LEAVES, BAOBAB_LEAVES, BIRCH_LEAVES, CHERRY_BLOSSOM, EUCALYPTUS_LEAVES,
    PALM_FRONDS, PINE_LEAVES, REDWOOD_NEEDLES,
};

pub const SPECIES_COUNT: usize = 10;
pub const SPECIES_OAK: u8 = 0;
pub const SPECIES_BIRCH: u8 = 1;
pub const SPECIES_PINE: u8 = 2;
pub const SPECIES_CHERRY: u8 = 3;
pub const SPECIES_PALM: u8 = 4;
pub const SPECIES_BAOBAB: u8 = 5;
pub const SPECIES_ACACIA: u8 = 6;
pub const SPECIES_REDWOOD: u8 = 7;
pub const SPECIES_EUCALYPTUS: u8 = 8;
pub const SPECIES_BAMBOO: u8 = 9;

/// Layer-role bits in [`season_layers`] (2 bits per texture layer, up to 2048 layers, matching `block/turn.rs`).
#[cfg(test)]
pub const ROLE_NONE: u32 = 0;
pub const ROLE_GRASS: u32 = 1;
pub const ROLE_WOODLAND: u32 = 2;
pub const ROLE_FLOWER: u32 = 3;

/// Floats per species in [`pack_foliage`]: `leafA` (4), `leafB` (4), `litter` (4).
pub const FOLIAGE_FLOATS: usize = SPECIES_COUNT * 12;
/// Floats in [`pack_ground`]: `grass` (4), `woodland` (4), `flags` (4).
pub const GROUND_FLOATS: usize = 12;

/// Maps any leaf block to its species index `0..SPECIES_COUNT`. Unknown leaf blocks fall back to oak.
#[inline]
pub const fn species_of(b: BlockId) -> u8 {
    match b {
        BIRCH_LEAVES => SPECIES_BIRCH,
        PINE_LEAVES => SPECIES_PINE,
        CHERRY_BLOSSOM => SPECIES_CHERRY,
        PALM_FRONDS => SPECIES_PALM,
        BAOBAB_LEAVES => SPECIES_BAOBAB,
        ACACIA_LEAVES => SPECIES_ACACIA,
        REDWOOD_NEEDLES => SPECIES_REDWOOD,
        EUCALYPTUS_LEAVES => SPECIES_EUCALYPTUS,
        BAMBOO_LEAVES => SPECIES_BAMBOO,
        _ => SPECIES_OAK,
    }
}

/// Whether `b` is a deciduous or dry-season-shedding leaf block that drops fallen litter onto the ground below.
#[inline]
pub const fn sheds_litter(b: BlockId) -> bool {
    matches!(b, crate::block::LEAVES | BIRCH_LEAVES | CHERRY_BLOSSOM | BAOBAB_LEAVES | ACACIA_LEAVES)
}

#[derive(Clone, Copy)]
struct CrownKey {
    t: f32,
    a: [f32; 3],
    b: [f32; 3],
    mix: f32,
    cover: f32,
}

#[derive(Clone, Copy)]
struct LitterKey {
    t: f32,
    /// Target RGB, or `[-1.0, 0.0, 0.0]` to draw the native texture (spring cherry blossom petals).
    col: [f32; 3],
    cover: f32,
}

struct Phenology {
    /// Typical luminance of this species' base leaf texture, used so recolouring keeps natural brightness.
    mid_lum: f32,
    crown: &'static [CrownKey],
    litter: &'static [LitterKey],
}

const NONE_LITTER: &[LitterKey] = &[];
const fn ck(t: f32, a: [f32; 3], b: [f32; 3], mix: f32, cover: f32) -> CrownKey {
    CrownKey { t, a, b, mix, cover }
}
const fn lk(t: f32, col: [f32; 3], cover: f32) -> LitterKey {
    LitterKey { t, col, cover }
}

/// Per-species yearly phenology in `SPECIES_*` order.
const SPECIES: [Phenology; SPECIES_COUNT] = [
    // 0: Oak (Quercus) — tender spring buds, deep summer green, late amber/russet/crimson autumn, late drop + light marcescence.
    Phenology {
        mid_lum: 0.34,
        crown: &[
            ck(0.00, [0.42, 0.62, 0.18], [0.34, 0.54, 0.16], 0.56, 0.62),
            ck(0.09, [0.34, 0.56, 0.18], [0.28, 0.50, 0.16], 0.22, 1.00),
            ck(0.22, [0.26, 0.44, 0.18], [0.24, 0.40, 0.17], 0.00, 1.00),
            ck(0.52, [0.26, 0.44, 0.18], [0.24, 0.40, 0.17], 0.00, 1.00),
            ck(0.60, [0.74, 0.46, 0.10], [0.56, 0.22, 0.08], 0.84, 1.00),
            ck(0.68, [0.64, 0.28, 0.09], [0.48, 0.16, 0.08], 0.95, 0.82),
            ck(0.77, [0.44, 0.26, 0.13], [0.36, 0.21, 0.11], 0.95, 0.07),
            ck(0.86, [0.42, 0.25, 0.13], [0.36, 0.21, 0.11], 0.95, 0.00),
            ck(0.95, [0.44, 0.64, 0.18], [0.36, 0.56, 0.16], 0.60, 0.00),
        ],
        litter: &[
            lk(0.00, [0.32, 0.18, 0.09], 0.00),
            lk(0.63, [0.48, 0.25, 0.08], 0.00),
            lk(0.75, [0.44, 0.20, 0.08], 0.95),
            lk(0.86, [0.32, 0.18, 0.09], 0.72),
            lk(0.95, [0.32, 0.18, 0.09], 0.00),
        ],
    },
    // 1: Birch (Betula) — early bright chartreuse buds, airy summer green, luminous golden-yellow autumn, early bare winter.
    Phenology {
        mid_lum: 0.38,
        crown: &[
            ck(0.00, [0.50, 0.72, 0.22], [0.42, 0.64, 0.20], 0.50, 0.72),
            ck(0.06, [0.40, 0.64, 0.22], [0.34, 0.58, 0.20], 0.20, 1.00),
            ck(0.18, [0.32, 0.52, 0.22], [0.28, 0.48, 0.20], 0.00, 1.00),
            ck(0.49, [0.32, 0.52, 0.22], [0.28, 0.48, 0.20], 0.00, 1.00),
            ck(0.56, [0.86, 0.72, 0.14], [0.76, 0.56, 0.10], 0.80, 1.00),
            ck(0.63, [0.90, 0.74, 0.13], [0.80, 0.52, 0.09], 0.96, 0.76),
            ck(0.72, [0.68, 0.48, 0.14], [0.56, 0.36, 0.12], 0.95, 0.00),
            ck(0.94, [0.50, 0.72, 0.22], [0.42, 0.64, 0.20], 0.52, 0.00),
        ],
        litter: &[
            lk(0.00, [0.50, 0.38, 0.10], 0.00),
            lk(0.58, [0.62, 0.48, 0.10], 0.00),
            lk(0.70, [0.58, 0.44, 0.10], 0.95),
            lk(0.84, [0.42, 0.32, 0.12], 0.62),
            lk(0.94, [0.42, 0.32, 0.12], 0.00),
        ],
    },
    // 2: Pine (Pinus) — evergreen conifer; never sheds, slightly fresher in spring, cooler/darker in winter.
    Phenology {
        mid_lum: 0.30,
        crown: &[
            ck(0.08, [0.24, 0.44, 0.22], [0.20, 0.38, 0.20], 0.18, 1.00),
            ck(0.25, [0.20, 0.36, 0.22], [0.18, 0.33, 0.20], 0.00, 1.00),
            ck(0.68, [0.20, 0.36, 0.22], [0.18, 0.33, 0.20], 0.00, 1.00),
            ck(0.84, [0.14, 0.27, 0.21], [0.12, 0.23, 0.18], 0.26, 1.00),
        ],
        litter: NONE_LITTER,
    },
    // 3: Cherry (Prunus serrulata) — spring pink sakura blossoms (native texture), glossy deep green summer,
    // fiery orange-crimson autumn, bare lenticel branches in winter; drops pink petals in late spring & red leaves in autumn.
    Phenology {
        mid_lum: 0.66,
        crown: &[
            ck(0.00, [0.88, 0.64, 0.74], [0.84, 0.56, 0.68], 0.00, 0.92),
            ck(0.03, [0.88, 0.64, 0.74], [0.84, 0.56, 0.68], 0.00, 1.00),
            ck(0.13, [0.88, 0.64, 0.74], [0.84, 0.56, 0.68], 0.00, 1.00),
            ck(0.18, [0.46, 0.35, 0.18], [0.30, 0.44, 0.17], 0.85, 1.00),
            ck(0.24, [0.18, 0.40, 0.15], [0.24, 0.47, 0.18], 1.00, 1.00),
            ck(0.50, [0.18, 0.40, 0.15], [0.24, 0.47, 0.18], 1.00, 1.00),
            ck(0.58, [0.82, 0.44, 0.10], [0.74, 0.24, 0.09], 1.00, 1.00),
            ck(0.65, [0.76, 0.26, 0.09], [0.60, 0.12, 0.10], 1.00, 0.76),
            ck(0.74, [0.56, 0.18, 0.10], [0.46, 0.14, 0.09], 1.00, 0.00),
            ck(0.94, [0.88, 0.64, 0.74], [0.84, 0.56, 0.68], 0.00, 0.00),
        ],
        litter: &[
            lk(0.06, [-1.0, 0.0, 0.0], 0.00),
            lk(0.14, [-1.0, 0.0, 0.0], 0.90),
            lk(0.22, [-1.0, 0.0, 0.0], 0.00),
            lk(0.60, [0.56, 0.18, 0.08], 0.00),
            lk(0.72, [0.52, 0.15, 0.08], 0.92),
            lk(0.85, [0.36, 0.14, 0.08], 0.60),
            lk(0.94, [0.36, 0.14, 0.08], 0.00),
        ],
    },
    // 4: Palm (Cocos) — tropical evergreen; steady glossy fronds all year.
    Phenology {
        mid_lum: 0.36,
        crown: &[
            ck(0.10, [0.36, 0.58, 0.20], [0.32, 0.52, 0.18], 0.10, 1.00),
            ck(0.32, [0.30, 0.48, 0.20], [0.28, 0.44, 0.18], 0.00, 1.00),
            ck(0.62, [0.30, 0.48, 0.20], [0.28, 0.44, 0.18], 0.00, 1.00),
            ck(0.82, [0.26, 0.43, 0.20], [0.24, 0.39, 0.18], 0.10, 1.00),
        ],
        litter: NONE_LITTER,
    },
    // 5: Baobab (Adansonia) — savanna dry-season deciduous; lush wet-season green, turns dry tan and drops bare.
    Phenology {
        mid_lum: 0.36,
        crown: &[
            ck(0.00, [0.38, 0.58, 0.20], [0.34, 0.52, 0.18], 0.34, 0.82),
            ck(0.06, [0.30, 0.48, 0.18], [0.28, 0.44, 0.18], 0.00, 1.00),
            ck(0.40, [0.30, 0.48, 0.18], [0.28, 0.44, 0.18], 0.00, 1.00),
            ck(0.48, [0.70, 0.54, 0.18], [0.58, 0.40, 0.15], 0.84, 0.84),
            ck(0.58, [0.56, 0.42, 0.18], [0.46, 0.34, 0.16], 0.92, 0.00),
            ck(0.92, [0.40, 0.60, 0.22], [0.34, 0.54, 0.20], 0.40, 0.00),
        ],
        litter: &[
            lk(0.00, [0.40, 0.30, 0.14], 0.00),
            lk(0.45, [0.48, 0.36, 0.14], 0.00),
            lk(0.58, [0.42, 0.32, 0.14], 0.86),
            lk(0.78, [0.35, 0.27, 0.14], 0.40),
            lk(0.90, [0.35, 0.27, 0.14], 0.00),
        ],
    },
    // 6: Acacia (Vachellia) — savanna semi-deciduous; full olive-green in wet season, thins to ~42% dusty khaki in dry season.
    Phenology {
        mid_lum: 0.35,
        crown: &[
            ck(0.00, [0.38, 0.56, 0.20], [0.34, 0.50, 0.18], 0.26, 0.88),
            ck(0.07, [0.30, 0.46, 0.18], [0.28, 0.42, 0.18], 0.00, 1.00),
            ck(0.42, [0.30, 0.46, 0.18], [0.28, 0.42, 0.18], 0.00, 1.00),
            ck(0.52, [0.56, 0.48, 0.20], [0.48, 0.40, 0.18], 0.55, 0.72),
            ck(0.64, [0.50, 0.44, 0.20], [0.44, 0.36, 0.18], 0.62, 0.42),
            ck(0.88, [0.50, 0.44, 0.20], [0.44, 0.36, 0.18], 0.62, 0.42),
            ck(0.94, [0.38, 0.56, 0.20], [0.34, 0.50, 0.18], 0.35, 0.58),
        ],
        litter: &[
            lk(0.00, [0.48, 0.40, 0.19], 0.00),
            lk(0.48, [0.52, 0.44, 0.20], 0.00),
            lk(0.62, [0.48, 0.40, 0.19], 0.45),
            lk(0.80, [0.44, 0.36, 0.18], 0.22),
            lk(0.92, [0.44, 0.36, 0.18], 0.00),
        ],
    },
    // 7: Redwood (Sequoia sempervirens) — coastal evergreen conifer; deep green year-round, subtly darker in winter.
    Phenology {
        mid_lum: 0.28,
        crown: &[
            ck(0.10, [0.20, 0.40, 0.20], [0.18, 0.36, 0.18], 0.14, 1.00),
            ck(0.25, [0.18, 0.34, 0.20], [0.16, 0.31, 0.18], 0.00, 1.00),
            ck(0.70, [0.18, 0.34, 0.20], [0.16, 0.31, 0.18], 0.00, 1.00),
            ck(0.85, [0.13, 0.26, 0.19], [0.11, 0.22, 0.17], 0.22, 1.00),
        ],
        litter: NONE_LITTER,
    },
    // 8: Eucalyptus — evergreen broadleaf; silvery blue-green year-round, slightly warmer sage in late summer.
    Phenology {
        mid_lum: 0.38,
        crown: &[
            ck(0.12, [0.30, 0.48, 0.42], [0.28, 0.44, 0.38], 0.10, 1.00),
            ck(0.45, [0.38, 0.48, 0.36], [0.34, 0.44, 0.34], 0.14, 1.00),
            ck(0.84, [0.26, 0.40, 0.38], [0.24, 0.36, 0.34], 0.16, 1.00),
        ],
        litter: NONE_LITTER,
    },
    // 9: Bamboo — evergreen woody grass; fresh emerald spring flush, muted winter olive tinge.
    Phenology {
        mid_lum: 0.38,
        crown: &[
            ck(0.08, [0.38, 0.62, 0.22], [0.34, 0.56, 0.20], 0.22, 1.00),
            ck(0.25, [0.32, 0.52, 0.22], [0.30, 0.48, 0.20], 0.00, 1.00),
            ck(0.62, [0.32, 0.52, 0.22], [0.30, 0.48, 0.20], 0.00, 1.00),
            ck(0.85, [0.32, 0.44, 0.22], [0.38, 0.44, 0.20], 0.22, 1.00),
        ],
        litter: NONE_LITTER,
    },
];

/// Evaluated seasonal state for one tree species at a given year fraction.
#[derive(Clone, Copy, Debug)]
pub struct SpeciesLook {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub mix: f32,
    pub cover: f32,
    pub litter_col: [f32; 3],
    pub litter_cover: f32,
}

/// Evaluates species `species` (`0..SPECIES_COUNT`) at `year` in `[0, 1)`.
pub fn species_look(species: usize, year: f32) -> SpeciesLook {
    let p = &SPECIES[species.min(SPECIES_COUNT - 1)];
    let y = year.rem_euclid(1.0);
    let (c0, c1, u) = bracket(p.crown, y, |k| k.t);
    let a = lerp3(c0.a, c1.a, u);
    let b = lerp3(c0.b, c1.b, u);
    let mix = lerp(c0.mix, c1.mix, u);
    let cover = lerp(c0.cover, c1.cover, u);
    let (litter_col, litter_cover) = if p.litter.is_empty() {
        ([0.0; 3], 0.0)
    } else {
        let (l0, l1, lu) = bracket(p.litter, y, |k| k.t);
        let col = if l0.col[0] < 0.0 || l1.col[0] < 0.0 { [-1.0, 0.0, 0.0] } else { lerp3(l0.col, l1.col, lu) };
        (col, lerp(l0.cover, l1.cover, lu))
    };
    SpeciesLook { a, b, mix, cover, litter_col, litter_cover }
}

/// Packs all species' seasonal uniforms at `year` into [`FOLIAGE_FLOATS`] floats:
/// `leafA[10]` (40), `leafB[10]` (40), `litter[10]` (40). Target colours are pre-divided by `mid_lum`.
pub fn pack_foliage(year: f32) -> [f32; FOLIAGE_FLOATS] {
    let mut out = [0.0f32; FOLIAGE_FLOATS];
    for (s, p) in SPECIES.iter().enumerate() {
        let look = species_look(s, year);
        let inv = 1.0 / p.mid_lum;
        let ia = s * 4;
        out[ia..ia + 4].copy_from_slice(&[look.a[0] * inv, look.a[1] * inv, look.a[2] * inv, look.mix]);
        let ib = SPECIES_COUNT * 4 + s * 4;
        out[ib..ib + 4].copy_from_slice(&[look.b[0] * inv, look.b[1] * inv, look.b[2] * inv, look.cover]);
        let il = SPECIES_COUNT * 8 + s * 4;
        let lit = if look.litter_col[0] < 0.0 {
            [-1.0, 0.0, 0.0, look.litter_cover]
        } else {
            [look.litter_col[0] * inv, look.litter_col[1] * inv, look.litter_col[2] * inv, look.litter_cover]
        };
        out[il..il + 4].copy_from_slice(&lit);
    }
    out
}

/// Evaluates seasonal grass tint, woodland-fern/reed tint, spring budding flag, wildflower cover, and 3D grass height
/// & density at `year`. Layout (12 floats = 3 `vec4`s):
/// `[grass.rgb, 1, woodland.rgb, 1, budding, flower_cover, grass_height, grass_density]`.
pub fn pack_ground(year: f32) -> [f32; GROUND_FLOATS] {
    #[derive(Clone, Copy)]
    struct Gk {
        t: f32,
        grass: [f32; 3],
        wood: [f32; 3],
        bud: f32,
        flowers: f32,
        gh: f32,
        gd: f32,
    }
    const KEYS: [Gk; 7] = [
        Gk {
            t: 0.00,
            grass: [0.95, 1.07, 0.88],
            wood: [0.95, 1.07, 0.88],
            bud: 1.0,
            flowers: 0.65,
            gh: 0.72,
            gd: 0.76,
        },
        Gk {
            t: 0.12,
            grass: [0.94, 1.11, 0.85],
            wood: [0.94, 1.10, 0.86],
            bud: 0.25,
            flowers: 1.00,
            gh: 1.12,
            gd: 1.00,
        },
        Gk {
            t: 0.30,
            grass: [1.00, 1.00, 1.00],
            wood: [1.00, 1.00, 1.00],
            bud: 0.0,
            flowers: 1.00,
            gh: 1.06,
            gd: 1.00,
        },
        Gk {
            t: 0.46,
            grass: [1.10, 1.01, 0.78],
            wood: [1.08, 0.98, 0.74],
            bud: 0.0,
            flowers: 0.92,
            gh: 0.96,
            gd: 0.92,
        },
        Gk {
            t: 0.64,
            grass: [1.14, 0.94, 0.66],
            wood: [1.18, 0.88, 0.54],
            bud: 0.0,
            flowers: 0.22,
            gh: 0.78,
            gd: 0.72,
        },
        Gk {
            t: 0.82,
            grass: [0.95, 0.87, 0.73],
            wood: [0.86, 0.74, 0.58],
            bud: 0.0,
            flowers: 0.00,
            gh: 0.42,
            gd: 0.38,
        },
        Gk {
            t: 0.94,
            grass: [0.94, 0.90, 0.77],
            wood: [0.88, 0.80, 0.64],
            bud: 1.0,
            flowers: 0.00,
            gh: 0.46,
            gd: 0.44,
        },
    ];
    let (k0, k1, u) = bracket(&KEYS, year.rem_euclid(1.0), |k| k.t);
    let (g, w) = (lerp3(k0.grass, k1.grass, u), lerp3(k0.wood, k1.wood, u));
    [
        g[0],
        g[1],
        g[2],
        1.0,
        w[0],
        w[1],
        w[2],
        1.0,
        lerp(k0.bud, k1.bud, u),
        lerp(k0.flowers, k1.flowers, u),
        lerp(k0.gh, k1.gh, u),
        lerp(k0.gd, k1.gd, u),
    ]
}

/// Packed 2-bit seasonal role per texture layer (128 `u32` words = 2048 layers, same layout as `block::turn::MASK`).
/// Ore-indicator layers keep `ROLE_NONE` so they remain recognisable year-round.
pub fn season_layers() -> Vec<u32> {
    let mut mask = vec![0u32; 128];
    let mut set = |l: u16, r: u32| mask[usize::from(l) >> 4] |= (r & 3) << ((usize::from(l) & 15) * 2);
    for l in [tex::GRASS_TOP, tex::GRASS_SIDE, tex::GRASS_TUFT] {
        set(l, ROLE_GRASS);
    }
    for l in tex::RUSTY_GRASS_TOP..=tex::PALE_GRASS_SIDE {
        set(l, ROLE_GRASS);
    }
    for l in [tex::FERN, tex::REED] {
        set(l, ROLE_WOODLAND);
    }
    for l in [tex::DAISY, tex::POPPY, tex::BLUEBELL] {
        set(l, ROLE_FLOWER);
    }
    mask
}

/// Finds the cyclic keyframe pair bracketing `y` in `[0, 1)` and returns `(prev, next, smoothstep_t)`.
fn bracket<T: Copy>(keys: &[T], y: f32, time: impl Fn(&T) -> f32) -> (T, T, f32) {
    let n = keys.len();
    if n <= 1 {
        return (keys[0], keys[0], 0.0);
    }
    let i = keys.iter().rposition(|k| time(k) <= y).unwrap_or(n - 1);
    let j = (i + 1) % n;
    let (t0, mut t1, mut yy) = (time(&keys[i]), time(&keys[j]), y);
    if t1 <= t0 {
        t1 += 1.0;
        if yy < t0 {
            yy += 1.0;
        }
    }
    let t = ((yy - t0) / (t1 - t0).max(1e-5)).clamp(0.0, 1.0);
    (keys[i], keys[j], t * t * (3.0 - 2.0 * t))
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)]
}
