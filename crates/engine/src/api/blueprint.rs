//! The blueprint library for the host (`web/src/ui/blueprints.ts`): the names to list, picking one up,
//! renaming and deleting, the keys' marking and copying, and the bytes the host keeps with the world
//! (`blueprint/`).

use wasm_bindgen::prelude::*;

use crate::blueprint;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Z in ghost mode: mark a box corner, finish the box, clear it, or put a held blueprint away.
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

    /// One line per blueprint: its name, a tab, and how many machines it holds.
    pub fn blueprint_list(&self) -> String {
        let rows: Vec<String> = self.library.list.iter().map(|b| format!("{}\t{}", b.name, b.entries.len())).collect();
        rows.join("\n")
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
