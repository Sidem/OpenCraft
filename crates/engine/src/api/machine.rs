//! Machine panels (`web/src/ui/machine.ts`, the quarry's part in `ui/quarry.ts`) and box screens
//! (`ui/inventory.ts`): which panel to open, what it shows (`factory/panel.rs`), the machine recipe
//! table, and the buttons and slot clicks, which queue actions for the local player.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::factory;
use crate::factory::makes;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::recipes::MACHINE_RECIPES;
use crate::research::Unlock;
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
        (0..MACHINE_RECIPES.len() as u32).filter(|&i| makes(block, MACHINE_RECIPES[i as usize].category)).collect()
    }

    /// The tech whose research unlocks machine recipe `i`, or -1 if it isn't locked.
    pub fn machine_recipe_locked_by(&self, i: u32) -> i32 {
        let unlock = Unlock::MachineRecipe(i.min(u16::MAX as u32) as u16);
        self.sim.factory.research.locked_by(unlock).map_or(-1, i32::from)
    }

    /// A machine recipe's inputs as flat (item, count) pairs.
    pub fn machine_recipe_inputs(&self, i: u32) -> Vec<u32> {
        MACHINE_RECIPES
            .get(i as usize)
            .map_or_else(Vec::new, |r| r.inputs.iter().flat_map(|&(it, n)| [it.0 as u32, n]).collect())
    }

    /// A machine recipe's outputs as flat (item, count) pairs, the main product first.
    pub fn machine_recipe_output(&self, i: u32) -> Vec<u32> {
        MACHINE_RECIPES
            .get(i as usize)
            .map_or_else(Vec::new, |r| r.outputs.iter().flat_map(|&(it, n)| [it.0 as u32, n]).collect())
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

    /// The quarry panel's figures: `[width choice, depth choice, 1 if paused, layer, layers, blocks dug,
    /// blocks left (in loaded ground)]`; empty if there is no quarry there. Choices index
    /// `quarry_widths` and `quarry_depth_label`.
    pub fn quarry_panel(&self, x: i32, y: i32, z: i32) -> Vec<u32> {
        let Some(q) = self.sim.factory.quarry(IVec3::new(x, y, z)) else { return Vec::new() };
        let (left, _) = factory::survey(&q.dig_box(), q.next, &self.sim.world);
        let (layer, layers) = q.layer();
        vec![q.width as u32, q.depth as u32, q.paused as u32, layer, layers, q.dug, left]
    }

    /// What the quarry uncovered, a line per deposit.
    pub fn quarry_found(&self, x: i32, y: i32, z: i32) -> String {
        self.sim.factory.quarry(IVec3::new(x, y, z)).map_or_else(String::new, |q| q.found_lines().join("\n"))
    }

    /// The quarry's box as `[x0, y0, z0, x1, y1, z1]` (lowest and highest cells); empty if none there.
    pub fn quarry_box(&self, x: i32, y: i32, z: i32) -> Vec<i32> {
        self.sim.factory.quarry(IVec3::new(x, y, z)).map_or_else(Vec::new, |q| {
            let (lo, hi) = q.dig_box().bounds();
            vec![lo.x, lo.y, lo.z, hi.x, hi.y, hi.z]
        })
    }

    /// The box widths a quarry offers, in choice order.
    pub fn quarry_widths(&self) -> Vec<u32> {
        factory::WIDTHS.iter().map(|&w| w as u32).collect()
    }

    pub fn quarry_depth_count(&self) -> u32 {
        factory::DEPTHS.len() as u32
    }

    /// A depth choice as the panel shows it ("16 deep", "to bedrock").
    pub fn quarry_depth_label(&self, i: u32) -> String {
        factory::DEPTHS.get(i as usize).map_or_else(String::new, |d| d.label())
    }

    /// Sets the quarry's box and pause (next tick; a new box starts over from its top).
    pub fn set_quarry(&mut self, x: i32, y: i32, z: i32, width: u8, depth: u8, paused: bool) {
        self.act(Action::SetQuarry { pos: IVec3::new(x, y, z), width, depth, paused });
    }

    /// Takes the machine's output into the inventory (next tick).
    pub fn take_machine_output(&mut self, x: i32, y: i32, z: i32) {
        self.act(Action::TakeContents { pos: IVec3::new(x, y, z) });
    }
}
