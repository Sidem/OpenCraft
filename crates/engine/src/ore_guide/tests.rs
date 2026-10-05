//! The ore guide follows the world's generator; stained soil tells how far down its ore lies.

use rustc_hash::FxHashMap;

use super::*;
use crate::block::{GRASS, RUSTY_SOIL};
use crate::chunk::Chunk;

/// The guide's rows split into fields.
fn rows(gen: &WorldGen) -> Vec<Vec<String>> {
    guide_rows(gen).lines().map(|l| l.split('\t').map(str::to_string).collect()).collect()
}

#[test]
fn the_guide_gives_each_ores_band_for_the_worlds_generator() {
    for version in [1, 2, 3, 4, 5, 6] {
        let gen = WorldGen::with_version(7, version);
        let rows = rows(&gen);
        let made = GUIDE.iter().filter(|g| gen.has_ore(g.0)).count();
        assert_eq!(rows.len(), made, "the extra ores only from their versions");
        assert_eq!(made, [5, 5, 5, 5, 6, 8][version as usize - 1]);
        for (row, (ore, ..)) in rows.iter().zip(GUIDE) {
            assert_eq!(row.len(), 7, "{row:?}");
            assert_eq!(row[0], ore.to_string());
            assert_eq!((row[3].parse().unwrap(), row[4].parse().unwrap()), gen.ore_band(ore), "v{version}");
        }
        let notes = guide_notes(&gen);
        assert_eq!(notes.contains("Stained soil"), version >= 2, "v{version}: {notes}");
    }
    let iron = &rows(&WorldGen::with_version(7, 4))[1];
    assert_eq!((iron[2].as_str(), iron[3].as_str(), iron[4].as_str()), ("Iron ore", "8", "28"));
    assert!(iron[5].starts_with("Share of deposits: basalt fields (black rock) 60%"), "{}", iron[5]);
    assert!(iron[6].contains("rusty soil"), "{}", iron[6]);
}

#[test]
fn stained_soil_tells_the_ore_below_and_its_depth() {
    let mut gen = WorldGen::new(2024);
    let mut chunks: FxHashMap<IVec3, Chunk> = FxHashMap::default();
    let mut sources = Vec::new();
    for cz in -2..2 {
        for cx in -2..2 {
            gen.seed_deposits(cx, cz, &mut sources);
        }
    }
    let mut checked = 0;
    // The tops around each vein and lode, read the way the HUD does.
    for d in sources.iter().filter(|d| d.tier() != Tier::Outcrop) {
        for (dx, dz) in [(0, 0), (2, 1), (-1, 3), (-3, -2), (4, 0)] {
            let (x, z) = (d.center.x + dx, d.center.z + dz);
            let h = gen.height_at(x, z);
            let key = IVec3::new(x >> 5, h >> 5, z >> 5);
            let top = chunks.entry(key).or_insert_with(|| gen.generate(key)).get(
                (x & 31) as usize,
                (h & 31) as usize,
                (z & 31) as usize,
            );
            let Some(text) = stain_reading(&gen, IVec3::new(x, h, z), top) else { continue };
            let source = gen.hint_source(x, z, top).expect("every generated stain has a source");
            let depth = (h - source.bounds().1.y).max(1);
            assert!(text.contains(&format!("about {depth} blocks down")), "{text}");
            assert!(text.contains(&source.name().to_ascii_lowercase()), "{text}");
            checked += 1;
        }
    }
    assert!(checked >= 5, "only {checked} stains read");
    assert_eq!(stain_reading(&gen, IVec3::new(0, 70, 0), GRASS), None, "plain grass says nothing");
    assert!(stain_reading(&gen, IVec3::new(0, 70, 0), RUSTY_SOIL).is_some(), "a stain always says something");
}
