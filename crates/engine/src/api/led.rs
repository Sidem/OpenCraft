//! LED lamps for the host (`led/`): what a lamp is set to, for its panel (`web/src/ui/led-panel.ts`, opened through
//! `take_panel_request` like a sign's editor), and the edit.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::block::LED_STEPS;
use crate::led::{step_of, DEFAULT_COLOUR};
use crate::math::IVec3;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The LED lamp at a block as its panel shows it: brightness step (0 is the dimmest) and colour (0xRRGGBB); empty
    /// when no LED lamp stands there.
    pub fn led_state(&self, x: i32, y: i32, z: i32) -> Vec<u32> {
        let pos = IVec3::new(x, y, z);
        let step = self.sim.world.get_block(pos).and_then(step_of);
        step.map_or_else(Vec::new, |s| vec![s as u32, self.sim.leds.colour(pos)])
    }

    /// How many brightness steps an LED lamp has.
    pub fn led_steps(&self) -> u32 {
        LED_STEPS.len() as u32
    }

    /// The colour of a lamp nobody coloured (0xRRGGBB).
    pub fn led_default_colour(&self) -> u32 {
        DEFAULT_COLOUR
    }

    /// Sets the LED lamp at a block to a brightness step and a colour.
    pub fn set_led(&mut self, x: i32, y: i32, z: i32, step: u32, rgb: u32) {
        let step = u8::try_from(step).unwrap_or(u8::MAX);
        self.act(Action::SetLed { pos: IVec3::new(x, y, z), step, rgb });
    }
}
