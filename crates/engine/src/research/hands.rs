//! The hand-crafting speed techs (less hand-crafting interlude), appended after `ai.rs`'s in `techs::TECHS`, so the
//! saved index order is: Handcrafting I 74, Handcrafting II 75, Handcrafting III 76; add rows at the bottom, never
//! reorder. Each lists one tier of `Stat::Crafting`; `perks::player_bonus` adds up the finished ones.

use crate::item::{BLUE_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::perks::Stat;

use super::{Tech, Unlock};

pub const HANDS: [Tech; 3] = [
    Tech {
        name: "Handcrafting I",
        blurb: "You craft by hand 25% faster.",
        needs: &[2],
        packs: &[RED_PACK, GREEN_PACK],
        units: 40,
        seconds: 10.0,
        unlocks: &[Unlock::Perk(Stat::Crafting, 1)],
    },
    Tech {
        name: "Handcrafting II",
        blurb: "You craft by hand another 25% faster (50% in all).",
        needs: &[74, 10],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 60,
        seconds: 20.0,
        unlocks: &[Unlock::Perk(Stat::Crafting, 2)],
    },
    Tech {
        name: "Handcrafting III",
        blurb: "You craft by hand another 25% faster (75% in all).",
        needs: &[75, 18],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 100,
        seconds: 30.0,
        unlocks: &[Unlock::Perk(Stat::Crafting, 3)],
    },
];
