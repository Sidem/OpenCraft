use super::*;
use crate::item::MINER_MK3;

#[test]
fn the_standard_palette_is_the_default_and_the_colour_blind_one_replaces_every_colour() {
    let mut g = Game::new(7, 2);
    assert_eq!(g.item_tier_colour(MINER_MK3.0), TIER_COLOURS[2], "the standard palette until one is chosen");
    g.set_tier_palette(COLOUR_BLIND);
    assert_eq!(g.item_tier_colour(MINER_MK3.0), TIER_COLOURS_CB[2]);
    for tier in 0..5u8 {
        assert_eq!(g.tier_colour(tier), TIER_COLOURS_CB[tier as usize]);
        assert!(!TIER_COLOURS.contains(&TIER_COLOURS_CB[tier as usize]), "Mk{} keeps no standard colour", tier + 1);
    }
    assert_eq!(g.tier_palette_colours(COLOUR_BLIND), TIER_COLOURS_CB.to_vec());
    assert_eq!(g.tier_palette_colours(STANDARD), TIER_COLOURS.to_vec());
    assert_eq!(colours(200), TIER_COLOURS, "an unknown palette is the standard one");
}

#[test]
fn the_recoloured_layers_are_the_fourteen_that_wear_a_tier_colour() {
    let all = layers();
    assert_eq!(all.len(), 14 * 3, "5 stripes, 5 paints and the 4 kit cases");
    let mut seen: Vec<u16> = all.chunks(3).map(|t| t[0]).collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 14, "no layer twice");
    for t in all.chunks(3) {
        assert!(t[1] < 5 && t[2] < 3 && usize::from(t[0]) < tex::COUNT, "{t:?}");
    }
}

#[test]
fn the_colour_blind_palette_runs_dark_to_light() {
    let lightness =
        |c: u32| 0.2126 * f64::from(c >> 16) + 0.7152 * f64::from(c >> 8 & 0xff) + 0.0722 * f64::from(c & 0xff);
    for pair in TIER_COLOURS_CB.windows(2) {
        assert!(lightness(pair[0]) < lightness(pair[1]), "{:06x} then {:06x}", pair[0], pair[1]);
    }
}

#[test]
fn choosing_a_palette_leaves_the_state_hash_alone() {
    let mut a = Game::new(7, 2);
    a.run_ticks(3);
    a.set_tier_palette(COLOUR_BLIND);
    a.run_ticks(3);
    let mut b = Game::new(7, 2);
    b.run_ticks(6);
    assert_eq!(a.state_hash(), b.state_hash(), "a palette is not core state");
}
