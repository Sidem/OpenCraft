//! Readout text for a targeted machine (belt load and link, box contents, miner status and
//! deposit life), plus the integer formatting helpers the HUD shares. Read-only queries.
//! Integers only: formatting floats pulls ~25 KB of float printing into the wasm.

use crate::block::{self, BlockId};
use crate::math::IVec3;

use super::storage::STORAGE_SLOTS;
use super::{Factory, Link, MinerStatus, Slot, MINER_RECOVERY};

const DIR_NAMES: [&str; 4] = ["north", "east", "south", "west"];

/// `1234567` → `"1,234,567"`.
pub fn fmt_int(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
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
    /// Detail lines for the target readout.
    pub fn describe(&self, pos: IVec3) -> Option<String> {
        match *self.at.get(&pos)? {
            Slot::Belt(i) => {
                let b = &self.belts[i as usize];
                let load = match b.items.len() {
                    0 => "Empty".to_string(),
                    1 => "Carrying 1 item".to_string(),
                    n => format!("Carrying {n} items"),
                };
                let end = if self.dirty {
                    ""
                } else {
                    match b.out {
                        Link::None => " · nothing in front, items wait at the end",
                        Link::Belt { mid: true, .. } => " · joins the next belt from the side",
                        Link::Belt { .. } => "",
                        Link::Storage(_) => " · delivers into a box",
                    }
                };
                Some(format!("{load} · heading {}{end}", DIR_NAMES[b.dir as usize]))
            }
            Slot::Storage(i) => {
                let s = &self.storages[i as usize];
                let used = s.slots.iter().filter(|st| !st.is_empty()).count();
                let mut totals: Vec<(BlockId, u32)> = Vec::new();
                for st in s.slots.iter().filter(|st| !st.is_empty()) {
                    match totals.iter_mut().find(|(item, _)| *item == st.item) {
                        Some((_, n)) => *n += st.count,
                        None => totals.push((st.item, st.count)),
                    }
                }
                // Largest first. At most 24 entries: an insertion sort, rather than pulling a
                // stable-sort instantiation (~9 KB) into the wasm for a readout.
                for i in 1..totals.len() {
                    let mut j = i;
                    while j > 0 && totals[j - 1].1 < totals[j].1 {
                        totals.swap(j - 1, j);
                        j -= 1;
                    }
                }
                let mut lines = vec![format!("{used} of {STORAGE_SLOTS} slots used")];
                if !totals.is_empty() {
                    let list: Vec<String> = totals
                        .iter()
                        .take(3)
                        .map(|(item, n)| format!("{} {}", fmt_int(*n as u64), block::def(*item).name))
                        .collect();
                    lines.push(list.join(", ") + if totals.len() > 3 { ", ..." } else { "" });
                    lines.push("Right-click to take everything".to_string());
                }
                Some(lines.join("\n"))
            }
            Slot::Miner(i) => {
                let m = &self.miners[i as usize];
                let Some(key) = m.deposit else {
                    return Some("Not on an ore deposit. Place miners against an ore block.".to_string());
                };
                let st = self.deposits.get(&key)?;
                let mut lines = vec![format!(
                    "{} · {} of {} blocks left",
                    st.deposit.name(),
                    fmt_int(st.remaining_blocks as u64),
                    fmt_int(st.initial_blocks as u64)
                )];
                lines.push(match m.status {
                    MinerStatus::Running if m.draw_rate > 0.01 => format!(
                        "Running · {} ore/min ({}% recovery)",
                        (m.draw_rate * MINER_RECOVERY * 60.0).round() as u32,
                        (MINER_RECOVERY * 100.0).round() as u32
                    ),
                    MinerStatus::Running => "Waiting: other miners are using this deposit's full draw".to_string(),
                    MinerStatus::OutputFull => "Output full: put a belt leading away, or a box, next to it".to_string(),
                    MinerStatus::Exhausted => "Deposit worked out".to_string(),
                    MinerStatus::NoDeposit => String::new(),
                });
                if m.held > 0 {
                    lines.push(format!("Holding {} {} · right-click to take", m.held, block::def(m.ore).name));
                }
                let total_draw: f64 = self.miners.iter().filter(|o| o.deposit == Some(key)).map(|o| o.draw_rate).sum();
                if total_draw > 0.01 && !st.exhausted() {
                    lines.push(format!(
                        "{} units left · about {} at the current draw",
                        fmt_int(st.remaining_units() as u64),
                        fmt_duration(st.remaining_units() / total_draw)
                    ));
                }
                Some(lines.join("\n"))
            }
        }
    }
}
