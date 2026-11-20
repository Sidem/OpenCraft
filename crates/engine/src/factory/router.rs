//! Splitter, filter and merger: belt routers facing the way they were placed. Each holds one item at a time.
//! - Splitter: belts deliver into it from any side; round robin over its three outputs (a belt leading away in front,
//!   to the left or to the right: `outs`, rebuilt by `relink`), skipping any that are missing or blocked.
//! - Filter: the splitter, but each output takes the items picked for it in the panel, and one (or, in filters from
//!   before save version 72, both sides) takes everything else (`filter_rules.rs`). A new filter sends everything
//!   straight on.
//! - Merger: the splitter's mirror. Belts never push into it; it pulls the item waiting at the end of its input belts
//!   (behind, left, right: `ins`, rebuilt by `relink`) in turn, skipping any with nothing waiting, so one busy belt
//!   cannot starve the others, and puts everything out in front.
//!
//! None needs power: like belts they work without a connection to the grid. Drawn by `router_model.rs`.
//!
//! Invariants: at most one item held; it only leaves into a belt that accepts it, so nothing is lost. `turn` is below 3.
//! Saved layout: the kind is the byte older saves wrote as the filter flag (0 splitter, 1 filter, 2 merger), and `turn`
//! is the round-robin byte, so adding the merger changed no layout.

use crate::block::{BlockId, FILTER, MERGER, SPLITTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};

use super::belt::Belt;
use super::filter_rules::FilterRules;
use super::links::{Link, Slot};
use super::panel::{Panel, ROLE_INPUT};
use super::provenance::Tag;
use super::{opposite, Factory, Machine};

/// The save version that stores a filter's rules (`filter_rules.rs`) instead of one item.
const RULES_SAVED: u32 = 72;

/// What a router does; the number is its saved byte.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RouterKind {
    Splitter = 0,
    Filter = 1,
    Merger = 2,
}

impl RouterKind {
    pub fn of(block: BlockId) -> RouterKind {
        match block {
            FILTER => RouterKind::Filter,
            MERGER => RouterKind::Merger,
            _ => RouterKind::Splitter,
        }
    }

    pub fn block(self) -> BlockId {
        [SPLITTER, FILTER, MERGER][self as usize]
    }

    fn from_u8(v: u8) -> Option<RouterKind> {
        [RouterKind::Splitter, RouterKind::Filter, RouterKind::Merger].get(v as usize).copied()
    }
}

pub struct Router {
    pub pos: IVec3,
    pub dir: u8,
    pub kind: RouterKind,
    /// What a filter sends where (unused by the others).
    pub rules: FilterRules,
    /// The item passing through (`NONE`: empty).
    pub held: ItemId,
    /// How many of it: 1, or a tool's uses.
    pub held_n: u32,
    /// Its tag, passed on to the belt it leaves into.
    pub held_tag: Tag,
    /// Belts leading away: front, left, right; a merger only ever has the front one (derived, from `relink`).
    pub outs: [Option<u32>; 3],
    /// A merger's input belts, whose end is this router: behind, left, right (derived, from `relink`).
    pub ins: [Option<u32>; 3],
    /// Round robin: the output (splitter, filter) or input (merger) to try first.
    pub turn: u8,
    /// Which output (splitter, filter) or input (merger) the last item went by: presentation only, never saved or
    /// read by the core (the model swings its paddle there).
    pub last: u8,
}

impl Router {
    pub fn new(pos: IVec3, dir: u8, kind: RouterKind) -> Router {
        let none = ItemId::NONE;
        Router {
            pos,
            dir: dir % 4,
            kind,
            rules: FilterRules::default(),
            held: none,
            held_n: 0,
            held_tag: Tag::UNKNOWN,
            outs: [None; 3],
            ins: [None; 3],
            turn: 0,
            last: 0,
        }
    }

    pub fn is_filter(&self) -> bool {
        self.kind == RouterKind::Filter
    }

    /// The horizontal directions of its sides in `outs` and `ins` order: front, left, right.
    pub fn out_dirs(&self) -> [u8; 3] {
        [self.dir, (self.dir + 3) % 4, (self.dir + 1) % 4]
    }

    /// The `ins` slot of a belt heading `dir` into it: behind (0), from the left (1), from the right (2); None from
    /// in front.
    pub fn in_slot(&self, dir: u8) -> Option<usize> {
        // The side it comes from is behind its heading.
        let [front, left, right] = self.out_dirs();
        match opposite(dir) {
            s if s == opposite(front) => Some(0),
            s if s == left => Some(1),
            s if s == right => Some(2),
            _ => None,
        }
    }

    /// Sets the derived links: a merger keeps only its front output.
    pub fn set_links(&mut self, mut outs: [Option<u32>; 3], ins: [Option<u32>; 3]) {
        if self.kind == RouterKind::Merger {
            (outs[1], outs[2]) = (None, None);
        }
        (self.outs, self.ins) = (outs, ins);
    }

    /// Whether belts and machines may push an item in now (never into a merger: it pulls).
    pub fn can_accept(&self) -> bool {
        self.held == ItemId::NONE && self.kind != RouterKind::Merger
    }

    pub fn accept(&mut self, item: ItemId, n: u32, tag: Tag) -> bool {
        let free = self.can_accept();
        if free {
            (self.held, self.held_n, self.held_tag) = (item, n, tag);
        }
        free
    }

    /// Passes the held item into the next output that is allowed for it and accepts it; a merger then pulls its next.
    pub fn step(&mut self, belts: &mut [Belt]) {
        if self.held != ItemId::NONE {
            self.pass_on(belts);
        }
        if self.kind == RouterKind::Merger && self.held == ItemId::NONE {
            self.pull(belts);
        }
    }

    fn pass_on(&mut self, belts: &mut [Belt]) {
        let allowed = |slot: usize| !self.is_filter() || self.rules.allows(self.held, slot);
        for k in 0..3 {
            let slot = if self.kind == RouterKind::Merger { 0 } else { (self.turn as usize + k) % 3 };
            let Some(b) = self.outs[slot].filter(|_| allowed(slot)) else { continue };
            if belts[b as usize].accept(self.held, self.held_n, self.held_tag, false, 0.0) {
                (self.held, self.held_n) = (ItemId::NONE, 0);
                if self.kind != RouterKind::Merger {
                    self.turn = ((slot + 1) % 3) as u8;
                    self.last = slot as u8;
                }
                return;
            }
        }
        // Waiting counts as riding, so the tick it was put on a belt (`ItemKey`) stays the same.
        self.held_tag.age();
    }

    /// A merger takes the item waiting at the end of its next input belt in turn that has one.
    fn pull(&mut self, belts: &mut [Belt]) {
        for k in 0..3 {
            let slot = (self.turn as usize + k) % 3;
            let Some(b) = self.ins[slot] else { continue };
            let belt = &mut belts[b as usize];
            let Some(front) = belt.items.first().copied().filter(|it| it.p >= 1.0) else { continue };
            belt.items.remove(0);
            belt.passed = belt.passed.saturating_add(1);
            (self.held, self.held_n, self.held_tag) = (front.item, front.n, front.tag);
            self.turn = ((slot + 1) % 3) as u8;
            self.last = slot as u8;
            return;
        }
    }

    fn status_text(&self) -> String {
        self.rules.describe()
    }

    pub fn panel(&self) -> Panel {
        Panel {
            block: FILTER,
            recipe: None,
            choosable: false,
            progress: 0,
            fire: 0,
            slots: vec![(ROLE_INPUT, Stack { item: self.held, count: self.held_n })],
            status: self.status_text(),
            filter: Some(self.rules.clone()),
        }
    }
}

impl Machine for Router {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: position, facing, kind, filter rules, held item, round-robin position.
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.dir);
        w.u8(self.kind as u8);
        self.rules.write_state(w);
        w.item(self.held);
        w.u32(self.held_n);
        w.u8(self.turn);
        self.held_tag.write_state(w);
    }

    fn read_state(r: &mut ByteReader) -> Option<Router> {
        let (pos, dir, kind) = (r.ivec3()?, r.u8()?, RouterKind::from_u8(r.u8()?)?);
        let mut m = Router::new(pos, dir, kind);
        // Before version 72 a filter had one item, sent straight on.
        m.rules = if r.version >= RULES_SAVED { FilterRules::read_state(r)? } else { FilterRules::legacy(r.item()?) };
        m.held = r.item()?;
        // Before version 39 a router held a single item (a tool: one use).
        m.held_n = if r.version >= 39 { r.u32()? } else { (m.held != ItemId::NONE) as u32 };
        m.turn = r.u8()?;
        m.held_tag = Tag::read_state(r)?;
        (dir < 4 && m.turn < 3).then_some(m)
    }

    fn contents(&self) -> Vec<Stack> {
        if self.held == ItemId::NONE {
            Vec::new()
        } else {
            vec![Stack { item: self.held, count: self.held_n }]
        }
    }

    fn describe(&self, _: &Factory) -> String {
        let count = |links: &[Option<u32>; 3]| links.iter().filter(|o| o.is_some()).count();
        match self.kind {
            RouterKind::Filter => format!(
                "{}\n{} of 3 outputs connected (front, left, right)\n\tRMB=Configure filter",
                self.status_text(),
                count(&self.outs)
            ),
            RouterKind::Splitter => format!(
                "Splits items evenly between the belts leading away\n{} of 3 outputs connected (front, left, right)",
                count(&self.outs)
            ),
            RouterKind::Merger => {
                let out = if self.outs[0].is_some() { "connected" } else { "no belt leading away" };
                format!(
                    "Takes from the belts coming in, in turn, and puts everything out in front\n{} of 3 inputs \
                     connected (behind, left, right) · output in front: {out}",
                    count(&self.ins)
                )
            }
        }
    }

    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        super::router_model::draw(self, out, rel, time);
    }
}

/// Each router's inputs from the belts' links `outs` (`relink`): the belts that end in a merger, by the side they come
/// from (the first belt of each side); none for a splitter or filter, which belts push into.
pub(super) fn merger_ins(routers: &[Router], belts: &[Belt], outs: &[Link]) -> Vec<[Option<u32>; 3]> {
    let mut ins = vec![[None; 3]; routers.len()];
    for (j, link) in outs.iter().enumerate() {
        if let Link::Machine(Slot::Router(i)) = *link {
            if let Some(slot) = routers[i as usize].in_slot(belts[j].dir) {
                ins[i as usize][slot].get_or_insert(j as u32);
            }
        }
    }
    ins
}

#[cfg(test)]
mod tests;
