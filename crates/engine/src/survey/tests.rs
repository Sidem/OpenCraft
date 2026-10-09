use super::*;
use crate::block::{IRON_ORE, STONE};
use crate::math::IVec3;
use crate::minimap::MARK_GUESS;
use crate::tests::run_until_ready;

fn patch(atlas: &mut Atlas, x: i32, z: i32, size: i32, stain: BlockId) {
    for dz in 0..size {
        for dx in 0..size {
            atlas.plant(x + dx, z + dz, stain);
        }
    }
}

#[test]
fn stained_ground_is_guessed_as_the_ore_below_and_a_lone_stain_is_not() {
    let mut atlas = Atlas::default();
    patch(&mut atlas, 100, 100, 6, RUSTY_SOIL);
    patch(&mut atlas, -60, 20, 3, DARK_SOIL); // 9 columns: enough
    atlas.plant(5, 5, GREEN_SAND); // alone
    atlas.plant(7, 5, STONE);
    let guesses = survey(&atlas, (0, 0), &[]);
    assert_eq!(guesses.len(), 2, "{guesses:?}");
    assert_eq!((guesses[0].ore, guesses[1].ore), (COAL_ORE, IRON_ORE), "nearest first");
    let iron = guesses[1];
    assert!((iron.x - 103).abs() <= FUZZ && (iron.z - 103).abs() <= FUZZ, "{iron:?}");
    assert_eq!((guesses[0].strength, iron.strength), (1, 2), "36 stained columns are stronger than 9");
}

#[test]
fn only_ground_within_the_radius_counts_and_neighbouring_squares_join() {
    let mut atlas = Atlas::default();
    patch(&mut atlas, 300, 0, 6, PALE_SOIL);
    assert!(survey(&atlas, (0, 0), &[]).is_empty(), "300 blocks out");
    assert_eq!(survey(&atlas, (100, 0), &[]).len(), 1, "200 blocks out");
    // A stain across two squares is one guess.
    patch(&mut atlas, 14, 0, 6, RUSTY_SOIL);
    let guesses = survey(&atlas, (0, 0), &[]);
    assert_eq!((guesses.len(), guesses[0].ore), (1, IRON_ORE));
}

#[test]
fn a_prospected_deposit_of_that_ore_clears_the_guess() {
    let mut atlas = Atlas::default();
    patch(&mut atlas, 100, 100, 6, RUSTY_SOIL);
    assert_eq!(survey(&atlas, (0, 0), &[(110, 110, COAL_ORE)]).len(), 1, "another ore does not");
    assert_eq!(survey(&atlas, (0, 0), &[(110, 110, IRON_ORE)]).len(), 0, "found");
    assert_eq!(survey(&atlas, (0, 0), &[(300, 300, IRON_ORE)]).len(), 1, "far away it does not");
}

#[test]
fn the_maps_show_guesses_only_once_ai_survey_is_researched() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let c = g.body().pos.floor();
    // Stain a patch of the loaded ground near the player.
    for dz in 0..6 {
        for dx in 0..6 {
            let (x, z) = (c.x + 20 + dx, c.z + 20 + dz);
            let top = (0..crate::worldgen::WORLD_HEIGHT)
                .rev()
                .find(|&y| g.sim.world.get_block(IVec3::new(x, y, z)).is_some_and(|b| b != crate::block::AIR))
                .unwrap();
            assert!(g.sim.world.set_block(IVec3::new(x, top, z), RUSTY_SOIL));
        }
    }
    while g.next_event() != 0 {}
    g.minimap_redraw();
    let window = |g: &Game| g.world_map_marks(c.x, c.z, c.x + 60, c.z + 60);
    let guesses =
        |m: &[i32]| m.chunks(4).filter(|r| r[3] >= MARK_GUESS).map(|r| (r[0], r[1], r[2])).collect::<Vec<_>>();
    assert!(guesses(&window(&g)).is_empty(), "not researched");
    g.sim.factory.research.complete_all();
    g.minimap_redraw();
    // The world's own stains are guessed too; ours is the iron ring near the patch.
    let iron = crate::minimap::ore_color(IRON_ORE);
    let ours = guesses(&window(&g))
        .into_iter()
        .filter(|&(x, z, colour)| colour == iron && (x - (c.x + 23)).abs() <= 10 && (z - (c.z + 23)).abs() <= 10);
    assert_eq!(ours.count(), 1);
}

#[test]
fn the_worlds_own_stains_stand_for_the_ore_the_generator_put_below() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.minimap_redraw();
    let (mut seen, mut wrong) = (0, Vec::new());
    g.minimap.atlas.each_tile_in((-40, -40), (40, 40), |cx, cz, tile| {
        for (i, &c) in tile.columns.iter().enumerate() {
            let stain = (c & 0xff) as BlockId;
            let Some(ore) = ore_of(stain) else { continue };
            let (x, z) = ((cx << 5) + (i as i32 & 31), (cz << 5) + (i as i32 >> 5));
            seen += 1;
            if let Some(d) = g.sim.world.generator().hint_source(x, z, stain) {
                let pale_pair = ore == LIMESTONE && d.ore() == crate::block::QUARTZ_ORE;
                if d.ore() != ore && !pale_pair {
                    wrong.push((x, z, stain, d.ore()));
                }
            }
        }
    });
    assert!(seen > 0, "this seed shows stained ground near the start");
    assert!(wrong.is_empty(), "{wrong:?}");
}
