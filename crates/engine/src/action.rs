//! Actions: the only way anything changes the core (`Sim`). Players' input, the authority (pickups)
//! and debug helpers all queue actions with `Sim::queue`; `Sim::step` applies them at a tick boundary.
//!
//! Actions carry resolved data (positions, slots, facing), never "what the player is looking at", so
//! any peer can apply them without that player's camera. `apply` validates against the current state
//! and quietly does nothing when an action no longer fits (the block changed, the slot is empty).
//! Results leave as `SimEvent`s. To add an action: a variant here, its arm in
//! `Sim::apply_to_player` (or in `Sim::apply` if it doesn't need the player to be here yet) and its
//! bytes in `action/codec.rs` (co-op sends actions to every peer).

use crate::block::BlockId;
use crate::factory::Job;
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
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
    /// Chooses the rule of the sensor at `pos` (`factory::sensor::RULES` index).
    SetSensor {
        pos: IVec3,
        rule: u8,
    },
    /// Plants a ghost (`ghosts.rs`) of what the item in `slot` places, at `pos` and `facing`; nothing is used.
    PlaceGhost {
        pos: IVec3,
        slot: u8,
        facing: u8,
    },
    /// Plants a ghost of `block` at `pos` outright (a stamped blueprint: `blueprint/`); nothing is used.
    PlantGhost {
        pos: IVec3,
        block: BlockId,
        facing: u8,
        tier: u8,
    },
    /// Marks the block (or machine) at `pos` for tear-down: a ghost of air that drones break (`drones/`).
    MarkRemoval {
        pos: IVec3,
    },
    /// Starts or stops the jetpack's thrust (`helpers/`): the player holds jump in the air with one in the pack.
    Jetpack {
        on: bool,
    },
    /// Sends the personal drone to fetch `item` from the nearest box to `at`, the player's cell (`helpers/`).
    Fetch {
        item: ItemId,
        at: IVec3,
    },
    /// Removes the ghost covering `pos`.
    RemoveGhost {
        pos: IVec3,
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
    /// Marks a tunnel site from the block `from` towards `to` with section `size` (`factory::Tunnel::new`,
    /// `Sites::mark_tunnel`, which refuse what doesn't fit).
    MarkTunnel {
        from: IVec3,
        to: IVec3,
        size: u8,
    },
    /// Removes the terraforming site with this id.
    RemoveSite {
        id: u32,
    },
    /// Wires the power pole at `pole` to the machine or pole standing at `to` (`factory::Factory::connect`:
    /// only if it is in range and both have a free slot). A machine on another pole moves to this one.
    Connect {
        pole: IVec3,
        to: IVec3,
    },
    /// Cuts the wire between the pole at `pole` and what stands at `to`.
    Disconnect {
        pole: IVec3,
        to: IVec3,
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
    /// Queues `times` crafts of a hand recipe (`crafting.rs`), with the crafts of any missing parts; what
    /// the inventory can't pay for is left out.
    Craft {
        recipe: u16,
        times: u32,
    },
    /// Cancels the player's `order`th queued craft, giving back what it took.
    CancelCraft {
        order: u16,
    },
    /// Equipment-panel click on worn-gear `slot` (`equipment.rs`): the cursor stack swaps with the gear (only gear for
    /// that slot goes on); `shift` takes it off into the inventory.
    ClickGear {
        slot: u8,
        shift: bool,
    },
    /// Inventory-screen click; `shift` moves the stack between hotbar and backpack (gear goes on instead).
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
    /// Shift-right-click on inventory `slot`: moves every stack of its item between hotbar and backpack
    /// (until the other side is full).
    QuickMoveAll {
        slot: u8,
    },
    /// Box-screen shift-right-click on inventory `slot`: moves every stack of its item into the box.
    StoreAll {
        pos: IVec3,
        slot: u8,
    },
    /// Box-screen shift-right-click on the box's `slot`: moves every stack of its item into the inventory.
    TakeAll {
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
                    self.players[slot] =
                        Some(PlayerCore { inventory, key, crafts: Default::default(), helpers: Default::default() });
                }
            }
            Action::Leave { pos } => {
                let Some(mut core) = self.players.get_mut(slot).and_then(Option::take) else { return };
                core.crafts.cancel_all(&mut core.inventory, player, &mut self.events);
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
            Action::SetSensor { pos, rule } => self.factory.set_sensor(pos, rule),
            Action::PlaceGhost { pos, slot, facing } => self.place_ghost(player, pos, slot, facing),
            Action::PlantGhost { pos, block, facing, tier } => self.plant_ghost(pos, block, facing, tier),
            Action::MarkRemoval { pos } => self.mark_removal(pos),
            Action::Jetpack { on } => self.set_thrust(player, on),
            Action::Fetch { item, at } => self.start_fetch(player, item, at),
            Action::RemoveGhost { pos } => {
                self.ghosts.remove_at(pos);
            }
            Action::SetQuarry { pos, width, depth, paused } => self.factory.set_quarry(pos, width, depth, paused),
            Action::Rotate { pos } => {
                self.factory.rotate(pos);
            }
            Action::Connect { pole, to } => self.factory.connect(pole, to),
            Action::Disconnect { pole, to } => self.factory.disconnect(pole, to),
            Action::MarkSite { a, b, level, job } => {
                self.factory.sites.mark(&mut self.world, a, b, level, job);
            }
            Action::MarkTunnel { from, to, size } => {
                self.factory.sites.mark_tunnel(from, to, size);
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
                core.crafts.enqueue(inv, &self.factory.research, recipe, times);
            }
            Action::CancelCraft { order } => core.crafts.cancel(order as usize, inv, player, &mut self.events),
            Action::ClickGear { .. }
            | Action::ClickSlot { .. }
            | Action::ClickBox { .. }
            | Action::StoreSlot { .. }
            | Action::QuickMoveAll { .. }
            | Action::StoreAll { .. }
            | Action::TakeAll { .. }
            | Action::SortInventory
            | Action::SortBox { .. } => self.apply_click(player, action),
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
}

mod blocks;
mod clicks;
mod codec;
mod multiblock;
#[cfg(test)]
mod tests;
