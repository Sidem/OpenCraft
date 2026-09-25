//! Readout text for a targeted machine (each kind writes its own in `Machine::describe`), plus the
//! integer formatting helpers the HUD shares. Read-only queries.
//! Integers only: formatting floats pulls ~25 KB of float printing into the wasm.

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

impl Factory {
    /// Detail lines for the target readout; `None` when there is no machine or nothing to say.
    pub fn describe(&self, pos: IVec3) -> Option<String> {
        let text = match *self.at.get(&pos)? {
            Slot::Belt(i) => self.belts[i as usize].describe(self),
            Slot::Miner(i) => self.miners[i as usize].describe(self),
            Slot::Storage(i) => self.storages[i as usize].describe(self),
            Slot::Smelter(i) => self.smelters[i as usize].describe(self),
            Slot::Constructor(i) => self.constructors[i as usize].describe(self),
        };
        Some(text).filter(|t| !t.is_empty())
    }
}
