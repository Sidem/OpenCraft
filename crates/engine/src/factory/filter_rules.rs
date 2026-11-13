//! What a filter sends where: each of its three outputs (straight on, left, right, as `Router::out_dirs` orders them)
//! has a list of picked items, and at least one output takes everything else (`rest`, one bit an output). An item is
//! picked for at most one output; a picked item goes only that way, anything else only to the `rest` outputs (round
//! robin when more than one, as filters before save version 72 sent everything but their item to both sides).
//!
//! Changed only through `Action::FilterPick` and `Action::FilterRest` (`Factory::filter_pick`, `filter_rest`).
//! Invariants: `rest` is never 0; no item is in two lists; no list holds `NONE`, a repeat or more than `MAX_PICKS`.

use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{self, ItemId};

/// The outputs, in `Router::out_dirs` order.
pub const STRAIGHT: u8 = 0;
pub const LEFT: u8 = 1;
pub const RIGHT: u8 = 2;
/// `FilterPick`'s "out" that sends an item nowhere in particular: off every list, so it goes with everything else.
pub const UNPICKED: u8 = 3;
/// The most items one output can list.
pub const MAX_PICKS: usize = 8;
const OUTPUTS: usize = 3;
const NAMES: [&str; OUTPUTS] = ["Straight on", "Left", "Right"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterRules {
    /// The items picked for each output: straight, left, right.
    pub picks: [Vec<ItemId>; OUTPUTS],
    /// The outputs that take everything not picked: bit `1 << out`.
    pub rest: u8,
}

impl Default for FilterRules {
    /// A new filter passes everything straight on until something is picked for a side.
    fn default() -> Self {
        FilterRules { picks: Default::default(), rest: 1 << STRAIGHT }
    }
}

impl FilterRules {
    /// The rules of a filter from before save version 72: its one item straight on, everything else to the sides.
    pub fn legacy(item: ItemId) -> Self {
        let mut rules = FilterRules { picks: Default::default(), rest: 1 << LEFT | 1 << RIGHT };
        if item != ItemId::NONE {
            rules.picks[STRAIGHT as usize].push(item);
        }
        rules
    }

    /// Whether `item` may leave by output `out`.
    pub fn allows(&self, item: ItemId, out: usize) -> bool {
        match self.picks.iter().position(|list| list.contains(&item)) {
            Some(o) => o == out,
            None => self.rest & 1 << out != 0,
        }
    }

    /// Sends `item` out `out` (off any other list), or off every list with `UNPICKED`. Refused (false) for an unknown
    /// item, an output past the three, or a full list.
    pub fn pick(&mut self, item: ItemId, out: u8) -> bool {
        if item == ItemId::NONE || !item.is_valid() || out > UNPICKED {
            return false;
        }
        if out != UNPICKED && !self.picks[out as usize].contains(&item) && self.picks[out as usize].len() >= MAX_PICKS {
            return false;
        }
        for list in &mut self.picks {
            list.retain(|&i| i != item);
        }
        if out != UNPICKED {
            self.picks[out as usize].push(item);
        }
        true
    }

    /// Makes output `out` the one that takes everything else (alone); what was picked for it goes with everything else,
    /// that is still that way.
    pub fn set_rest(&mut self, out: u8) -> bool {
        if out as usize >= OUTPUTS {
            return false;
        }
        self.rest = 1 << out;
        self.picks[out as usize].clear();
        true
    }

    /// One line an output: where each picked item goes and which way everything else does.
    pub fn describe(&self) -> String {
        let rest = (0..OUTPUTS).filter(|&o| self.rest & 1 << o != 0).count();
        let line = |o: usize| {
            let picked: Vec<&str> = self.picks[o].iter().map(|&i| item::name(i)).collect();
            let what = match (self.rest & 1 << o != 0, picked.is_empty()) {
                (true, _) if rest > 1 => "everything else (shared)".to_string(),
                (true, _) => "everything else".to_string(),
                (false, true) => "nothing".to_string(),
                (false, false) => picked.join(", "),
            };
            format!("{}: {what}", NAMES[o])
        };
        [LEFT as usize, STRAIGHT as usize, RIGHT as usize].map(line).join("\n")
    }

    /// For the panel (`api/machine.rs`): `rest`, then each output's count and items, straight, left, right.
    pub fn to_ints(&self) -> Vec<i32> {
        let mut out = vec![i32::from(self.rest)];
        for list in &self.picks {
            out.push(list.len() as i32);
            out.extend(list.iter().map(|i| i32::from(i.0)));
        }
        out
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u8(self.rest);
        for list in &self.picks {
            w.u8(list.len() as u8);
            for &i in list {
                w.item(i);
            }
        }
    }

    /// Reads what `write_state` wrote; `None` when it breaks an invariant.
    pub fn read_state(r: &mut ByteReader) -> Option<Self> {
        let mut rules = FilterRules { picks: Default::default(), rest: r.u8()? };
        if rules.rest == 0 || rules.rest >= 1 << OUTPUTS {
            return None;
        }
        for o in 0..OUTPUTS {
            let n = r.u8()? as usize;
            if n > MAX_PICKS {
                return None;
            }
            for _ in 0..n {
                let i = r.item()?;
                if i == ItemId::NONE || rules.picks.iter().any(|l| l.contains(&i)) {
                    return None;
                }
                rules.picks[o].push(i);
            }
        }
        Some(rules)
    }
}

#[cfg(test)]
mod tests;
