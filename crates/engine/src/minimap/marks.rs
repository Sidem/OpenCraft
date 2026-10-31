//! Map marks: deposits the local player has prospected, ore seen at the surface (the explored map's
//! spots, `atlas.rs`) and the factory's machines, as flat records for the host to draw over the map
//! images (`web/src/ui/minimap.ts`, `web/src/ui/worldmap.ts`).
//!
//! Presentation only. The prospected deposits (`Known`) are the local player's memory, not core state:
//! the scanner and core drill add veins and lodes (`remember`; outcrops show on the map by themselves),
//! and the host keeps the list with the world's record in the browser (`export` / `import`), not in the
//! save file. A deposit that runs dry keeps a mark of its own (`MARK_DRY`), and so does a miner that worked it
//! out, so a dead outpost stands out; deposits under the player's own miners are marked even if never scanned. Minimap records
//! are relative to the column the image was last drawn around, so marks line up with its pixels; world
//! map records are in world columns.
//!
//! To mark another machine kind: a line in `Factory::map_machines` and a colour in `machine_color`
//! (processors: `map_colour` in their spec). Every mark is a `Subject`; a new shape constant needs a variant
//! and its words in `describe.rs` (a test fails without them).

use rustc_hash::FxHashSet;

use crate::block::{self, BlockId};
use crate::chunk::CHUNK_SHIFT;
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::factory::{self, Factory};
use crate::math::IVec3;
use crate::settlements::{Town, KINDS};
use crate::worldgen::WorldGen;

use super::{Minimap, HALF};
use crate::site_hands::SITE_COLOURS;

/// Numbers per mark: dx, dz (blocks from the image's centre column), colour (0xRRGGBB), shape.
pub const MARK_FIELDS: usize = 4;
/// Mark shapes: a deposit (round), a machine (square), ore seen at the surface (a diamond), a terraforming site
/// (a hollow square at its centre, in its job colour).
pub const MARK_DEPOSIT: i32 = 0;
pub const MARK_MACHINE: i32 = 1;
pub const MARK_ORE: i32 = 2;
pub const MARK_SITE: i32 = 3;
/// An AI survey guess (`survey.rs`): this plus its strength (0..3) as the shape, in the ore's colour.
pub const MARK_GUESS: i32 = 4;
/// A satellite's footprint (`satellites.rs`): this plus its kind as the shape, and the third number is its reach in
/// blocks instead of a colour.
pub const MARK_COVER: i32 = 8;
/// A long-distance pole's reach while one is held (`power_tools/long_pole.rs`): the third number is the reach in blocks.
pub const MARK_RANGE: i32 = 12;
/// A landing pad (a square); this plus 1 is a pod launcher (a square with an arrow up). Drawn in its channel's colour.
pub const MARK_CHANNEL: i32 = 16;
/// A launch site (a rocket shape), in its colour.
pub const MARK_LAUNCH: i32 = 20;
/// A town (a house at its depot), in its kind's colour (`KINDS`): this plus its state as the shape (`TOWN_STATES`:
/// growing or supplied, stalled, dormant).
pub const MARK_TOWN: i32 = 24;
/// A deposit with no ore left (a crossed-out ring in the ore's colour).
pub const MARK_DRY: i32 = 28;
/// A miner whose deposit is worked out, so it has stopped (a machine square with a red cross).
pub const MARK_DRY_MINER: i32 = 29;
/// Numbers per deposit in `export` / `import`: tier, chunk column x and z, ore, index (a `DepositKey`).
const KNOWN_FIELDS: usize = 5;
/// The most deposits remembered; the oldest is forgotten first.
const MAX_KNOWN: usize = 1024;

/// Deposits the local player has prospected, oldest first.
#[derive(Default)]
pub struct Known {
    deposits: Vec<Deposit>,
}

impl Known {
    /// Remembers a prospected vein or lode (outcrops are skipped).
    pub fn remember(&mut self, d: &Deposit) {
        if d.tier() == Tier::Outcrop || self.deposits.iter().any(|k| k.key == d.key) {
            return;
        }
        if self.deposits.len() == MAX_KNOWN {
            self.deposits.remove(0);
        }
        self.deposits.push(*d);
    }

    fn has(&self, key: &DepositKey) -> bool {
        self.deposits.iter().any(|d| d.key == *key)
    }

    /// Where the remembered deposits lie and their ore: x, z, block.
    pub fn centres(&self) -> impl Iterator<Item = (i32, i32, BlockId)> + '_ {
        self.deposits.iter().map(|d| (d.center.x, d.center.z, d.ore()))
    }

    /// The remembered deposits, worked out or not, `KNOWN_FIELDS` numbers each.
    pub fn export(&self) -> Vec<i32> {
        let mut out = Vec::new();
        for d in &self.deposits {
            let k = d.key;
            out.extend_from_slice(&[k.tier as i32, k.cx, k.cz, k.ore as i32, k.index as i32]);
        }
        out
    }

    /// Replaces the list with `export`'s numbers, skipping any that name no deposit of this world.
    pub fn import(&mut self, data: &[i32], generator: &WorldGen) {
        self.deposits.clear();
        for r in data.chunks_exact(KNOWN_FIELDS) {
            let Some(tier) = u8::try_from(r[0]).ok().and_then(Tier::from_u8) else { continue };
            let (Ok(ore), Ok(index)) = (BlockId::try_from(r[3]), u16::try_from(r[4])) else { continue };
            let key = DepositKey { tier, cx: r[1], cz: r[2], ore, index };
            if let Some(d) = generator.deposit_by_key(key) {
                self.remember(&d);
            }
        }
    }
}

/// What a mark stands for. Every mark is built from one, so what the maps draw (`shape`, `colour`) and what hovering
/// says (`describe.rs`) cannot differ. Add a variant with its shape constant, and `describe.rs` must explain it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Subject {
    /// A deposit the player knows of; `dry` once no ore is left in it.
    Deposit {
        ore: BlockId,
        tier: Tier,
        dry: bool,
    },
    Ore(BlockId),
    /// An AI survey guess; `strength` is 1 to 3.
    Guess {
        ore: BlockId,
        strength: u8,
    },
    /// A terraforming site of this job (`Job as u8`).
    Site {
        job: u8,
    },
    /// One of the player's machines, at its tier (0 for a machine without tiers).
    Machine {
        block: BlockId,
        tier: u8,
        /// A miner that has worked out its deposit.
        dry: bool,
    },
    Launch,
    Pod {
        launcher: bool,
        channel: u8,
    },
    /// A satellite footprint of this kind (`satellites::KINDS`) reaching this far.
    Cover {
        kind: u8,
        reach: i32,
    },
    Range {
        reach: i32,
    },
    /// A town of this kind (`KINDS`) in this state (`TOWN_STATES`).
    Town {
        kind: u8,
        state: u8,
    },
}

impl Subject {
    /// The shape the host draws (`MARK_*`).
    pub fn shape(self) -> i32 {
        match self {
            Subject::Deposit { dry: false, .. } => MARK_DEPOSIT,
            Subject::Deposit { dry: true, .. } => MARK_DRY,
            Subject::Ore(_) => MARK_ORE,
            Subject::Guess { strength, .. } => MARK_GUESS + i32::from(strength) - 1,
            Subject::Site { .. } => MARK_SITE,
            Subject::Machine { dry: false, .. } => MARK_MACHINE,
            Subject::Machine { dry: true, .. } => MARK_DRY_MINER,
            Subject::Launch => MARK_LAUNCH,
            Subject::Pod { launcher, .. } => MARK_CHANNEL + i32::from(launcher),
            Subject::Cover { kind, .. } => MARK_COVER + i32::from(kind),
            Subject::Range { .. } => MARK_RANGE,
            Subject::Town { state, .. } => MARK_TOWN + i32::from(state),
        }
    }

    /// The third number of the mark: its colour (0xRRGGBB), or the reach in blocks for a footprint or a pole's ring.
    pub fn colour(self) -> i32 {
        match self {
            Subject::Deposit { ore, .. } | Subject::Ore(ore) | Subject::Guess { ore, .. } => ore_color(ore),
            Subject::Site { job } => SITE_COLOURS[usize::from(job)],
            Subject::Machine { block, .. } => machine_color(block),
            Subject::Launch => machine_color(block::LAUNCH_SITE),
            Subject::Pod { channel, .. } => factory::channel_colour(channel) as i32,
            Subject::Cover { reach, .. } | Subject::Range { reach } => reach,
            Subject::Town { kind, .. } => KINDS[usize::from(kind)].colour,
        }
    }
}

impl Minimap {
    /// Marks inside the minimap image (`MARK_FIELDS` numbers each), relative to its centre column.
    /// Empty before the first redraw.
    pub fn marks(&self, factory: &Factory, towns: &[Town], ranges: bool) -> Vec<i32> {
        let Some((cx, cz)) = self.centre else { return Vec::new() };
        self.marks_in(factory, towns, (cx - HALF, cz - HALF), (cx + HALF - 1, cz + HALF - 1), (cx, cz), ranges)
    }

    /// The flat records of `each_mark`, relative to column `origin`.
    pub fn marks_in(
        &self,
        factory: &Factory,
        towns: &[Town],
        lo: (i32, i32),
        hi: (i32, i32),
        origin: (i32, i32),
        ranges: bool,
    ) -> Vec<i32> {
        let mut out = Vec::new();
        self.each_mark(factory, towns, lo, hi, ranges, &mut |x, z, s| {
            out.extend_from_slice(&[x - origin.0, z - origin.1, s.colour(), s.shape()]);
        });
        out
    }

    /// Calls `f` with the column and subject of every mark for columns `lo..=hi` (x, z): satellite footprints, with
    /// `ranges` every long-distance pole's reach, survey guesses, ore seen at the surface, then remembered deposits and
    /// those survey satellites saw (both while they hold ore), then terraforming sites, then machines (pod launchers and
    /// pads last, with their channel), then towns (in place of their depots).
    pub(super) fn each_mark(
        &self,
        factory: &Factory,
        towns: &[Town],
        lo: (i32, i32),
        hi: (i32, i32),
        ranges: bool,
        f: &mut impl FnMut(i32, i32, Subject),
    ) {
        let inside = |p: IVec3| (lo.0..=hi.0).contains(&p.x) && (lo.1..=hi.1).contains(&p.z);
        let (mid, half) = (((lo.0 + hi.0) / 2, (lo.1 + hi.1) / 2), (hi.0 - lo.0).max(hi.1 - lo.1) / 2);
        for (x, z, reach, kind) in factory.orbit.footprints(mid.0, mid.1, factory.orbit.now) {
            if (x - mid.0).abs() <= half + reach && (z - mid.1).abs() <= half + reach {
                f(x, z, Subject::Cover { kind, reach });
            }
        }
        let reach = factory::LONG_STATS.link;
        for (p, _, _) in factory.charge_poles().filter(|p| ranges && p.2) {
            if (p.x - mid.0).abs() <= half + reach && (p.z - mid.1).abs() <= half + reach {
                f(p.x, p.z, Subject::Range { reach });
            }
        }
        for g in self.survey.guesses().iter().filter(|g| (lo.0..=hi.0).contains(&g.x) && (lo.1..=hi.1).contains(&g.z)) {
            f(g.x, g.z, Subject::Guess { ore: g.ore, strength: g.strength });
        }
        let tiles = ((lo.0 >> CHUNK_SHIFT, lo.1 >> CHUNK_SHIFT), (hi.0 >> CHUNK_SHIFT, hi.1 >> CHUNK_SHIFT));
        self.atlas.each_tile_in(tiles.0, tiles.1, |cx, cz, tile| {
            for &(x, z, ore) in &tile.spots {
                let p = IVec3::new((cx << CHUNK_SHIFT) + x as i32, 0, (cz << CHUNK_SHIFT) + z as i32);
                if inside(p) {
                    f(p.x, p.z, Subject::Ore(ore));
                }
            }
        });
        let mut deposit = |d: &Deposit| {
            if inside(d.center) {
                f(d.center.x, d.center.z, Subject::Deposit { ore: d.ore(), tier: d.tier(), dry: dry(factory, d) });
            }
        };
        self.known.deposits.iter().for_each(&mut deposit);
        self.seen.deposits().filter(|d| !self.known.has(&d.key)).for_each(&mut deposit);
        // The player found the deposits their miners stand on by building there, scanned or not.
        let mut listed: FxHashSet<DepositKey> = self.known.deposits.iter().map(|d| d.key).collect();
        listed.extend(self.seen.deposits().map(|d| d.key));
        for (_, key, _) in factory.miner_deposits() {
            if !listed.insert(key) {
                continue;
            }
            if let Some(st) = factory.deposits.get(&key) {
                deposit(&st.deposit);
            }
        }
        let dry_miners: Vec<IVec3> = factory.miner_deposits().filter(|m| m.2).map(|m| m.0).collect();
        for s in &factory.sites.list {
            let (x, z) = ((s.lo.0 + s.hi.0) / 2, (s.lo.1 + s.hi.1) / 2);
            if (lo.0..=hi.0).contains(&x) && (lo.1..=hi.1).contains(&z) {
                f(x, z, Subject::Site { job: s.job as u8 });
            }
        }
        factory.map_machines(lo, hi, &mut |block, tier, p| {
            let subject = match block {
                block::POD_LAUNCHER | block::LANDING_PAD | block::TOWN_DEPOT => return, // drawn below
                block::LAUNCH_SITE => Subject::Launch,
                _ => Subject::Machine { block, tier, dry: dry_miners.contains(&p) },
            };
            f(p.x, p.z, subject);
        });
        for (block, p, channel) in factory.pod_machines().filter(|&(_, p, _)| inside(p)) {
            f(p.x, p.z, Subject::Pod { launcher: block == block::POD_LAUNCHER, channel });
        }
        for t in towns.iter().filter(|t| inside(t.pos)) {
            f(t.pos.x, t.pos.z, Subject::Town { kind: t.kind, state: t.state() });
        }
    }
}

/// Whether a deposit has been worked out (one nobody has touched is full).
fn dry(factory: &Factory, d: &Deposit) -> bool {
    factory.deposits.get(&d.key).is_some_and(|st| st.exhausted())
}

/// Mark colours, in the spirit of the surface hints (`worldgen/geology.rs`); also the map's colour for
/// ore seen at the surface, and the ore guide's.
pub fn ore_color(ore: BlockId) -> i32 {
    match ore {
        block::COAL_ORE => 0x2a2a2e,
        block::IRON_ORE => 0xd0703a,
        block::COPPER_ORE => 0x3cc8a0,
        block::LIMESTONE => 0xefe6c8,
        block::QUARTZ_ORE => 0xf4c6e8,
        block::BAUXITE_ORE => 0xc2553a,
        block::OIL_SAND => 0xa87a2c,
        block::URANIUM_ORE => 0x9bd84a,
        _ => 0xffffff,
    }
}

/// A machine's mark colour (a processor's comes from its spec).
fn machine_color(block: BlockId) -> i32 {
    match block {
        block::MINER => 0xf2c230,
        block::COAL_MINER => 0xc9902c,
        block::STORAGE => 0xb07a44,
        block::GENERATOR => 0xa070e0,
        block::LAB => 0x5ad1e0,
        block::QUARRY => 0xe08a3c,
        _ => factory::process_spec(block).map_or(0xcccccc, |s| s.map_colour),
    }
}

#[cfg(test)]
mod tests;
