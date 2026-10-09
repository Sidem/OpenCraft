//! Map marks: deposits the local player has prospected, ore seen at the surface (the explored map's
//! spots, `atlas.rs`) and the factory's machines, as flat records for the host to draw over the map
//! images (`web/src/ui/minimap.ts`, `web/src/ui/worldmap.ts`).
//!
//! Presentation only. The prospected deposits (`Known`) are the local player's memory, not core state:
//! the scanner and core drill add veins and lodes (`remember`; outcrops show on the map by themselves),
//! and the host keeps the list with the world's record in the browser (`export` / `import`), not in the
//! save file. A deposit that runs dry loses its mark and is left out of the next export. Minimap records
//! are relative to the column the image was last drawn around, so marks line up with its pixels; world
//! map records are in world columns.
//!
//! To mark another machine kind: a line in `Factory::map_machines` and a colour in `machine_color`
//! (processors: `map_colour` in their spec).

use crate::block::{self, BlockId};
use crate::chunk::CHUNK_SHIFT;
use crate::deposits::{Deposit, DepositKey, Tier};
use crate::factory::{self, Factory};
use crate::math::IVec3;
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

    /// Where the remembered deposits lie and their ore: x, z, block.
    pub fn centres(&self) -> impl Iterator<Item = (i32, i32, BlockId)> + '_ {
        self.deposits.iter().map(|d| (d.center.x, d.center.z, d.ore()))
    }

    /// The remembered deposits that still hold ore, `KNOWN_FIELDS` numbers each.
    pub fn export(&self, factory: &Factory) -> Vec<i32> {
        let mut out = Vec::new();
        for d in self.deposits.iter().filter(|d| !dry(factory, d)) {
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

impl Minimap {
    /// Marks inside the minimap image (`MARK_FIELDS` numbers each), relative to its centre column.
    /// Empty before the first redraw.
    pub fn marks(&self, factory: &Factory) -> Vec<i32> {
        let Some((cx, cz)) = self.centre else { return Vec::new() };
        self.marks_in(factory, (cx - HALF, cz - HALF), (cx + HALF - 1, cz + HALF - 1), (cx, cz))
    }

    /// Marks for columns `lo..=hi` (x, z), relative to column `origin`: survey guesses, ore seen at the surface, then
    /// remembered deposits that still hold ore, then terraforming sites, then machines.
    pub fn marks_in(&self, factory: &Factory, lo: (i32, i32), hi: (i32, i32), origin: (i32, i32)) -> Vec<i32> {
        let inside = |p: IVec3| (lo.0..=hi.0).contains(&p.x) && (lo.1..=hi.1).contains(&p.z);
        let (ox, oz) = origin;
        let mut out = Vec::new();
        for g in self.survey.guesses().iter().filter(|g| (lo.0..=hi.0).contains(&g.x) && (lo.1..=hi.1).contains(&g.z)) {
            out.extend_from_slice(&[g.x - ox, g.z - oz, ore_color(g.ore), MARK_GUESS + g.strength as i32 - 1]);
        }
        let tiles = ((lo.0 >> CHUNK_SHIFT, lo.1 >> CHUNK_SHIFT), (hi.0 >> CHUNK_SHIFT, hi.1 >> CHUNK_SHIFT));
        self.atlas.each_tile_in(tiles.0, tiles.1, |cx, cz, tile| {
            for &(x, z, ore) in &tile.spots {
                let p = IVec3::new((cx << CHUNK_SHIFT) + x as i32, 0, (cz << CHUNK_SHIFT) + z as i32);
                if inside(p) {
                    out.extend_from_slice(&[p.x - ox, p.z - oz, ore_color(ore), MARK_ORE]);
                }
            }
        });
        for d in self.known.deposits.iter().filter(|d| inside(d.center) && !dry(factory, d)) {
            out.extend_from_slice(&[d.center.x - ox, d.center.z - oz, ore_color(d.ore()), MARK_DEPOSIT]);
        }
        for s in &factory.sites.list {
            let (x, z) = ((s.lo.0 + s.hi.0) / 2, (s.lo.1 + s.hi.1) / 2);
            if (lo.0..=hi.0).contains(&x) && (lo.1..=hi.1).contains(&z) {
                out.extend_from_slice(&[x - ox, z - oz, SITE_COLOURS[s.job as usize], MARK_SITE]);
            }
        }
        factory.map_machines(lo, hi, &mut |block, p| {
            out.extend_from_slice(&[p.x - ox, p.z - oz, machine_color(block), MARK_MACHINE]);
        });
        out
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
        block::STORAGE => 0xb07a44,
        block::GENERATOR => 0xa070e0,
        block::LAB => 0x5ad1e0,
        block::QUARRY => 0xe08a3c,
        _ => factory::process_spec(block).map_or(0xcccccc, |s| s.map_colour),
    }
}

#[cfg(test)]
mod tests;
