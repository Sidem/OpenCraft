//! JS API for the Codex (periodic table & achievements, `codex/`): read-only queries only.

use wasm_bindgen::prelude::*;

use crate::codex::{element_info, ACHIEVEMENTS, ELEMENTS, FORMS};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Monotonic count of distinct items obtained; changes whenever the codex needs a UI redraw.
    pub fn codex_obtained_count(&self) -> u32 {
        self.sim.codex.obtained_count()
    }

    /// `[discovered_elements, reachable_elements, unlocked_achievements, total_achievements]`.
    pub fn codex_summary(&self) -> Vec<u32> {
        vec![
            self.sim.codex.discovered_elements() as u32,
            ELEMENTS.len() as u32,
            self.sim.codex.unlocked_count() as u32,
            ACHIEVEMENTS.len() as u32,
        ]
    }

    /// Flat `[z, done_forms, total_forms]` triples for every element reachable in this world.
    pub fn codex_reachable_elements(&self) -> Vec<u32> {
        let mut out = Vec::with_capacity(ELEMENTS.len() * 3);
        for e in ELEMENTS {
            let (done, total) = self.sim.codex.element_forms_done(e.z);
            out.push(u32::from(e.z));
            out.push(done as u32);
            out.push(total as u32);
        }
        out
    }

    /// Flat `[item_id, form_code, obtained_0_or_1]` triples for every in-game form of element `z`.
    pub fn codex_element_forms(&self, z: u8) -> Vec<u32> {
        let mut out = Vec::new();
        for f in FORMS.iter().filter(|f| f.z == z) {
            out.push(u32::from(f.item.0));
            out.push(u32::from(f.form as u8));
            out.push(u32::from(self.sim.codex.has(f.item)));
        }
        out
    }

    /// `\n`-separated chemistry notes for every form of element `z` (same order as `codex_element_forms`).
    pub fn codex_element_form_notes(&self, z: u8) -> String {
        FORMS.iter().filter(|f| f.z == z).map(|f| f.note).collect::<Vec<_>>().join("\n")
    }

    /// Real-world fact and in-game role separated by `\n`, or empty when `z` is not in this world yet.
    pub fn codex_element_info(&self, z: u8) -> String {
        element_info(z).map_or_else(String::new, |e| format!("{}\n{}", e.fact, e.role))
    }

    /// Total number of achievements.
    pub fn codex_achievement_count(&self) -> u32 {
        ACHIEVEMENTS.len() as u32
    }

    /// One achievement as `"title\ndesc\nicon_id\nprogress\ntarget\ndone_0_or_1"`.
    pub fn codex_achievement(&self, index: u32) -> String {
        let Some(a) = ACHIEVEMENTS.get(index as usize) else { return String::new() };
        let (cur, target) = self.sim.codex.achievement_progress(index as usize);
        let done = u32::from(target > 0 && cur >= target);
        format!("{}\n{}\n{}\n{cur}\n{target}\n{done}", a.title, a.desc, a.icon.0)
    }

    /// Bitmask of unlocked achievements (bit `i` = achievement `i`), for HUD unlock toasts.
    pub fn codex_unlocked_mask(&self) -> u32 {
        self.sim.codex.unlocked_mask()
    }
}
