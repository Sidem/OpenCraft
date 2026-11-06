use crate::action::Action;
use crate::block::{AIR, COAL_MINER, COPPER_ORE};
use crate::hints::GO_ELECTRIC;
use crate::math::IVec3;
use crate::minimap::{MARK_FIELDS, MARK_GOAL};
use crate::opening::{Progress, PROMPT_COPPER};
use crate::Game;

/// A new world past its opening, with Go electric the next tip.
fn at_go_electric(seed: u32) -> Game {
    let mut g = Game::new_normal(seed, 2);
    g.sim.opening.as_mut().unwrap().progress = Progress::Over;
    g.sim.hints_reached = GO_ELECTRIC as u8;
    g
}

#[test]
fn go_electric_sends_the_player_to_the_copper_patch_until_a_miner_digs_it() {
    let mut g = at_go_electric(2024);
    assert_eq!(g.hint_progress() as usize, GO_ELECTRIC);
    let patch = g.copper_goal().expect("a starter copper patch");
    assert_eq!(g.sim.world.block_anywhere_or_generate(patch), COPPER_ORE, "the patch's top block is copper");

    let text = g.hint_text(GO_ELECTRIC as u32);
    assert!(text.contains("coal miner") && text.contains("It lies about"), "names the miner and the way: {text}");
    assert_eq!(g.hint_marker().get(3), Some(&(PROMPT_COPPER as f32)), "the Copper prompt floats over it");
    assert_eq!(g.guide_craft(), COAL_MINER as i32, "none held: the build menu points at one");

    // The maps flag it, in the world map's marks.
    g.minimap_redraw();
    let marks = g.world_map_marks(patch.x - 8, patch.z - 8, patch.x + 8, patch.z + 8);
    let goal = marks.chunks(MARK_FIELDS).find(|m| m[3] == MARK_GOAL).expect("a goal mark");
    assert_eq!((goal[0], goal[1]), (patch.x, patch.z));

    // A coal miner set on it ends the guide.
    g.give(COAL_MINER as u16, 1);
    g.run_ticks(1);
    assert_eq!(g.guide_craft(), -1, "one held: nothing to craft");
    assert_eq!(g.sim.world.block_anywhere_or_generate(patch + IVec3::new(0, 1, 0)), AIR, "room on top");
    let slot = g.inventory().slots.iter().position(|s| s.item == COAL_MINER.into()).unwrap() as u8;
    g.act(Action::PlaceBlock { pos: patch + IVec3::new(0, 1, 0), slot, facing: 0, against: patch });
    g.run_ticks(1);
    assert_eq!(g.copper_goal(), None);
    assert!(!g.hint_text(GO_ELECTRIC as u32).contains("It lies about"));
    assert!(g.hint_marker().is_empty());
    g.minimap_redraw();
    assert_eq!(g.minimap.goal, None, "the flag comes down");
}

#[test]
fn the_copper_guide_waits_for_its_tip_and_skips_creative_worlds() {
    let mut g = at_go_electric(2024);
    g.sim.hints_reached = GO_ELECTRIC as u8 - 1;
    assert_eq!(g.copper_goal(), None, "an earlier tip still shows");
    g.sim.hints_reached = GO_ELECTRIC as u8 + 1;
    assert_eq!(g.copper_goal(), None, "Go electric is done");
    g.sim.opening.as_mut().unwrap().progress = Progress::Mining;
    g.sim.hints_reached = GO_ELECTRIC as u8;
    assert_eq!(g.copper_goal(), None, "the opening still runs");

    let mut g = Game::new_creative(2024, 2);
    g.sim.hints_reached = GO_ELECTRIC as u8;
    assert_eq!(g.copper_goal(), None);
    assert_eq!(g.guide_craft(), -1);
}
