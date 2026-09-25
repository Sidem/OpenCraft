//! Machine panels (`web/src/ui/machine.ts`) and box screens (`ui/inventory.ts`): which panel to open,
//! what it shows (`factory/panel.rs`), the machine recipe table, and the buttons and slot clicks, which
//! queue actions for the local player.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::recipes::MACHINE_RECIPES;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The machine the player right-clicked to open, as `[x, y, z]` (empty if none); asking clears it.
    pub fn take_panel_request(&mut self) -> Vec<i32> {
        self.panel_request.take().map_or_else(Vec::new, |p| vec![p.x, p.y, p.z])
    }

    /// The panel of the machine at a position, flat: `[block, recipe (u32::MAX for none), progress in
    /// thousandths, seconds of fire, 1 if the player chooses the recipe, slot count, then (role, item,
    /// count) per slot]`. Empty if there is no machine with a panel there. Roles: `panel_role_*`.
    pub fn machine_panel(&self, x: i32, y: i32, z: i32) -> Vec<u32> {
        let Some(p) = self.sim.factory.panel(IVec3::new(x, y, z)) else { return Vec::new() };
        let recipe = p.recipe.map_or(u32::MAX, u32::from);
        let mut out = vec![p.block as u32, recipe, p.progress, p.fire, p.choosable as u32, p.slots.len() as u32];
        for (role, s) in p.slots {
            out.extend([role as u32, s.item.0 as u32, s.count]);
        }
        out
    }

    /// The panel's status line.
    pub fn machine_status(&self, x: i32, y: i32, z: i32) -> String {
        self.sim.factory.panel(IVec3::new(x, y, z)).map_or_else(String::new, |p| p.status)
    }

    pub fn panel_role_input(&self) -> u32 {
        crate::factory::ROLE_INPUT as u32
    }

    pub fn panel_role_fuel(&self) -> u32 {
        crate::factory::ROLE_FUEL as u32
    }

    pub fn panel_role_output(&self) -> u32 {
        crate::factory::ROLE_OUTPUT as u32
    }

    /// Indices of the machine recipes the machine block `block` makes.
    pub fn machine_recipes(&self, block: u8) -> Vec<u32> {
        (0..MACHINE_RECIPES.len() as u32).filter(|&i| MACHINE_RECIPES[i as usize].machine == block).collect()
    }

    /// A machine recipe's inputs as flat (item, count) pairs.
    pub fn machine_recipe_inputs(&self, i: u32) -> Vec<u32> {
        MACHINE_RECIPES
            .get(i as usize)
            .map_or_else(Vec::new, |r| r.inputs.iter().flat_map(|&(it, n)| [it.0 as u32, n]).collect())
    }

    /// A machine recipe's output as `[item, count]`.
    pub fn machine_recipe_output(&self, i: u32) -> Vec<u32> {
        MACHINE_RECIPES.get(i as usize).map_or_else(Vec::new, |r| vec![r.output.0 .0 as u32, r.output.1])
    }

    pub fn machine_recipe_seconds(&self, i: u32) -> f64 {
        MACHINE_RECIPES.get(i as usize).map_or(0.0, |r| r.seconds)
    }

    /// Whether the machine at a position would take `item` from the inventory now.
    pub fn machine_wants(&self, x: i32, y: i32, z: i32, item: u16) -> bool {
        self.sim.factory.wants(IVec3::new(x, y, z), ItemId(item))
    }

    /// Chooses what the machine makes (`-1` for nothing); the inputs it held come back (next tick).
    pub fn set_machine_recipe(&mut self, x: i32, y: i32, z: i32, recipe: i32) {
        let recipe = u16::try_from(recipe).unwrap_or(u16::MAX);
        self.act(Action::SetRecipe { pos: IVec3::new(x, y, z), recipe });
    }

    /// The filter's chosen item (0 for none), or `u32::MAX` if the machine there isn't a filter.
    pub fn machine_filter(&self, x: i32, y: i32, z: i32) -> u32 {
        let p = self.sim.factory.panel(IVec3::new(x, y, z));
        p.and_then(|p| p.filter).map_or(u32::MAX, |f| f.0 as u32)
    }

    /// Sets what the filter sends straight on (0 for nothing; next tick).
    pub fn set_machine_filter(&mut self, x: i32, y: i32, z: i32, item: u16) {
        self.act(Action::SetFilter { pos: IVec3::new(x, y, z), item: ItemId(item) });
    }

    /// Puts as many of `item` from the inventory into the machine as it takes (next tick).
    pub fn insert_into_machine(&mut self, x: i32, y: i32, z: i32, item: u16) {
        self.act(Action::Insert { pos: IVec3::new(x, y, z), item: ItemId(item) });
    }

    /// The box's slots as flat (item, count) pairs; empty if there is no box there.
    pub fn box_slots(&self, x: i32, y: i32, z: i32) -> Vec<u32> {
        let slots = self.sim.factory.box_slots(IVec3::new(x, y, z)).unwrap_or_default();
        slots.iter().flat_map(|s| [s.item.0 as u32, s.count]).collect()
    }

    /// Box-screen click on box slot `slot` (with the cursor stack; `shift` moves it to the inventory).
    pub fn click_box(&mut self, x: i32, y: i32, z: i32, slot: u8, shift: bool) {
        self.act(Action::ClickBox { pos: IVec3::new(x, y, z), slot, shift });
    }

    /// Box-screen shift-click on inventory slot `slot`: moves its stack into the box.
    pub fn store_slot(&mut self, x: i32, y: i32, z: i32, slot: u8) {
        self.act(Action::StoreSlot { pos: IVec3::new(x, y, z), slot });
    }

    /// Takes the machine's output into the inventory (next tick).
    pub fn take_machine_output(&mut self, x: i32, y: i32, z: i32) {
        self.act(Action::TakeContents { pos: IVec3::new(x, y, z) });
    }
}
