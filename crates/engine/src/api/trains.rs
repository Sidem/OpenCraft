//! Trains for the host's train panel (`web/src/ui/train.ts`, opened through `take_panel_request` with a car's cargo
//! address, `factory::car_box`): the panel's numbers, what the train is doing, the schedule edits. A wagon's slots are
//! read and clicked as a box at its cargo address (`box_slots`, `click_box`, ...).

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::factory::{car_at, car_box, DROP_ALL};
use crate::math::IVec3;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// For a cargo address: wagons, the next stop's index, the number of stops, each stop's dock (x, y, z), then each
    /// wagon's used slots and items (`Factory::train_panel`). Empty for any other cell or a train that is gone.
    pub fn train_panel(&self, x: i32, y: i32, z: i32) -> Vec<i32> {
        car_at(IVec3::new(x, y, z)).map_or_else(Vec::new, |aim| self.sim.factory.train_panel(aim))
    }

    /// What the train of a cargo address is doing, in a few words.
    pub fn train_state(&self, x: i32, y: i32, z: i32) -> String {
        let f = &self.sim.factory;
        let train = car_at(IVec3::new(x, y, z)).and_then(|aim| f.train_by_id(aim.train));
        train.map_or_else(String::new, |t| f.train_state(t))
    }

    /// The cargo address of car `car` of the train of a cargo address.
    pub fn train_car(&self, x: i32, y: i32, z: i32, car: u8) -> Vec<i32> {
        car_at(IVec3::new(x, y, z)).map_or_else(Vec::new, |aim| {
            let p = car_box(aim.train, car);
            vec![p.x, p.y, p.z]
        })
    }

    /// Takes stop `index` off the schedule of the train of a cargo address (next tick).
    pub fn drop_stop(&mut self, x: i32, y: i32, z: i32, index: u8) {
        if let Some(aim) = car_at(IVec3::new(x, y, z)) {
            self.act(Action::DropStop { train: aim.train, index });
        }
    }

    /// Empties that train's schedule, so it stops at every dock it reaches (next tick).
    pub fn clear_stops(&mut self, x: i32, y: i32, z: i32) {
        self.drop_stop(x, y, z, DROP_ALL);
    }
}
