//! Note signs for the host (`signs/`): what a sign says for its editor (`web/src/ui/sign.ts`, opened through
//! `take_panel_request` like a logbook) and for painting its face layers (`web/src/render/sign-faces.ts`), and the edit.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::math::IVec3;
use crate::signs::{line_bytes, LINES, LINE_BYTES};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The lines of the sign with a cell at a block, split by newlines (always `LINES` of them); empty when there is none.
    pub fn sign_text(&self, x: i32, y: i32, z: i32) -> String {
        self.sim.signs.at(IVec3::new(x, y, z)).map_or_else(String::new, |s| s.lines.join("\n"))
    }

    /// The sign's board and ink colours (0xRRGGBB); empty when there is none.
    pub fn sign_colours(&self, x: i32, y: i32, z: i32) -> Vec<u32> {
        self.sim.signs.at(IVec3::new(x, y, z)).map_or_else(Vec::new, |s| vec![s.back, s.ink])
    }

    /// Bytes of UTF-8 a sign's line holds (the editor's limit).
    pub fn sign_line_bytes(&self) -> u32 {
        LINE_BYTES as u32
    }

    /// Writes the sign with a cell at a block: `text`'s first `LINES` lines (each cut to what fits) and the colours.
    pub fn edit_sign(&mut self, x: i32, y: i32, z: i32, text: &str, back: u32, ink: u32) {
        let pos = IVec3::new(x, y, z);
        let mut lines = text.split('\n');
        for line in 0..LINES as u8 {
            let text = line_bytes(lines.next().unwrap_or(""));
            self.act(Action::EditSign { pos, line, text, back, ink });
        }
    }

    /// The first texture layer of the signs' text (`tex::SIGN_FACE`): slot `i` paints layers `2i` and `2i + 1` after it.
    pub fn sign_face_layer(&self) -> u32 {
        crate::block::tex::SIGN_FACE as u32
    }

    /// Each text slot's sign this frame: x, y, z and a stamp that changes with its text and colours (0: empty slot).
    pub fn sign_faces(&self) -> Vec<i32> {
        self.sign_slot_list()
    }
}
