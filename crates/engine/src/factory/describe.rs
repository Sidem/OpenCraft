//! Readout text for a targeted machine (each kind writes its own in `Machine::describe`), plus the
//! integer formatting helpers the HUD shares. Read-only queries.
//! Integers only: formatting floats pulls ~25 KB of float printing into the wasm.

use crate::item;
use crate::math::IVec3;

use super::links::Slot;
use super::{Factory, Machine};

/// `1234567` → `"1,234,567"`.
pub fn fmt_int(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A rough human duration: `"42 s"`, `"2 min"`, `"3 h 20 min"`, `"1 d 1 h"`.
pub fn fmt_duration(seconds: f64) -> String {
    let s = seconds.max(0.0).round() as u64;
    match s {
        0..=59 => format!("{s} s"),
        60..=3599 => format!("{} min", s / 60),
        3600..=86_399 => format!("{} h {} min", s / 3600, s % 3600 / 60),
        _ => format!("{} d {} h", s / 86_400, s % 86_400 / 3600),
    }
}

/// `1` → `"1st"`, `2` → `"2nd"`, `3` → `"3rd"`, `4` → `"4th"`.
pub fn ordinal(n: u8) -> String {
    let suffix = match n % 100 {
        11..=13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{n}{suffix}")
}

/// Encodes structured HUD control chips (`\tKEY=Label\t...`) for `#target-controls` (`web/src/ui/target-controls.ts`).
/// A line starting with `\t` is rendered as keycap/mouse chips below the status text, never mixed into prose.
pub fn hud_keys(items: &[(&str, &str)]) -> String {
    let mut out = String::new();
    for &(key, label) in items {
        out.push('\t');
        out.push_str(key);
        out.push('=');
        out.push_str(label);
    }
    out
}

/// Visual status pill row (`tone`: `ok`, `warn`, `err`, `off`, `info`) for `web/src/ui/target-card.ts`.
pub fn hud_status(tone: &str, text: &str) -> String {
    format!("@status|{tone}|{text}")
}

/// Visual horizontal meter row (`tone`: `ok`, `warn`, `err`, `cyan`, `accent`).
pub fn hud_bar(label: &str, value: u64, max: u64, right: &str, tone: &str) -> String {
    format!("@bar|{label}|{value}|{max}|{right}|{tone}")
}

/// Visual mini-slot inventory strip (`group`: `"In"`, `"Out"`, `"Fuel"`, `"Side"`, `"Holding"`, etc.).
pub fn hud_items(group: &str, items: &[(item::ItemId, u32)]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let list: Vec<String> = items.iter().map(|&(i, n)| format!("{}:{n}", i.0)).collect();
    Some(format!("@items|{group}|{}", list.join(",")))
}

/// Visual dot-pip meter (`used` lit pips out of `total`).
pub fn hud_pips(label: &str, used: usize, total: usize) -> String {
    format!("@pips|{label}|{used}|{total}")
}

/// Compact horizontal stat chips (`@stats|chip1|chip2|...`).
pub fn hud_stats(chips: &[String]) -> Option<String> {
    (!chips.is_empty()).then(|| format!("@stats|{}", chips.join("|")))
}

impl Factory {
    /// What the processing machine at `pos` is stuck on: the inputs it lacks (`Processor::lacking`) and the output it
    /// has nowhere to put. Empty for anything else, so the HUD can say where an item comes from or what it is for.
    pub fn stuck_on(&self, pos: IVec3) -> (Vec<item::ItemId>, Option<item::ItemId>) {
        let anchor = self.anchor_of(pos).unwrap_or(pos);
        match self.at.get(&anchor) {
            Some(&Slot::Process(i)) => {
                let p = &self.processors[i as usize];
                (p.lacking().into_iter().map(|(item, _)| item).collect(), p.blocked_output())
            }
            _ => (Vec::new(), None),
        }
    }

    /// Detail lines for the target readout; `None` when there is no machine or nothing to say.
    pub fn describe(&self, pos: IVec3) -> Option<String> {
        let text = match *self.at.get(&pos)? {
            Slot::Belt(i) => self.belts[i as usize].describe(self),
            Slot::Miner(i) => self.miners[i as usize].describe(self),
            Slot::Storage(i) => self.storages[i as usize].describe(self),
            Slot::Process(i) => {
                let text = self.processors[i as usize].describe(self);
                self.compute_line(i as usize)
                    .or_else(|| self.beam_line(i as usize))
                    .map_or(text.clone(), |l| format!("{text}\n{l}"))
            }
            Slot::Router(i) => self.routers[i as usize].describe(self),
            Slot::Generator(i) => self.generators[i as usize].describe(self),
            Slot::Pole(i) => self.poles[i as usize].describe(self),
            Slot::Lab(i) => self.labs[i as usize].describe(self),
            Slot::Pipe(i) => self.pipework[i as usize].describe(self),
            Slot::Quarry(i) => self.quarries[i as usize].describe(self),
            Slot::Sensor(i) => self.sensors[i as usize].describe(self),
            Slot::Rail(i) => self.rails[i as usize].describe(self),
            Slot::Node(i) => self.nodes[i as usize].describe(self),
        };
        // A machine a sensor switched off is wired; say so instead of 'no power'.
        let text = if self.anchor_of(pos).is_some_and(|a| self.power.off.contains(&a)) {
            text.replace(super::power::NOT_WIRED, super::sensor::SWITCHED_OFF)
                .replace("@status|err|No power", "@status|off|Switched off by sensor")
        } else {
            text
        };
        // A tiered machine emits a header badge (`@tier|mk|item_id|kit_id|kits`) instead of duplicating Mk in prose.
        let text = match (self.tiered_at(pos), self.next_upgrade(pos)) {
            (Some((block, t)), next) => {
                let id = super::tiers::family(block).and_then(|f| f.items.get(t as usize)).map_or(0, |i| i.0);
                let (kit, kits) = next.map_or((0, 0), |s| (s.kit.0, s.kits));
                format!("@tier|{}|{id}|{kit}|{kits}\n{text}", t + 1)
            }
            _ => text,
        };
        Some(text).filter(|t| !t.is_empty())
    }
}
