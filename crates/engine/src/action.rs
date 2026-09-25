//! Actions: the only way anything changes the core (`Sim`). Players' input, the authority (pickups)
//! and debug helpers all queue actions with `Sim::queue`; `Sim::step` applies them at a tick boundary.
//!
//! Actions carry resolved data (positions, slots, facing), never "what the player is looking at", so
//! any peer can apply them without that player's camera. `apply` validates against the current state
//! and quietly does nothing when an action no longer fits (the block changed, the slot is empty).
//! Results leave as `SimEvent`s. To add an action: a variant here and its arm in
//! `Sim::apply_to_player` (or in `Sim::apply` if it doesn't need the player to be here yet).

use crate::block::{self, AIR};
use crate::deposits::HAND_YIELD;
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::recipes::RECIPES;
use crate::sim::{PlayerCore, PlayerId, Sim, SimEvent};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    /// Breaks the block at `pos` by hand: ore keeps [`HAND_YIELD`] and costs its deposit a block;
    /// machines drop their contents.
    BreakBlock {
        pos: IVec3,
    },
    /// Places one item from `slot` at `pos`. `facing` is the belt direction (`factory::dir_from_yaw`);
    /// `against` is the clicked block, which gives a miner its drill face and deposit.
    PlaceBlock {
        pos: IVec3,
        slot: u8,
        facing: u8,
        against: IVec3,
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
    /// The player joins with an empty inventory (nothing happens if it is already here).
    Join,
    /// The player leaves; its inventory goes with it and its id is free again.
    Leave,
}

impl Sim {
    pub fn apply(&mut self, player: PlayerId, action: Action) {
        let slot = player.0 as usize;
        match action {
            Action::Join => {
                if self.players.len() <= slot {
                    self.players.resize_with(slot + 1, || None);
                }
                self.players[slot].get_or_insert_with(PlayerCore::default);
            }
            Action::Leave => {
                if let Some(core) = self.players.get_mut(slot) {
                    *core = None;
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
            Action::Join | Action::Leave => {}
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
                for s in self.factory.set_recipe(pos, recipe).unwrap_or_default() {
                    let left = inv.add(s.item, s.count);
                    if left > 0 {
                        self.events.push(SimEvent::Thrown { player, item: s.item, count: left });
                    }
                }
            }
            Action::SetFilter { pos, item } => self.factory.set_filter(pos, item),
            Action::Insert { pos, item } => {
                let put = self.factory.insert(pos, item, inv.count(item));
                if put > 0 {
                    inv.remove(item, put);
                }
            }
            Action::Craft { recipe, times } => {
                let Some(r) = RECIPES.get(recipe as usize) else { return };
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
            Action::CloseInventory => {
                let left = inv.return_cursor();
                if !left.is_empty() {
                    self.events.push(SimEvent::Thrown { player, item: left.item, count: left.count });
                }
            }
            Action::SelectSlot { slot } => inv.select(slot as usize),
            Action::ScrollSlot { delta } => inv.scroll(delta as i32),
            Action::DropSelected { count } => {
                if let Some((item, count)) = inv.take_slot(inv.selected, count) {
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
        let mut drops = self.factory.remove(pos);
        if def.drop != AIR {
            drops.insert(0, Stack { item: ItemId::block(def.drop), count: if ore { HAND_YIELD } else { 1 } });
        }
        let center = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        for s in drops {
            let vel = Vec3::new(self.rng.range(-1.5, 1.5), 4.0, self.rng.range(-1.5, 1.5));
            self.events.push(SimEvent::Dropped { pos: center, vel, item: s.item, count: s.count });
        }
    }

    fn place_block(&mut self, player: PlayerId, pos: IVec3, slot: u8, facing: u8, against: IVec3) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let inv = &mut core.inventory;
        let Some(stack) = inv.slots.get(slot as usize).copied() else { return };
        let Some(placed) = stack.item.places().filter(|_| !stack.is_empty()) else { return };
        if self.world.block_anywhere_or_generate(pos) != AIR || !self.world.set_block_anywhere(pos, placed) {
            return;
        }
        inv.take_slot(slot as usize, 1);
        self.factory.place(&mut self.world, placed, pos, facing, against);
        self.events.push(SimEvent::BlockPlaced { player, pos, block: placed });
    }
}

#[cfg(test)]
mod tests;
