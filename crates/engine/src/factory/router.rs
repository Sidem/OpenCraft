//! Splitter and filter: belt routers facing the way they were placed. Belts deliver into them from any
//! side; each holds one item at a time and passes it into a belt leading away in front, to the left or
//! to the right (`outs`, rebuilt by `relink`).
//! - Splitter: round robin over the three outputs, skipping any that are missing or blocked.
//! - Filter: its chosen item (set in the panel, `SetFilter`) goes straight on; everything else goes
//!   left or right (round robin). With no item chosen, everything goes to the sides.
//!
//! Invariants: at most one item held; it only leaves into a belt that accepts it, so nothing is lost.

use crate::block::{tex, FILTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};

use super::belt::{Belt, ITEM_SIZE};
use super::panel::{Panel, ROLE_INPUT};
use super::render::push_box;
use super::{Factory, Machine};

pub struct Router {
    pub pos: IVec3,
    pub dir: u8,
    /// A filter (true) or a splitter.
    pub is_filter: bool,
    /// The item a filter sends straight on (`NONE`: nothing chosen).
    pub filter: ItemId,
    /// The item passing through (`NONE`: empty).
    pub held: ItemId,
    /// Belts leading away: front, left, right (derived, from `relink`).
    pub outs: [Option<u32>; 3],
    /// The output to try first (round robin).
    pub next_out: u8,
}

impl Router {
    pub fn new(pos: IVec3, dir: u8, is_filter: bool) -> Router {
        let none = ItemId::NONE;
        Router { pos, dir: dir % 4, is_filter, filter: none, held: none, outs: [None; 3], next_out: 0 }
    }

    /// The horizontal directions of its outputs: front, left, right.
    pub fn out_dirs(&self) -> [u8; 3] {
        [self.dir, (self.dir + 3) % 4, (self.dir + 1) % 4]
    }

    pub fn can_accept(&self) -> bool {
        self.held == ItemId::NONE
    }

    pub fn accept(&mut self, item: ItemId) -> bool {
        let free = self.can_accept();
        if free {
            self.held = item;
        }
        free
    }

    /// Passes the held item into the next output that is allowed for it and accepts it.
    pub fn step(&mut self, belts: &mut [Belt]) {
        if self.held == ItemId::NONE {
            return;
        }
        let allowed = |slot: usize| !self.is_filter || (slot == 0) == (self.held == self.filter);
        for k in 0..3 {
            let slot = (self.next_out as usize + k) % 3;
            let Some(b) = self.outs[slot].filter(|_| allowed(slot)) else { continue };
            if belts[b as usize].accept(self.held, false, 0.0) {
                self.held = ItemId::NONE;
                self.next_out = ((slot + 1) % 3) as u8;
                return;
            }
        }
    }

    fn status_text(&self) -> String {
        match self.filter {
            ItemId::NONE => "No item chosen: everything goes left and right".to_string(),
            f => format!("{} goes straight on; everything else left and right", item::name(f)),
        }
    }

    pub fn panel(&self) -> Panel {
        Panel {
            block: FILTER,
            recipe: None,
            choosable: false,
            progress: 0,
            fire: 0,
            slots: vec![(ROLE_INPUT, Stack { item: self.held, count: (self.held != ItemId::NONE) as u32 })],
            status: self.status_text(),
            filter: Some(self.filter),
        }
    }
}

impl Machine for Router {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: position, facing, kind, filter item, held item, round-robin position.
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.dir);
        w.bool(self.is_filter);
        w.item(self.filter);
        w.item(self.held);
        w.u8(self.next_out);
    }

    fn read_state(r: &mut ByteReader) -> Option<Router> {
        let (pos, dir, is_filter) = (r.ivec3()?, r.u8()?, r.bool()?);
        let mut m = Router::new(pos, dir, is_filter);
        (m.filter, m.held, m.next_out) = (r.item()?, r.item()?, r.u8()?);
        (dir < 4 && m.next_out < 3).then_some(m)
    }

    fn contents(&self) -> Vec<Stack> {
        if self.held == ItemId::NONE {
            Vec::new()
        } else {
            vec![Stack { item: self.held, count: 1 }]
        }
    }

    fn describe(&self, _: &Factory) -> String {
        let used = self.outs.iter().filter(|o| o.is_some()).count();
        let outs = format!("{used} of 3 outputs connected (front, left, right)");
        if self.is_filter {
            format!("{}\n{outs}\nRight-click to choose the item", self.status_text())
        } else {
            format!("Splits items evenly between the belts leading away\n{outs}")
        }
    }

    /// A low housing with an arrow top, and the item passing through.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        let top = if self.is_filter { tex::FILTER_TOP } else { tex::SPLITTER_TOP };
        let yaw = self.dir as f32 * std::f32::consts::FRAC_PI_2;
        push_box(
            out,
            rel + Vec3::new(0.0, -0.36, 0.0),
            yaw,
            [0.96, 0.28, 0.96],
            0.0,
            [top, tex::FRAME, tex::FRAME],
            false,
        );
        if let Some(def) = item::def(self.held).filter(|_| self.held != ItemId::NONE) {
            let size = def.size.map(|s| s * ITEM_SIZE);
            push_box(out, rel + Vec3::new(0.0, -0.22 + size[1] as f64 * 0.5, 0.0), yaw, size, 0.0, def.tex, false);
        }
    }
}
