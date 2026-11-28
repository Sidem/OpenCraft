//! Item models for FIRE and hand lights: charred wood billets (`CHARCOAL`), the rifle-style liquid-fuel torch
//! (`FLAMETHROWER`), and the slim handheld `TORCH` (wooden shaft, dark collar and glowing flame tip).
//! Built from real cylinders, frustums, bars and boxes; bottoms rest at `-size[1] / 2` inside [-0.5, 0.5].

use super::shapes::{bar_x, cylinder, frustum, Build};
use super::{part, turned, Part};
use crate::block::{self, tex};
use crate::item::{ItemId, CHARCOAL, FLAMETHROWER};

const ITEM_TORCH: ItemId = ItemId::block(block::TORCH);

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        CHARCOAL => &CHARCOAL_MODEL,
        FLAMETHROWER => &FLAMETHROWER_MODEL,
        ITEM_TORCH => &TORCH_MODEL,
        _ => &[],
    }
}

/// A slim handheld torch: wooden shaft, charred collar and warm flame tip (`icon_frame` keeps its flat sprite icon).
const TORCH_MODEL: [Part; 4] = [
    cylinder([0.0, -0.14, 0.0], 0.13, 0.54, tex::HANDLE),
    cylinder([0.0, 0.15, 0.0], 0.16, 0.07, tex::SOOT),
    frustum([0.0, 0.25, 0.0], 0.14, 0.14, 0.45, tex::LAMP_YELLOW),
    frustum([0.0, 0.34, 0.0], 0.09, 0.09, 0.2, tex::LAMP),
];

/// A small cluster of charred wood billets: matte charcoal cylinders and fractured chunks resting on -0.175.
const CHARCOAL_MODEL: [Part; 5] = [
    bar_x([-0.04, -0.095, -0.12], 0.44, 0.16, tex::CHARCOAL),
    bar_x([0.04, -0.095, 0.1], 0.4, 0.16, tex::SOOT),
    turned([-0.14, -0.02, 0.02], [0.22, 0.14, 0.14], 0.45, tex::CHARCOAL),
    turned([0.1, 0.04, -0.04], [0.26, 0.13, 0.13], -0.35, tex::CHARCOAL),
    cylinder([0.0, 0.12, 0.03], 0.14, 0.1, tex::SOOT),
];

/// Rifle-style liquid-fuel flamethrower pointing along +x, resting on -0.175: matte black skeletonized stock, lower
/// receiver, rear pistol grip and vertical foregrip; desert-tan vented upper shroud with top regulator knob; hanging
/// dark olive-green fuel bottle with brass valve; gleaming polished chrome cylindrical torch bell at the muzzle.
const FLAMETHROWER_MODEL: [Part; 15] = Build::new()
    // Skeletonized black stock & brace
    .with(part([-0.40, 0.13, 0.0], [0.18, 0.07, 0.07], tex::MAT_GUNMETAL))
    .with(part([-0.46, 0.06, 0.0], [0.05, 0.16, 0.07], tex::MAT_GUNMETAL))
    .with(part([-0.35, -0.01, 0.0], [0.22, 0.04, 0.06], tex::MAT_GUNMETAL))
    // Rear pistol grip (hand grip pivot at [-0.24, 0.04, 0.0]) & front vertical foregrip
    .with(part([-0.24, 0.04, 0.0], [0.075, 0.15, 0.07], tex::SOOT))
    .with(part([0.13, 0.04, 0.0], [0.065, 0.14, 0.065], tex::SOOT))
    // Matte black lower receiver chassis
    .with(part([-0.08, 0.14, 0.0], [0.54, 0.085, 0.095], tex::MAT_GUNMETAL))
    // Tan vented upper tactical shroud, rear sight ramp & top adjustment knob
    .with(part([-0.03, 0.225, 0.0], [0.46, 0.095, 0.10], tex::FLAMETHROWER))
    .with(part([-0.22, 0.285, 0.0], [0.09, 0.04, 0.07], tex::FLAMETHROWER))
    .with(cylinder([-0.06, 0.29, 0.0], 0.055, 0.045, tex::SOOT))
    // Hanging olive-green pressurized fuel bottle (bottom at -0.175) & brass neck valve
    .with(cylinder([-0.02, 0.08, 0.0], 0.055, 0.05, tex::MAT_BRASS))
    .with(frustum([-0.02, 0.035, 0.0], 0.145, 0.05, 0.45, tex::FLAMETHROWER_TANK))
    .with(cylinder([-0.02, -0.08, 0.0], 0.145, 0.19, tex::FLAMETHROWER_TANK))
    // Slim steel feed stem + gleaming polished chrome torch bell & flared rim
    .with(bar_x([0.24, 0.225, 0.0], 0.10, 0.036, tex::MAT_METAL))
    .with(bar_x([0.38, 0.225, 0.0], 0.20, 0.105, tex::FLAMETHROWER_CHROME))
    .with(bar_x([0.48, 0.225, 0.0], 0.025, 0.12, tex::FLAMETHROWER_CHROME))
    .done();
