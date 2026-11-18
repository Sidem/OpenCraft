//! The LED lamp (ids 220–222): a lamp that lights wider than the plain one (`light::SOURCES` rows `LED_LIGHTS`: loss 2 a
//! block, so the field never seams at chunk borders) and takes a colour from its panel (`led/`). The three ids are its
//! three brightness steps, because block light is derived from blocks only: the panel swaps one for another in place
//! (`Action::SetLed`). The item and the id it places, 220, is the BRIGHTEST step (never weaker than a plain lamp at any
//! distance: a test checks it); the two dimmer steps are only reached from the panel. Every step drops the item.
//! Unpowered, like the lamp. To add a block after this one: a set file of its own like this, joined in `defs.rs` and
//! counted in `BLOCK_COUNT`.

use super::make::{all, cube};
use super::{sound, tex, BlockDef, BlockId};

pub const LED: BlockId = 220;
pub const LED_MEDIUM: BlockId = 221;
pub const LED_DIM: BlockId = 222;

/// The brightness steps, dimmest first (the panel's index is a position here); the last is the item, `LED`.
pub const LED_STEPS: [BlockId; 3] = [LED_DIM, LED_MEDIUM, LED];

pub const LED_COUNT: usize = 3;

/// Whether `b` is an LED lamp at any step.
pub const fn is_led(b: BlockId) -> bool {
    b >= LED && b <= LED_DIM
}

const fn led(name: &'static str, light: u8, placeable: bool) -> BlockDef {
    BlockDef { light, placeable, ..cube(name, 0.4, all(tex::LED_PANEL), LED, sound::METAL) }
}

/// The rows of `DEFS` from block 220 on, in id order (`LED_LIGHTS` is dimmest first).
pub(super) const LED_DEFS: [BlockDef; LED_COUNT] = [
    led("LED Lamp", crate::light::LED_LIGHTS[2], true),
    led("LED Lamp (medium)", crate::light::LED_LIGHTS[1], false),
    led("LED Lamp (dim)", crate::light::LED_LIGHTS[0], false),
];
