//! Seismic Thumper hand recipes (F-SEIS): Mk1 (`Seismic Prospecting`, tech 93) plus Mk2 (violet kits, Mk4 Machines)
//! and Mk3 (gold kits, Mk5 Machines).

use crate::block::THUMPER;
use crate::item::{
    CIRCUIT, COPPER_WIRE, GOLD_KIT, MOTOR, STEEL_BEAM, STEEL_PLATE, THUMPER_ITEM, THUMPER_MK2, THUMPER_MK3, VIOLET_KIT,
};

use super::{b, Group, Recipe};

pub const THUMPER_RECIPE: Recipe = Recipe {
    output: b(THUMPER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 12), (STEEL_BEAM, 6), (MOTOR, 4), (CIRCUIT, 6), (COPPER_WIRE, 8)],
    blurb: "A powered 2×2×2 geophysical thumper (R turns its slice axis): sends acoustic waves down to bedrock and \
            displays a 64-block vertical cross-section of ore bands, caves, water and your own tunnels. Upgrade to \
            Mk2 (violet kits) and Mk3 (gold kits) for sharper resolution and mineral identification. Needs 40 kW.",
};

pub const THUMPER_MK2_RECIPE: Recipe = Recipe {
    output: THUMPER_MK2,
    group: Group::Production,
    count: 1,
    inputs: &[(THUMPER_ITEM, 1), (VIOLET_KIT, 4)],
    blurb: "Upgraded geophone array (65 kW): cuts seismic noise by more than half, separates player tunnels from \
            natural caves and resolves mineral families. Or upgrade a placed thumper with 4 violet kits.",
};

pub const THUMPER_MK3_RECIPE: Recipe = Recipe {
    output: THUMPER_MK3,
    group: Group::Production,
    count: 1,
    inputs: &[(THUMPER_MK2, 1), (GOLD_KIT, 4)],
    blurb: "Tomographic array (90 kW): razor-sharp cross-section with minimal noise and confident ore identification \
            down to bedrock. Or upgrade a placed thumper with 4 gold kits.",
};
