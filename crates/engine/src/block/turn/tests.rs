use super::*;
use crate::block::tex;

fn mode(layer: u16) -> u32 {
    MASK[(layer / 16) as usize] >> (2 * (layer % 16)) & 3
}

#[test]
fn natural_layers_and_their_alternates_turn_but_machines_and_leaves_do_not() {
    assert!(mode(tex::STONE) == 3 && mode(tex::GRASS_TOP) == 3 && mode(tex::URANIUM_ORE) == 3);
    assert!(mode(tex::SANDSTONE) == 2 && mode(tex::SAND) == 2);
    assert!(mode(tex::GRASS_SIDE) == 1 && mode(tex::PALE_GRASS_SIDE) == 1);
    let first = tex::alternates(tex::LIMESTONE);
    assert!((first..first + tex::ALTERNATES).all(|l| mode(l) == 2));
    let coal = tex::alternates(tex::COAL_ORE);
    assert!((coal..coal + tex::ALTERNATES).all(|l| mode(l) == 3));
    let leaves = tex::alternates(tex::LEAVES);
    assert!(mode(tex::LEAVES) == 0 && (leaves..leaves + tex::ALTERNATES).all(|l| mode(l) == 0));
    assert!(mode(tex::BELT_TOP) == 0 && mode(tex::STONE_BRICKS) == 0 && mode(tex::WATER) == 0);
    let turning = (0..tex::COUNT as u16).filter(|&l| mode(l) != 0).count();
    assert_eq!(turning, TURNS.len() + LAYERED.len() + UPRIGHT.len() + 5 * tex::ALTERNATES as usize);
}
