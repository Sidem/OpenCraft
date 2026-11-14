//! Generator 12's species as data: one `Kind` row each (how tall, how wide the trunk, where the crown starts, how its
//! limbs go and how its leaves cluster), and where each grows (`pick`). The shapes these numbers drive are in
//! `shapes.rs`. Proportions follow the real trees at about a block a metre for the small ones and less for the giants
//! (a redwood is 30-odd blocks, not 90): an oak's crown about as wide as it is tall, a birch's narrow, a pine's a
//! cone, a cherry's a wide vase, an acacia's a flat umbrella, a baobab's few short limbs on a bottle, a redwood's
//! short drooping limbs up a tall bare column, a eucalyptus' open and drooping, a palm's fronds on a bending stem.
//! To add a species: its blocks (`block/trees.rs`), a row here, an arm in `pick` and `kind_of_sapling`.

use std::f32::consts::PI;

use crate::block::*;
use crate::worldgen::Biome;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Crown {
    /// Limbs out of the trunk's upper part, leaf clusters at their ends and forks (broadleaves).
    Round,
    /// Whorls of short limbs all up the trunk, longer below, needles all along them (conifers).
    Cone,
    /// Limbs that all leave one height and spread to a flat top (acacia).
    Umbrella,
    /// Fronds round the top of a bare, bending stem.
    Palm,
    /// A clump of thin culms with leaves along their tops (bamboo).
    Clump,
}

#[derive(Clone, Copy, Debug)]
pub struct Kind {
    pub log: BlockId,
    pub leaves: BlockId,
    pub sapling: BlockId,
    /// The surface it grows on.
    pub ground: BlockId,
    pub crown: Crown,
    /// Height of the trunk: the least and how many more it may be.
    pub height: (i32, u32),
    /// Width of the trunk at its foot (1, 2 or 3 logs square), and the share of the height where it starts to narrow.
    pub width: i32,
    pub narrow: f32,
    /// How far the trunk bends over, as a share of its height at its top (a palm's lean).
    pub lean: f32,
    /// The share of the height that is bare trunk before the first limb.
    pub bole: f32,
    /// Limbs: the least and how many more; their angle from upright (radians); their length as a share of the height.
    pub limbs: (u32, u32),
    pub angle: f32,
    pub reach: f32,
    /// How much a limb bends down along its length (negative: up).
    pub droop: f32,
    /// Whether limbs fork once (a smaller limb off their outer half).
    pub forks: bool,
    /// Leaf clusters: across and up (radii in blocks), and the share of their cells that hold leaves.
    pub cluster: (f32, f32),
    pub fill: f32,
    /// How often a candidate spot holds one, as a share of the forest's density.
    pub density: f64,
}

/// A broadleaf's defaults; rows below change what differs.
const BROADLEAF: Kind = Kind {
    log: OAK_TREE_LOG,
    leaves: LEAVES,
    sapling: SAPLING,
    ground: GRASS,
    crown: Crown::Round,
    height: (8, 4),
    width: 1,
    narrow: 1.0,
    lean: 0.0,
    bole: 0.4,
    limbs: (5, 3),
    angle: 0.95,
    reach: 0.45,
    droop: 0.15,
    forks: true,
    cluster: (2.2, 1.5),
    fill: 0.78,
    density: 1.0,
};

pub const OAK: usize = 0;
pub const BROAD: usize = 1;
pub const BIRCH: usize = 2;
pub const PINE: usize = 3;
pub const CHERRY: usize = 4;
pub const PALM: usize = 5;
pub const BAOBAB: usize = 6;
pub const ACACIA: usize = 7;
pub const REDWOOD: usize = 8;
pub const EUCALYPTUS: usize = 9;
pub const BAMBOO_CLUMP: usize = 10;

pub const KINDS: [Kind; 11] = [
    BROADLEAF,
    // A lowland oak: shorter, much wider.
    Kind { height: (7, 3), bole: 0.38, limbs: (6, 3), angle: 1.15, reach: 0.62, cluster: (2.4, 1.4), ..BROADLEAF },
    Kind {
        log: BIRCH_LOG,
        leaves: BIRCH_LEAVES,
        sapling: BIRCH_SAPLING,
        height: (10, 4),
        bole: 0.45,
        limbs: (6, 3),
        angle: 0.7,
        reach: 0.32,
        droop: 0.05,
        forks: false,
        cluster: (1.8, 1.8),
        fill: 0.78,
        ..BROADLEAF
    },
    Kind {
        log: PINE_LOG,
        leaves: PINE_LEAVES,
        sapling: PINE_SAPLING,
        crown: Crown::Cone,
        height: (12, 6),
        bole: 0.3,
        limbs: (0, 0),
        angle: 1.45,
        reach: 0.3,
        droop: 0.3,
        forks: false,
        cluster: (1.5, 1.2),
        fill: 0.9,
        ..BROADLEAF
    },
    Kind {
        log: CHERRY_LOG,
        leaves: CHERRY_BLOSSOM,
        sapling: CHERRY_SAPLING,
        height: (6, 3),
        bole: 0.35,
        limbs: (4, 3),
        angle: 0.9,
        reach: 0.7,
        droop: -0.1,
        cluster: (2.0, 1.3),
        fill: 0.8,
        ..BROADLEAF
    },
    Kind {
        log: PALM_LOG,
        leaves: PALM_FRONDS,
        sapling: PALM_SAPLING,
        ground: SAND,
        crown: Crown::Palm,
        height: (7, 5),
        lean: 0.35,
        bole: 1.0,
        limbs: (7, 3),
        reach: 0.0,
        forks: false,
        cluster: (0.0, 0.0),
        fill: 1.0,
        density: 0.6,
        ..BROADLEAF
    },
    Kind {
        log: BAOBAB_LOG,
        leaves: BAOBAB_LEAVES,
        sapling: BAOBAB_SAPLING,
        height: (8, 3),
        width: 3,
        narrow: 0.75,
        bole: 0.7,
        limbs: (6, 3),
        angle: 1.3,
        reach: 0.9,
        droop: -0.15,
        forks: true,
        cluster: (2.1, 1.2),
        fill: 0.65,
        density: 0.25,
        ..BROADLEAF
    },
    Kind {
        log: ACACIA_LOG,
        leaves: ACACIA_LEAVES,
        sapling: ACACIA_SAPLING,
        crown: Crown::Umbrella,
        height: (6, 3),
        lean: 0.12,
        bole: 0.45,
        limbs: (3, 2),
        angle: 0.8,
        reach: 0.75,
        droop: -0.05,
        forks: true,
        cluster: (3.2, 1.2),
        fill: 0.72,
        density: 0.45,
        ..BROADLEAF
    },
    Kind {
        log: REDWOOD_LOG,
        leaves: REDWOOD_NEEDLES,
        sapling: REDWOOD_SAPLING,
        crown: Crown::Cone,
        height: (26, 12),
        width: 2,
        narrow: 0.82,
        bole: 0.45,
        limbs: (0, 0),
        angle: 1.5,
        reach: 0.12,
        droop: 0.45,
        forks: false,
        cluster: (1.3, 1.1),
        fill: 0.9,
        density: 0.8,
        ..BROADLEAF
    },
    Kind {
        log: EUCALYPTUS_LOG,
        leaves: EUCALYPTUS_LEAVES,
        sapling: EUCALYPTUS_SAPLING,
        height: (14, 6),
        lean: 0.06,
        bole: 0.5,
        limbs: (5, 3),
        angle: 0.85,
        reach: 0.42,
        droop: 0.1,
        forks: true,
        cluster: (2.0, 1.5),
        fill: 0.6,
        ..BROADLEAF
    },
    Kind {
        log: BAMBOO,
        leaves: BAMBOO_LEAVES,
        sapling: BAMBOO_SHOOT,
        crown: Crown::Clump,
        height: (8, 6),
        bole: 0.5,
        limbs: (4, 4),
        reach: 0.0,
        forks: false,
        cluster: (0.0, 0.0),
        fill: 0.8,
        density: 0.7,
        ..BROADLEAF
    },
];

/// The angle between limbs round the trunk: the golden angle, so they never line up.
pub const GOLDEN: f32 = PI * (3.0 - 2.236_068);

/// The kind a planted sapling grows into.
pub fn kind_of_sapling(sapling: BlockId) -> usize {
    KINDS.iter().position(|k| k.sapling == sapling).unwrap_or(OAK)
}

/// What grows on a candidate spot: its forest biome (`WorldGen::local_biome_at`), surface block `top`, warmth `t`
/// (about -0.6..0.6, calm near spawn), whether it is a shore (low ground by water) and whether it lies in a grove
/// (cherry and bamboo grow in stands), from the spot's hash `h`. `None`: nothing new (deserts and basalt fields keep
/// their old trees).
pub fn pick(biome: Biome, top: BlockId, t: f64, shore: bool, grove: bool, h: u32) -> Option<usize> {
    let roll = (h >> 24) % 20;
    let of = |table: &[(usize, u32)]| {
        let mut left = roll;
        for &(k, w) in table {
            if left < w {
                return Some(k);
            }
            left -= w;
        }
        None
    };
    if shore && t > -0.08 && (top == SAND || roll < 12) {
        return Some(PALM);
    }
    if top != GRASS {
        return None;
    }
    match biome {
        Biome::Desert | Biome::BasaltFields => None,
        Biome::Redwoods => of(&[(REDWOOD, 14), (PINE, 5), (BIRCH, 1)]),
        Biome::Highlands => of(&[(PINE, 17), (BIRCH, 3)]),
        Biome::Boreal => of(&[(PINE, 12), (BIRCH, 8)]),
        Biome::BambooForest if grove => of(&[(BAMBOO_CLUMP, 16), (EUCALYPTUS, 4)]),
        Biome::BambooForest => of(&[(BAMBOO_CLUMP, 7), (EUCALYPTUS, 7), (BROAD, 6)]),
        Biome::Lowlands => of(&[(BROAD, 12), (OAK, 4), (BIRCH, 4)]),
        Biome::Savanna => of(&[(ACACIA, 14), (BAOBAB, 6)]),
        Biome::Woodland => of(&[(EUCALYPTUS, 13), (OAK, 4), (BIRCH, 3)]),
        Biome::Plains if grove => of(&[(CHERRY, 14), (OAK, 4), (BIRCH, 2)]),
        Biome::Plains => of(&[(OAK, 13), (BIRCH, 5), (CHERRY, 1), (PINE, 1)]),
    }
}
