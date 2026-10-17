//! What a ruin's logbook says (step R5b): a pure function of the seed and the ruin's spot, so every player of a world
//! reads the same page, and nothing is saved. Three kinds of page: a bearing to richer ground (an ore the crew drew
//! out here, marked for a map pin), a tip that names real recipes ahead of time, and, from the far ruins, a fragment of
//! the story (the tower to the sky, the late tech). Near ruins (within `FAR` blocks of spawn) hold only the first two,
//! and only ores of the early game; far ones point at the ore that lies far from spawn.
//! To add a tip or fragment: a line in `TIPS` or `FRAGMENTS`. Text avoids float formatting (wasm size).

use super::WorldGen;
use crate::block::{
    self, BlockId, BAUXITE_ORE, COAL_ORE, COPPER_ORE, IRON_ORE, LIMESTONE, OIL_SAND, QUARTZ_ORE, URANIUM_ORE,
};
use crate::math::{hash2, IVec3};

/// Ruins this far from spawn (blocks) or further hold the late pages.
const FAR: i64 = 700;
/// A bearing nearer than this is not worth a page: the ground round the ruin already holds the ore (the common ores
/// lie under every biome, so near pages mostly point at quartz).
const MIN_BEARING: i32 = 48;
const NEAR_ORES: [BlockId; 5] = [IRON_ORE, COPPER_ORE, COAL_ORE, LIMESTONE, QUARTZ_ORE];
const FAR_ORES: [BlockId; 4] = [BAUXITE_ORE, URANIUM_ORE, OIL_SAND, IRON_ORE];

const TIPS: [&str; 6] = [
    "Stained soil lies over ore: rusty over iron, dark over coal, green over copper, pale over limestone and quartz. Flowers grow over each stain, too. Dig under them.",
    "Mining ore by hand keeps 3 per block and the rest of its deposit is lost. A miner recovers 60% of what it drills: build one early.",
    "A smelter melts scrap metal: 3 scrap make an iron ingot. Whatever rusted here is worth carrying home.",
    "Concrete: an assembler mixes 1 quicklime, 2 sand and 2 stone into 4 concrete. Quicklime comes from smelting limestone.",
    "A motor is 1 iron rod, 2 gears and 4 copper wire, put together in an assembler. A constructor presses an iron plate into a gear.",
    "Stone bricks: smelt 2 stone. Glass: smelt sand, or quartz ore for twice as much.",
];

const FRAGMENTS: [&str; 4] = [
    "The tower was to reach the sky: a cable from the ground to a station above, and everything we built was for it. We never found enough of the right ground to finish it.",
    "Seam after seam went to spent rock. The foreman kept saying the ground gives only once. We should have built where the ore was, and carried less.",
    "The reactor ran on enriched fuel cells and the satellites watched the ore from orbit. It was not enough to find what the ground no longer held.",
    "Whoever reads this: ground nobody has mined is the only thing worth marking. Mark it, and leave the old shafts alone.",
];

/// A page: a title, its text, and ground to pin on the map (the ore the page points at).
pub struct Entry {
    pub title: String,
    pub text: String,
    pub pin: Option<(i32, i32, BlockId)>,
}

impl WorldGen {
    /// The page of the logbook at `pos`, if `pos` is a ruin's logbook.
    pub fn logbook_at(&self, pos: IVec3) -> Option<Entry> {
        self.ruins_around(pos.x, pos.z).find(|r| r.logbook == pos).map(|r| self.page(r.x, r.z))
    }

    /// Where the logbook of the ruin nearest the spawn stands (tests).
    #[cfg(test)]
    pub(crate) fn first_logbook(&self) -> Option<IVec3> {
        self.ruin_at(0, 0).map(|r| r.logbook)
    }

    /// The page of a ruin centred on (x, z).
    pub(super) fn page(&self, x: i32, z: i32) -> Entry {
        let h = hash2(self.seed ^ 0x106B, x, z);
        let far = i64::from(x).pow(2) + i64::from(z).pow(2) >= FAR * FAR;
        let title = format!("Crew log, day {}", 40 + h % 600);
        let roll = (h >> 8) % 4;
        let (text, pin) = if far && roll >= 2 {
            (FRAGMENTS[(h >> 12) as usize % FRAGMENTS.len()].to_string(), None)
        } else if roll < 2 || far {
            self.bearing_page(x, z, h, if far { &FAR_ORES } else { &NEAR_ORES })
                .map_or_else(|| (TIPS[(h >> 12) as usize % TIPS.len()].to_string(), None), |(t, p)| (t, Some(p)))
        } else {
            (TIPS[(h >> 12) as usize % TIPS.len()].to_string(), None)
        };
        Entry { title, text, pin }
    }

    /// A page pointing at the nearest ground that holds one of `ores` (the first, from a hashed start, that lies far
    /// enough off).
    fn bearing_page(&self, x: i32, z: i32, h: u32, ores: &[BlockId]) -> Option<(String, (i32, i32, BlockId))> {
        (0..ores.len()).find_map(|k| {
            let ore = ores[((h >> 4) as usize + k) % ores.len()];
            let b = self.bearing_to(ore, x, z).filter(|b| b.distance >= MIN_BEARING)?;
            let (dx, dz) = (b.x - x, b.z - z);
            let text = format!(
                "{} ran thin on this ground. The nearest ground that holds it lies about {} blocks {} of here, where the crew once marked it. Pin it on your map and go.",
                block::def(ore).name,
                (b.distance + 5) / 10 * 10,
                compass(dx, dz),
            );
            Some((text, (b.x, b.z, ore)))
        })
    }
}

/// The compass point of (dx, dz) in words; north is -z.
fn compass(dx: i32, dz: i32) -> &'static str {
    let (ax, az) = (dx.abs() * 100, dz.abs() * 100);
    let (ew, ns) = (dx > 0, dz < 0);
    if az * 100 < 41 * ax {
        if ew {
            "east"
        } else {
            "west"
        }
    } else if ax * 100 < 41 * az {
        if ns {
            "north"
        } else {
            "south"
        }
    } else {
        match (ns, ew) {
            (true, true) => "north-east",
            (true, false) => "north-west",
            (false, true) => "south-east",
            (false, false) => "south-west",
        }
    }
}

#[cfg(test)]
mod tests;
