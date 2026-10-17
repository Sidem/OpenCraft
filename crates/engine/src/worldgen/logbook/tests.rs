use super::*;
use crate::worldgen::WorldGen;

fn is_tip(t: &str) -> bool {
    TIPS.contains(&t)
}

fn is_fragment(t: &str) -> bool {
    FRAGMENTS.contains(&t)
}

#[test]
fn the_first_ruin_has_a_logbook_on_its_slab() {
    for seed in [2024, 1337, 7, 99] {
        let mut g = WorldGen::new(seed);
        let r = g.ruin_at(0, 0).expect("a first ruin");
        let p = r.logbook;
        let c = g.generate(IVec3::new(p.x >> 5, p.y >> 5, p.z >> 5));
        assert_eq!(c.get((p.x & 31) as usize, (p.y & 31) as usize, (p.z & 31) as usize), block::LOGBOOK, "seed {seed}");
        assert!(g.logbook_at(p).is_some());
        assert!(g.logbook_at(p + IVec3::new(1, 0, 0)).is_none(), "only the logbook's own cell reads");
        assert!(g.logbook_at(IVec3::new(0, 64, 0)).is_none());
    }
}

#[test]
fn near_ruins_hold_early_pages_only() {
    let g = WorldGen::new(1337);
    let mut bearings = 0;
    for i in 0..60 {
        let (x, z) = (150 + i * 5, -100 + i * 6);
        let page = g.page(x, z);
        assert!(page.title.starts_with("Crew log, day "));
        assert!(!is_fragment(&page.text), "a near ruin tells no late story");
        match page.pin {
            Some((_, _, ore)) => {
                bearings += 1;
                assert!(NEAR_ORES.contains(&ore), "near pages point at early ore");
            }
            None => assert!(is_tip(&page.text), "{}", page.text),
        }
    }
    assert!(bearings >= 10, "bearings are about half the pages, got {bearings} of 60");
}

#[test]
fn far_ruins_tell_the_late_story_and_point_at_far_ground() {
    let g = WorldGen::new(1337);
    let (mut fragments, mut bearings) = (0, 0);
    for i in 0..80 {
        let (x, z) = (900 + i * 13, 400 - i * 17);
        let page = g.page(x, z);
        if is_fragment(&page.text) {
            fragments += 1;
            assert!(page.pin.is_none());
        }
        if let Some((_, _, ore)) = page.pin {
            bearings += 1;
            assert!(FAR_ORES.contains(&ore));
        }
    }
    assert!(fragments >= 15 && bearings >= 15, "{fragments} fragments, {bearings} bearings of 80");
}

#[test]
fn a_bearing_points_at_ground_that_holds_the_ore_and_says_how_far() {
    let g = WorldGen::new(1337);
    let mut checked = 0;
    for i in 0..60 {
        let (x, z) = (-300 + i * 17, 200 - i * 9);
        let page = g.page(x, z);
        let Some((px, pz, ore)) = page.pin else { continue };
        checked += 1;
        let biome = g.biome_at(px, pz, g.height_at(px, pz));
        assert!(g.holds_ore(ore, biome, IVec3::new(px, 0, pz)), "{px}, {pz} cannot hold {}", block::def(ore).name);
        let d = f64::from((px - x).pow(2) + (pz - z).pow(2)).sqrt();
        assert!(d >= f64::from(MIN_BEARING) - 1.0);
        let said = format!("about {} blocks {}", (d.round() as i32 + 5) / 10 * 10, compass(px - x, pz - z));
        assert!(page.text.contains(&said), "{} / {said}", page.text);
        assert!(page.text.contains(block::def(ore).name));
    }
    assert!(checked >= 10);
}

#[test]
fn a_page_depends_only_on_the_seed_and_the_spot() {
    let (a, b) = (WorldGen::new(5), WorldGen::new(5));
    b.bearing_to(IRON_ORE, 900, -900); // warms b's caches in another order
    for (x, z) in [(210, 40), (-640, 300), (1500, -700)] {
        let (p, q) = (a.page(x, z), b.page(x, z));
        assert_eq!((p.title, p.text, p.pin), (q.title, q.text, q.pin));
    }
    assert_ne!(WorldGen::new(6).page(210, 40).title, a.page(210, 40).title, "seeds write different logs");
}

/// Prints how far the nearest ground of each early ore lies from a few spots (`-- --ignored --nocapture`), to tune `MIN_BEARING`.
#[test]
#[ignore]
fn how_far_early_ore_lies() {
    for seed in [1337, 2024, 7] {
        let g = WorldGen::new(seed);
        for (x, z) in [(200, 0), (-250, 150), (0, -300), (400, 300)] {
            let d: Vec<String> = NEAR_ORES
                .iter()
                .map(|&o| {
                    g.bearing_to(o, x, z).map_or("-".to_string(), |b| format!("{}:{}", block::def(o).name, b.distance))
                })
                .collect();
            println!("seed {seed} at {x},{z}: {}", d.join("  "));
        }
    }
}

#[test]
fn compass_points() {
    assert_eq!(compass(100, 0), "east");
    assert_eq!(compass(-100, 20), "west");
    assert_eq!(compass(0, -100), "north");
    assert_eq!(compass(10, 100), "south");
    assert_eq!(compass(100, -100), "north-east");
    assert_eq!(compass(-80, -100), "north-west");
    assert_eq!(compass(70, 100), "south-east");
    assert_eq!(compass(-100, 90), "south-west");
}
