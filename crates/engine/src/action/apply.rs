//! `Sim::apply`: what each action does when its tick comes. Joining and leaving are handled first (they
//! do not need the player to be here yet); everything else goes through `apply_to_player`, one arm per
//! action, which hands the longer ones to `blocks.rs` and `clicks.rs` or to the module that owns the rule.
//! To add an action: its variant in `action.rs`, its arm here and its bytes in `codec.rs`.

use crate::research::Unlock;
use crate::sim::{Away, PlayerCore, PlayerId, Sim, SimEvent};
use crate::tools;

use super::Action;

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
            Action::PlaceTrain { pos, slot } => self.place_train(player, pos, slot),
            Action::TakeTrain { pos } => self.take_train(player, pos),
            Action::TrainStop { node, dock, clear } => {
                self.factory.set_stop(node, dock, clear);
            }
            Action::ToggleSignal { pos, slot } => self.toggle_signal(player, pos, slot),
            Action::SetRoute { from, to, clear } => self.set_route(from, to, clear),
            Action::Hover { on } => self.set_hover(player, on),
            Action::Charge { pole, on } => self.set_charge(player, pole, on),
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
            | Action::RightClickSlot { .. }
            | Action::RightClickBox { .. }
            | Action::ThrowCursor
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
