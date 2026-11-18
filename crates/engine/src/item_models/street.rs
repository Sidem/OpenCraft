//! The street lamp's item model (`block::STREET`), so its icon and the lamp in the hand read as a street lamp and not a
//! cube: the placed lamp (`street/model.rs`) at a quarter of its height, its arm leaning toward +x (side on to the icon's
//! viewer), the post and the lantern a little thicker than to scale so they read at hotbar size. Parts are listed back to
//! front for the icon (it has no depth test).
//!
//! Invariants: everything stays inside the item cube and stands on its floor; the gooseneck is leaned bars, the round
//! parts are round bodies. To change it, follow the placed model: heights there divided by four.

use std::f32::consts::FRAC_PI_2;

use super::shapes::{cylinder, frustum};
use super::{part, Form, Part};
use crate::block::{tex, STREET};
use crate::item::ItemId;

const ITEM_STREET: ItemId = ItemId::block(STREET);

/// The footing, the banded base, the tapering post with a band, a gooseneck of four leaned bars bending from the post's
/// top (x -0.03, y 0.4) over to the lantern (x 0.09), and the lantern: a bottom plate, the warm glass, the hood's rim
/// and the hood.
const LAMP_MODEL: [Part; 14] = [
    part([-0.03, -0.4825, 0.0], [0.15, 0.035, 0.15], tex::CONCRETE),
    cylinder([-0.03, -0.459, 0.0], 0.1, 0.012, tex::MAT_METAL),
    frustum([-0.03, -0.395, 0.0], 0.086, 0.13, 0.8, tex::MAT_GUNMETAL),
    cylinder([-0.03, -0.328, 0.0], 0.08, 0.012, tex::MAT_METAL),
    frustum([-0.03, 0.0375, 0.0], 0.054, 0.725, 0.6, tex::MAT_GUNMETAL),
    cylinder([-0.03, 0.275, 0.0], 0.044, 0.014, tex::MAT_METAL),
    arm([-0.0212, 0.4212], 0.3935),
    arm([0.0088, 0.4512], 1.1773),
    arm([0.0512, 0.4512], 1.9643),
    arm([0.0812, 0.4212], 2.7481),
    cylinder([0.09, 0.316, 0.0], 0.09, 0.012, tex::MAT_GUNMETAL),
    cylinder([0.09, 0.355, 0.0], 0.115, 0.07, tex::LAMP),
    cylinder([0.09, 0.392, 0.0], 0.17, 0.01, tex::MAT_METAL),
    frustum([0.09, 0.407, 0.0], 0.165, 0.024, 0.3, tex::MAT_GUNMETAL),
];

/// A gooseneck bar centred at (x, y), leaned by `pitch` toward +x.
const fn arm([x, y]: [f32; 2], pitch: f32) -> Part {
    Part {
        center: [x, y, 0.0],
        size: [0.026, 0.072, 0.026],
        yaw: -FRAC_PI_2,
        tex: [tex::MAT_GUNMETAL; 3],
        form: Form::Lean(pitch),
    }
}

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    match item {
        ITEM_STREET => &LAMP_MODEL,
        _ => &[],
    }
}
