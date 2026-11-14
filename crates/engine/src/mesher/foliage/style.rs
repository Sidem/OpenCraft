//! How each kind of leaf block is drawn as cards (`foliage.rs`): how many, how big and how they lie. Data: a row per
//! leaf block. To give a new leaf block its look: a row in `style_of` (blocks without one draw as broad leaves).

use crate::block::{
    BlockId, ACACIA_LEAVES, BAMBOO_LEAVES, BAOBAB_LEAVES, BIRCH_LEAVES, CHERRY_BLOSSOM, EUCALYPTUS_LEAVES, PALM_FRONDS,
    PINE_LEAVES, REDWOOD_NEEDLES,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lie {
    /// Clumps of broad leaves facing out of the crown, every way.
    Clumps,
    /// Flat sprays, their outer edge dipping (conifers, flat-topped crowns).
    Sprays,
    /// Hanging curtains, upright (weeping and drooping leaves).
    Hanging,
    /// Long arching fronds reaching out of the crown (palms).
    Fronds,
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub lie: Lie,
    /// Cards on a leaf block that shows, and on one buried in its crown.
    pub cards: u32,
    pub buried: u32,
    /// A card's width in blocks (a frond's length), and how far its middle strays from the block's.
    pub size: f32,
    pub stray: f32,
    /// How far an outer edge dips (sprays) or how far a card leans out of the crown (clumps), 0..1.
    pub tilt: f32,
}

const BROAD: Style = Style { lie: Lie::Clumps, cards: 3, buried: 1, size: 1.7, stray: 0.3, tilt: 0.8 };

/// The cards a leaf block is drawn with.
pub fn style_of(b: BlockId) -> Style {
    match b {
        BIRCH_LEAVES => Style { lie: Lie::Clumps, cards: 3, buried: 1, size: 1.45, stray: 0.35, tilt: 0.6 },
        PINE_LEAVES => Style { lie: Lie::Sprays, cards: 3, buried: 1, size: 1.6, stray: 0.3, tilt: 0.45 },
        REDWOOD_NEEDLES => Style { lie: Lie::Sprays, cards: 3, buried: 1, size: 1.7, stray: 0.3, tilt: 0.6 },
        ACACIA_LEAVES => Style { lie: Lie::Sprays, cards: 3, buried: 1, size: 1.9, stray: 0.35, tilt: 0.25 },
        CHERRY_BLOSSOM => Style { lie: Lie::Clumps, cards: 3, buried: 1, size: 1.6, stray: 0.3, tilt: 0.9 },
        BAOBAB_LEAVES => Style { lie: Lie::Clumps, cards: 2, buried: 1, size: 1.5, stray: 0.35, tilt: 0.7 },
        EUCALYPTUS_LEAVES => Style { lie: Lie::Hanging, cards: 3, buried: 1, size: 1.3, stray: 0.4, tilt: 0.0 },
        BAMBOO_LEAVES => Style { lie: Lie::Hanging, cards: 2, buried: 1, size: 1.2, stray: 0.45, tilt: 0.0 },
        PALM_FRONDS => Style { lie: Lie::Fronds, cards: 2, buried: 1, size: 2.6, stray: 0.2, tilt: 0.5 },
        _ => BROAD,
    }
}
