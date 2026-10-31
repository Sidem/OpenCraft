//! Note signs (`block::SIGN`): a board two cells wide, from the cell it was placed in to the placer's right, facing the
//! placer, with up to [`LINES`] lines of text in a board and an ink colour the players choose (`Action::EditSign`, the
//! host's editor `web/src/ui/sign.ts`). Core state, saved and hashed; the host draws them (`view.rs`).
//!
//! Invariants: every sign's two cells hold `SIGN` and no other sign's; a line is valid UTF-8 of at most [`LINE_BYTES`]
//! bytes without control characters; colours are 0xRRGGBB. When either cell stops being `SIGN` (broken, dug, built over),
//! `Sim::block_changed` calls `cell_changed`, which forgets the sign and clears its other cell, so only the broken cell
//! drops the sign item. Signs keep the order they were placed in (no hash-map order reaches the state).
//!
//! To change what a sign holds: its fields, `write_state` and `read_state` (with a `SAVE_VERSION` bump).

use crate::block::{self, AIR, SIGN};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::DIRS;
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim, SimEvent};

pub mod view;

/// Lines of text a sign holds.
pub const LINES: usize = 4;
/// Bytes of UTF-8 a line holds (the editor counts them; about 20 characters fit the board at a readable size).
pub const LINE_BYTES: usize = 32;
/// A new sign's colours: pale wood and dark ink.
pub const DEFAULT_BACK: u32 = 0xe6d3a3;
pub const DEFAULT_INK: u32 = 0x2b2118;
/// Every colour's bits.
const RGB: u32 = 0xff_ffff;

#[derive(Clone, PartialEq, Debug)]
pub struct Sign {
    /// The cell it was placed in (the left one, as its placer saw it).
    pub pos: IVec3,
    /// The way its placer faced (`DIRS` index); the text faces the other way.
    pub dir: u8,
    pub back: u32,
    pub ink: u32,
    pub lines: [String; LINES],
}

#[derive(Default)]
pub struct Signs {
    pub list: Vec<Sign>,
}

impl Sign {
    /// Its two cells, the placed one first.
    pub fn cells(&self) -> [IVec3; 2] {
        cells(self.pos, self.dir)
    }
}

/// The cells of a sign placed at `pos` by a player facing `dir`: that cell and the one to the placer's right.
pub fn cells(pos: IVec3, dir: u8) -> [IVec3; 2] {
    [pos, pos + DIRS[(dir as usize + 1) % 4]]
}

/// A line's text from the action's bytes: up to the first zero, if that is valid UTF-8 without control characters.
pub fn line_text(bytes: &[u8; LINE_BYTES]) -> Option<String> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(LINE_BYTES);
    let text = std::str::from_utf8(&bytes[..end]).ok()?;
    (!text.chars().any(char::is_control)).then(|| text.to_string())
}

/// `text` as an action's line bytes: as many whole characters as fit, the rest zeros.
pub fn line_bytes(text: &str) -> [u8; LINE_BYTES] {
    let mut out = [0; LINE_BYTES];
    let mut n = 0;
    for c in text.chars().filter(|c| !c.is_control()) {
        let w = c.len_utf8();
        if n + w > LINE_BYTES {
            break;
        }
        c.encode_utf8(&mut out[n..n + w]);
        n += w;
    }
    out
}

impl Signs {
    /// The sign with a cell at `pos`.
    pub fn at(&self, pos: IVec3) -> Option<&Sign> {
        self.list.iter().find(|s| s.cells().contains(&pos))
    }

    /// Writes line `line` of the sign with a cell at `pos` and sets its colours; nothing if no sign is there or the line
    /// is not one.
    pub fn edit(&mut self, pos: IVec3, line: u8, text: &[u8; LINE_BYTES], back: u32, ink: u32) {
        let Some(sign) = self.list.iter_mut().find(|s| s.cells().contains(&pos)) else { return };
        let Some(text) = line_text(text).filter(|_| (line as usize) < LINES) else { return };
        sign.lines[line as usize] = text;
        (sign.back, sign.ink) = (back & RGB, ink & RGB);
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.list.len());
        for s in &self.list {
            w.ivec3(s.pos);
            w.u8(s.dir);
            w.u32(s.back);
            w.u32(s.ink);
            for line in &s.lines {
                w.count(line.len());
                line.bytes().for_each(|b| w.u8(b));
            }
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Signs> {
        let mut list = Vec::new();
        for _ in 0..r.count()? {
            let (pos, dir, back, ink) = (r.ivec3()?, r.u8()?, r.u32()?, r.u32()?);
            if dir >= 4 || back > RGB || ink > RGB {
                return None;
            }
            let mut lines: [String; LINES] = Default::default();
            for line in &mut lines {
                let n = r.count()?;
                let mut bytes = [0; LINE_BYTES];
                bytes.get_mut(..n)?.copy_from_slice(r.bytes(n)?);
                *line = line_text(&bytes).filter(|t| t.len() == n)?;
            }
            list.push(Sign { pos, dir, back, ink, lines });
        }
        Some(Signs { list })
    }
}

impl Sim {
    /// Puts a blank sign at `pos` and the cell to the right of a player facing `facing`, if both are free (`put_block`).
    pub(crate) fn put_sign(&mut self, player: PlayerId, pos: IVec3, facing: u8) -> bool {
        let dir = facing % 4;
        let cells = cells(pos, dir);
        let olds = cells.map(|c| self.world.block_anywhere_or_generate(c));
        if !olds.iter().all(|&b| block::replaceable(b)) || !self.world.set_block_anywhere(cells[0], SIGN) {
            return false;
        }
        if !self.world.set_block_anywhere(cells[1], SIGN) {
            self.world.set_block_anywhere(cells[0], olds[0]);
            return false;
        }
        let lines = Default::default();
        self.signs.list.push(Sign { pos, dir, back: DEFAULT_BACK, ink: DEFAULT_INK, lines });
        self.events.push(SimEvent::BlockPlaced { player, pos, block: SIGN });
        for (c, old) in cells.into_iter().zip(olds) {
            self.block_changed(c, old);
        }
        true
    }

    /// After the block at `pos` changed from `old`: a sign that lost the cell is gone, and so is its other cell.
    pub(crate) fn sign_cell_changed(&mut self, pos: IVec3, old: block::BlockId) {
        if old != SIGN || self.block(pos) == SIGN {
            return;
        }
        let Some(i) = self.signs.list.iter().position(|s| s.cells().contains(&pos)) else { return };
        let sign = self.signs.list.remove(i);
        for c in sign.cells().into_iter().filter(|&c| c != pos) {
            if self.block(c) == SIGN && self.world.set_block_anywhere(c, AIR) {
                self.block_changed(c, SIGN);
            }
        }
    }
}

#[cfg(test)]
mod tests;
