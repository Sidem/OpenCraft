//! Actions: the only way anything changes the core (`Sim`). Players' input, the authority (pickups)
//! and debug helpers all queue actions with `Sim::queue`; `Sim::step` applies them at a tick boundary.
//!
//! Actions carry resolved data (positions, slots, facing), never "what the player is looking at", so
//! any peer can apply them without that player's camera. `apply` validates against the current state
//! and quietly does nothing when an action no longer fits (the block changed, the slot is empty).
//! Results leave as `SimEvent`s. To add an action: a variant here, its arm in
//! `Sim::apply_to_player` (or in `Sim::apply` if it doesn't need the player to be here yet) and its
//! bytes in `action/codec.rs` (co-op sends actions to every peer).

use crate::block::{self, BlockId, AIR, LEAVES, SAPLING};
use crate::factory::{self, Job};
use crate::inventory::{add_to_slots, click_stack, sort_stacks, Stack};
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::recipes::RECIPES;
use crate::research::Unlock;
use crate::sim::{Away, PlayerCore, PlayerId, Sim, SimEvent};
use crate::tools;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    /// Breaks the block at `pos` by hand, with whatever the player holds: ore keeps `HAND_YIELD`
    /// (more with a good pickaxe: tools.rs) and costs its deposit a block; a tool for the block loses a
    /// use; machines drop their contents.
    BreakBlock {
        pos: IVec3,
    },
    /// Places one item from `slot` at `pos`. `facing` is the belt direction (`factory::dir_from_yaw`;
    /// a multi-block machine's turn); `against` is the clicked block, which gives a miner its drill face
    /// and deposit. A multi-block machine takes `pos` as its anchor and needs every cell free.
    PlaceBlock {
        pos: IVec3,
        slot: u8,
        facing: u8,
        against: IVec3,
    },
    /// Turns the belt or router at `pos` a quarter turn clockwise (the R key).
    Rotate {
        pos: IVec3,
    },
    /// Empties the box or miner at `pos` into the inventory, or takes a machine's output.
    TakeContents {
        pos: IVec3,
    },
    /// Chooses what the machine at `pos` makes (`MACHINE_RECIPES` index; `u16::MAX` for nothing).
    /// The inputs it held go back to the player.
    SetRecipe {
        pos: IVec3,
        recipe: u16,
    },
    /// Sets what the filter at `pos` sends straight on (`NONE` for nothing).
    SetFilter {
        pos: IVec3,
        item: ItemId,
    },
    /// Sets the quarry at `pos`'s box (`factory::WIDTHS` and `DEPTHS` indices; a new box starts over)
    /// and whether it is paused.
    SetQuarry {
        pos: IVec3,
        width: u8,
        depth: u8,
        paused: bool,
    },
    /// Marks a terraforming site over the columns (x, z) between corners `a` and `b`: `job` to
    /// `level` (`factory::Sites::mark`, which refuses a site that doesn't fit).
    MarkSite {
        a: (i32, i32),
        b: (i32, i32),
        level: i32,
        job: Job,
    },
    /// Removes the terraforming site with this id.
    RemoveSite {
        id: u32,
    },
    /// Raises the tiered machine at `pos` one tier with kits from the inventory (`factory/upgrades.rs`),
    /// if research allows it and there are enough.
    Upgrade {
        pos: IVec3,
    },
    /// Chooses what every lab in the world researches (`u8::MAX` to stop); only an available tech.
    SetResearch {
        tech: u8,
    },
    /// Puts as many of `item` from the inventory into the machine at `pos` as it takes.
    Insert {
        pos: IVec3,
        item: ItemId,
    },
    Craft {
        recipe: u16,
        times: u32,
    },
    /// Inventory-screen click; `shift` moves the stack between hotbar and backpack.
    ClickSlot {
        slot: u8,
        shift: bool,
    },
    /// Box-screen click on the box's `slot`: like `ClickSlot` with the cursor stack; `shift` moves the
    /// stack into the inventory (what doesn't fit stays).
    ClickBox {
        pos: IVec3,
        slot: u8,
        shift: bool,
    },
    /// Box-screen shift-click on inventory `slot`: moves the stack into the box (what doesn't fit stays).
    StoreSlot {
        pos: IVec3,
        slot: u8,
    },
    /// Sorts the backpack (the hotbar keeps its layout): stacks merged, then ordered by item.
    SortInventory,
    /// Sorts the slots of the box at `pos` the same way.
    SortBox {
        pos: IVec3,
    },
    /// The inventory screen closed: the cursor stack goes back, or is thrown if there is no room.
    CloseInventory,
    SelectSlot {
        slot: u8,
    },
    ScrollSlot {
        delta: i8,
    },
    DropSelected {
        count: u32,
    },
    /// Issued by the authority when a loose item reaches the player.
    PickUp {
        item: ItemId,
        count: u32,
    },
    /// Debug and creative only.
    Give {
        item: ItemId,
        count: u32,
    },
    /// The player joins (nothing happens if it is already here): with what it had when it left, if
    /// `key` is away, else with an empty inventory.
    Join {
        key: u64,
    },
    /// The player leaves and its id is free again. With a key, its inventory and `pos` (where its
    /// body stood) wait in `Sim::away`; without one they are gone.
    Leave {
        pos: Vec3,
    },
}

impl Sim {
    pub fn apply(&mut self, player: PlayerId, action: Action) {
        let slot = player.0 as usize;
        match action {
            Action::Join { key } => {
                if self.players.len() <= slot {
                    self.players.resize_with(slot + 1, || None);
                }
                if self.players[slot].is_none() {
                    let back = self.away.iter().position(|a| key != 0 && a.key == key);
                    let inventory = back.map(|i| self.away.remove(i).inventory).unwrap_or_default();
                    self.players[slot] = Some(PlayerCore { inventory, key });
                }
            }
            Action::Leave { pos } => {
                let Some(core) = self.players.get_mut(slot).and_then(Option::take) else { return };
                if core.key != 0 {
                    self.away.retain(|a| a.key != core.key);
                    self.away.push(Away { key: core.key, pos, inventory: core.inventory });
                }
            }
            _ => self.apply_to_player(player, action),
        }
    }

    /// Everything except joining and leaving, which needs the player to be here.
    fn apply_to_player(&mut self, player: PlayerId, action: Action) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let inv = &mut core.inventory;
        match action {
            Action::Join { .. } | Action::Leave { .. } => {}
            Action::BreakBlock { pos } => self.break_block(player, pos),
            Action::PlaceBlock { pos, slot, facing, against } => self.place_block(player, pos, slot, facing, against),
            Action::TakeContents { pos } => {
                let events = &mut self.events;
                self.factory.take_contents(pos, |item, n| {
                    let taken = n - inv.add(item, n);
                    if taken > 0 {
                        events.push(SimEvent::Gained { player, item, count: taken });
                    }
                    taken
                });
            }
            Action::SetRecipe { pos, recipe } => {
                let recipe = (recipe != u16::MAX).then_some(recipe);
                if recipe.is_some_and(|i| !self.factory.research.has(Unlock::MachineRecipe(i))) {
                    return;
                }
                for s in self.factory.set_recipe(pos, recipe).unwrap_or_default() {
                    let left = inv.add(s.item, s.count);
                    if left > 0 {
                        self.events.push(SimEvent::Thrown { player, item: s.item, count: left });
                    }
                }
            }
            Action::SetFilter { pos, item } => self.factory.set_filter(pos, item),
            Action::SetQuarry { pos, width, depth, paused } => self.factory.set_quarry(pos, width, depth, paused),
            Action::Rotate { pos } => {
                self.factory.rotate(pos);
            }
            Action::MarkSite { a, b, level, job } => {
                self.factory.sites.mark(&mut self.world, a, b, level, job);
            }
            Action::RemoveSite { id } => {
                self.factory.sites.remove(id);
            }
            Action::Upgrade { pos } => {
                let Some(step) = self.factory.next_upgrade(pos) else { return };
                if !self.factory.research.has(Unlock::Upgrade(step.block, step.tier)) || inv.count(step.kit) < step.kits
                {
                    return;
                }
                inv.remove(step.kit, step.kits);
                self.factory.upgrade(pos);
                self.events.push(SimEvent::BlockPlaced { player, pos, block: step.block });
            }
            Action::SetResearch { tech } => self.factory.research.set_current((tech != u8::MAX).then_some(tech)),
            Action::Insert { pos, item } => {
                let put = self.factory.insert(pos, item, inv.count(item));
                if put > 0 {
                    inv.remove(item, put);
                }
            }
            Action::Craft { recipe, times } => {
                let Some(r) = RECIPES.get(recipe as usize) else { return };
                if !self.factory.research.has(Unlock::Recipe(r.output)) {
                    return;
                }
                let mut done = 0;
                while done < times && r.affordable(inv) > 0 {
                    for &(item, n) in r.inputs {
                        inv.remove(item, n);
                    }
                    let left = inv.add(r.output, r.count);
                    if left > 0 {
                        self.events.push(SimEvent::Thrown { player, item: r.output, count: left });
                    }
                    done += 1;
                }
                if done > 0 {
                    self.events.push(SimEvent::Crafted { player, item: r.output, count: r.count * done });
                }
            }
            Action::ClickSlot { slot, shift: true } => inv.quick_move(slot as usize),
            Action::ClickSlot { slot, shift: false } => inv.click(slot as usize),
            Action::ClickBox { pos, slot, shift } => {
                let Some(s) = self.factory.box_slots_mut(pos).and_then(|b| b.get_mut(slot as usize)) else { return };
                if shift && !s.is_empty() {
                    s.count = inv.add(s.item, s.count);
                    if s.is_empty() {
                        *s = Stack::default();
                    }
                } else if !shift && click_stack(s, &mut inv.cursor) {
                    inv.version += 1;
                }
            }
            Action::StoreSlot { pos, slot } => {
                let Some(b) = self.factory.box_slots_mut(pos) else { return };
                let Some((item, n)) = inv.take_slot(slot as usize, u32::MAX) else { return };
                let left = add_to_slots(b, item, n);
                if left > 0 {
                    inv.slots[slot as usize] = Stack { item, count: left };
                }
            }
            Action::SortInventory => inv.sort(),
            Action::SortBox { pos } => {
                if let Some(b) = self.factory.box_slots_mut(pos) {
                    sort_stacks(b);
                }
            }
            Action::CloseInventory => {
                let left = inv.return_cursor();
                if !left.is_empty() {
                    self.events.push(SimEvent::Thrown { player, item: left.item, count: left.count });
                }
            }
            Action::SelectSlot { slot } => inv.select(slot as usize),
            Action::ScrollSlot { delta } => inv.scroll(delta as i32),
            Action::DropSelected { count } => {
                // A tool's count is its uses, so it goes whole.
                let whole = tools::tool(inv.selected_stack().item).is_some();
                if let Some((item, count)) = inv.take_slot(inv.selected, if whole { u32::MAX } else { count }) {
                    self.events.push(SimEvent::Thrown { player, item, count });
                }
            }
            Action::PickUp { item, count } => {
                let left = inv.add(item, count);
                if left < count {
                    self.events.push(SimEvent::Gained { player, item, count: count - left });
                }
                if left > 0 {
                    self.events.push(SimEvent::Thrown { player, item, count: left });
                }
            }
            Action::Give { item, count } => {
                if item.is_valid() {
                    inv.add(item, count);
                }
            }
        }
    }

    fn break_block(&mut self, player: PlayerId, pos: IVec3) {
        // A multi-block machine breaks whole, from any of its cells, as its anchor (`multiblock.rs`).
        let pos = self.factory.footprint_at(pos).map_or(pos, |f| f.0);
        let id = self.world.block_anywhere_or_generate(pos);
        let def = block::def(id);
        if id == AIR || def.break_time < 0.0 {
            return;
        }
        let ore = block::is_ore(id);
        if ore {
            self.factory.deposits.hand_mined(&mut self.world, pos);
        }
        if !self.world.set_block_anywhere(pos, AIR) {
            return;
        }
        self.events.push(SimEvent::BlockBroken { player, pos, block: id });
        self.block_changed(pos, id);
        self.clear_parts(pos);
        let held = self.wear_tool(player, id);
        // A tiered machine drops its tier's item (`factory/tiers.rs`).
        let tiered = self.factory.tiered_at(pos).and_then(|t| factory::tiers::item_of(def.drop, t.1));
        let mut drops = self.factory.remove(pos);
        if def.drop != AIR {
            let item = tiered.unwrap_or(ItemId::block(def.drop));
            drops.insert(0, Stack { item, count: if ore { tools::ore_yield(held) } else { 1 } });
        }
        if id == LEAVES && self.leaf_drops_sapling() {
            drops.push(Stack { item: SAPLING.into(), count: 1 });
        }
        let center = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        for s in drops {
            let vel = Vec3::new(self.rng.range(-1.5, 1.5), 4.0, self.rng.range(-1.5, 1.5));
            self.events.push(SimEvent::Dropped { pos: center, vel, item: s.item, count: s.count });
        }
    }

    /// The item the player breaks `block` with (their selected slot). A tool for it loses a use (its
    /// stack count); one that wears out reports it.
    fn wear_tool(&mut self, player: PlayerId, block: BlockId) -> ItemId {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return ItemId::NONE };
        let inv = &mut core.inventory;
        let held = inv.selected_stack().item;
        if tools::tool_for(held, block).is_some() {
            inv.take_slot(inv.selected, 1);
            if inv.selected_stack().is_empty() {
                self.events.push(SimEvent::ToolWornOut { player, item: held });
            }
        }
        held
    }

    fn place_block(&mut self, player: PlayerId, pos: IVec3, slot: u8, facing: u8, against: IVec3) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let inv = &mut core.inventory;
        let Some(stack) = inv.slots.get(slot as usize).copied() else { return };
        let Some(placed) = stack.item.places().filter(|_| !stack.is_empty()) else { return };
        if let Some(fp) = factory::footprint::of(placed) {
            return self.place_footprint(player, pos, slot, (placed, fp), facing);
        }
        if placed == SAPLING && !self.can_plant(pos) || placed == block::TORCH && !self.torch_fits(pos) {
            return;
        }
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let inv = &mut core.inventory;
        let old = self.world.block_anywhere_or_generate(pos);
        if !crate::block::replaceable(old) || !self.world.set_block_anywhere(pos, placed) {
            return;
        }
        inv.take_slot(slot as usize, 1);
        let tier = factory::tiers::placed_by(stack.item).map_or(0, |(_, t)| t);
        self.factory.place(&mut self.world, placed, pos, facing, against, tier);
        self.events.push(SimEvent::BlockPlaced { player, pos, block: placed });
        self.block_changed(pos, old);
    }
}

mod codec;
mod multiblock;
#[cfg(test)]
mod tests;
