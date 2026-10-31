//! What a map mark is, in words: the info box when the player hovers a mark on the world map or the corner minimap
//! (`web/src/ui/map-info.ts`). A query, never core state: it walks the same marks the maps draw (`Minimap::each_mark`),
//! so it can only say what the player has learned (a prospected or surveyed deposit by ore and tier, never its reserve;
//! ore seen at the surface; their own machines; towns and sites on the map).
//!
//! The answer is up to `MAX_DESCRIBED` lines, nearest first, `shape<TAB>colour<TAB>title<TAB>detail` (the first two as
//! the mark's records `marks.rs`), so the host can skip the marks its filters hide. To explain a new mark: add its
//! `Subject` variant's words to `Subject::words`; a test fails for a shape constant with none.

use crate::block;
use crate::factory::{self, Factory};
use crate::satellites::{self, COMMS, POWER, SCIENCE, SURVEY};
use crate::settlements::{Town, DORMANT, KINDS as TOWN_KINDS, STALLED};

use super::marks::Subject;
use super::Minimap;
use crate::deposits::Tier;

/// The most marks one answer names.
pub const MAX_DESCRIBED: usize = 8;
/// The query's widest reach in blocks.
const MAX_REACH: i32 = 256;
/// What a terraforming site's job does, by its saved byte (`factory/sites.rs`).
const JOBS: [(&str, &str); 4] = [
    ("Dig", "Drones cut the ground above its level away."),
    ("Fill", "Drones build the ground up to its level."),
    ("Flatten", "Drones cut what is above its level and fill what is below."),
    ("Tunnel", "Drones bore a tunnel through the ground."),
];

impl Minimap {
    /// The marks within `reach` blocks of column `at` (x, z), nearest first, one line each (see the module header).
    /// A satellite's footprint counts by the distance to its ring, the only part drawn. `ranges` as in `marks`.
    pub fn describe(&self, factory: &Factory, towns: &[Town], at: (i32, i32), reach: i32, ranges: bool) -> String {
        let reach = reach.clamp(1, MAX_REACH);
        let (lo, hi, limit) = ((at.0 - reach, at.1 - reach), (at.0 + reach, at.1 + reach), f64::from(reach));
        let mut found: Vec<(f64, Subject)> = Vec::with_capacity(MAX_DESCRIBED + 1);
        self.each_mark(factory, towns, lo, hi, ranges, &mut |x, z, s| {
            let d = f64::from(x - at.0).hypot(f64::from(z - at.1));
            let d = if let Subject::Cover { reach, .. } = s { (d - f64::from(reach)).abs() } else { d };
            if d > limit || (found.len() == MAX_DESCRIBED && found[MAX_DESCRIBED - 1].0 <= d) {
                return;
            }
            let place = found.iter().position(|f| f.0 > d).unwrap_or(found.len());
            found.insert(place, (d, s));
            found.truncate(MAX_DESCRIBED);
        });
        let mut out = String::new();
        for (_, s) in found {
            let (title, detail) = s.words();
            out.push_str(&format!("{}\t{}\t{title}\t{detail}\n", s.shape(), s.colour()));
        }
        out
    }
}

impl Subject {
    /// A title and one sentence on what the mark is.
    pub fn words(self) -> (String, String) {
        let ore_name = |ore| block::def(ore).name;
        match self {
            Subject::Deposit { ore, tier, dry: true } => (
                format!("{} {}: worked out", ore_name(ore), tier.name()),
                "No ore is left in it. Miners on it have stopped: move them to another deposit.".into(),
            ),
            Subject::Deposit { ore, tier, dry: false } => (
                format!("{} {}", ore_name(ore), tier.name()),
                match tier {
                    Tier::Lode => "The richest kind of deposit. A miner placed on it draws ore until it runs dry.",
                    Tier::Vein => "A deposit you know of. A miner placed on it draws ore until it runs dry.",
                    Tier::Outcrop => "A small deposit at the surface.",
                }
                .into(),
            ),
            Subject::Ore(ore) => (
                format!("{} at the surface", ore_name(ore)),
                "Ore you have seen on the ground. A scanner shows what lies under it.".into(),
            ),
            Subject::Guess { ore, strength } => (
                format!("Possibly {}", ore_name(ore)),
                format!(
                    "{} guess from the stained soil seen so far. Go there with a scanner to find out.",
                    ["A weak", "A fair", "A strong"][usize::from(strength.clamp(1, 3)) - 1]
                ),
            ),
            Subject::Site { job } => {
                let (name, what) = JOBS.get(usize::from(job)).copied().unwrap_or(("Terraforming", ""));
                (format!("{name} site"), what.into())
            }
            Subject::Machine { block, tier, dry } => {
                let name = match factory::tiers::family(block) {
                    Some(_) => factory::tiers::name(block, tier),
                    None => block::def(block).name.into(),
                };
                match dry {
                    true => (
                        format!("{name}: stopped"),
                        "Its deposit is worked out. Break it and place it on another deposit.".into(),
                    ),
                    false => (name, "One of your machines.".into()),
                }
            }
            Subject::Launch => (block::def(block::LAUNCH_SITE).name.into(), "Rockets lift off from here.".into()),
            Subject::Pod { launcher, channel } => {
                let colour = factory::channel_name(channel);
                let (block, what) = if launcher {
                    (block::POD_LAUNCHER, format!("Sends drop pods to a landing pad on the {colour} channel."))
                } else {
                    (block::LANDING_PAD, format!("Takes drop pods sent on the {colour} channel."))
                };
                (format!("{} ({colour})", block::def(block).name), what)
            }
            Subject::Cover { kind, reach } => {
                let what = match kind {
                    SURVEY => "Deposits inside it show on the map.",
                    COMMS => "Data networks and drone ports inside it link up.",
                    POWER => "A rectenna inside it can receive power from it.",
                    SCIENCE => "A ground station inside it can receive compute.",
                    _ => "",
                };
                let name = satellites::KINDS.get(usize::from(kind)).map_or("A", |k| k.name);
                (format!("{name} satellite cover"), format!("Reaches {reach} blocks around its centre. {what}"))
            }
            Subject::Range { reach } => (
                "Long-distance pole".into(),
                format!("The next long pole can stand within {reach} blocks of this one and still link to it."),
            ),
            Subject::Town { kind, state } => (
                TOWN_KINDS[usize::from(kind)].name.into(),
                match state {
                    DORMANT => "A town whose depot is broken: it does nothing until a depot stands on its spot again.",
                    STALLED => "A town short of upkeep: its orders and perk are paused until you supply it.",
                    _ => "A town you are supplying: it grows as you deliver its orders.",
                }
                .into(),
            ),
        }
    }
}

#[cfg(test)]
mod tests;
