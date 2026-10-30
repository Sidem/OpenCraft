//! The blueprint library for the host (`web/src/ui/blueprints.ts`): the names to list, picking one up,
//! renaming and deleting, the keys' marking and copying, the bytes the host keeps with the world, and the
//! prompt offering a blueprint when a machine is built again (`blueprint/`).

use wasm_bindgen::prelude::*;

use crate::blueprint;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Z in ghost mode: mark a box corner (replacing a selected box), finish the box, clear it when aimed at
    /// nothing, or put a held blueprint away.
    pub fn blueprint_mark(&mut self) {
        self.mark_blueprint_corner();
    }

    /// Enter in ghost mode: copy the machines in the selected box as a new blueprint.
    pub fn blueprint_copy_selection(&mut self) {
        self.copy_blueprint();
    }

    /// Changes whenever the list does.
    pub fn blueprint_version(&self) -> u32 {
        self.library.version
    }

    /// One line per blueprint: its name, a tab, and how many pieces (machines, belts, poles…) it holds.
    pub fn blueprint_list(&self) -> String {
        let rows: Vec<String> = self.library.list.iter().map(|b| format!("{}\t{}", b.name, b.entries.len())).collect();
        rows.join("\n")
    }

    /// Blueprint `i` seen from above for the library's picture: its box size (x, y, z), then
    /// `blueprint_plan_fields()` numbers per piece (`blueprint/plan.rs`). Empty when there is no such blueprint.
    pub fn blueprint_plan(&self, i: u32) -> Vec<i32> {
        self.library.list.get(i as usize).map_or_else(Vec::new, |b| b.plan())
    }

    pub fn blueprint_plan_fields(&self) -> u32 {
        blueprint::PLAN_FIELDS as u32
    }

    /// What building blueprint `i` takes, most needed first: `[item, count, carried]` per item.
    pub fn blueprint_needs(&self, i: u32) -> Vec<u32> {
        let Some(bp) = self.library.list.get(i as usize) else { return Vec::new() };
        let inv = self.inventory();
        bp.needs().iter().flat_map(|&(it, n)| [u32::from(it.0), n, inv.count(it)]).collect()
    }

    /// The blueprint in hand, or -1.
    pub fn blueprint_held(&self) -> i32 {
        self.library.held.map_or(-1, |i| i as i32)
    }

    /// Takes blueprint `i` in hand (turns ghost mode on); -1 or out of range puts it away.
    pub fn blueprint_hold(&mut self, i: i32) {
        let held = usize::try_from(i).ok().filter(|&i| i < self.library.list.len());
        self.library.held = held;
        self.library.turns = 0;
        self.library.note.clear();
        if held.is_some() {
            self.ghost_mode = true;
        }
    }

    pub fn blueprint_rename(&mut self, i: u32, name: &str) {
        self.library.rename(i as usize, name);
    }

    pub fn blueprint_delete(&mut self, i: u32) {
        self.library.delete(i as usize);
    }

    /// The tip offering a blueprint after the same machine was built a third time nearby (hint markup), or
    /// empty while there is none (`blueprint/repeat.rs`).
    pub fn blueprint_prompt(&self) -> String {
        self.repeat_prompt()
    }

    /// H on that tip: hides it.
    pub fn blueprint_prompt_dismiss(&mut self) {
        self.dismiss_repeat_prompt();
    }

    /// The library as bytes for the host to keep.
    pub fn blueprint_export(&self) -> Vec<u8> {
        blueprint::export(&self.library.list)
    }

    /// Loads bytes from `blueprint_export`; false (and nothing changes) when they are damaged.
    pub fn blueprint_import(&mut self, bytes: &[u8]) -> bool {
        let Some(list) = blueprint::import(bytes) else { return false };
        self.library.list = list;
        self.library.held = None;
        self.library.version += 1;
        true
    }
}
