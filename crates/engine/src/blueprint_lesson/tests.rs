use super::{PROMPT_COPY, PROMPT_CORNER};
use crate::action::Action;
use crate::block::{BlockId, SMELTER};
use crate::item::ItemId;
use crate::lessons::{mask, BLUEPRINT_BIT};
use crate::math::IVec3;
use crate::need_lesson::NEED_BASE;
use crate::raycast::RayHit;
use crate::sim::PlayerId;
use crate::tests::run_until_ready;
use crate::Game;

const LESSON: u32 = NEED_BASE + BLUEPRINT_BIT as u32;

fn build(g: &mut Game, block: BlockId, pos: IVec3) {
    let p = PlayerId(0);
    g.sim.apply(p, Action::Give { item: block.into(), count: 1 });
    let slot = g.sim.player(p).unwrap().inventory.slots.iter().position(|s| s.item == ItemId::block(block)).unwrap();
    g.sim.apply(p, Action::PlaceBlock { pos, slot: slot as u8, facing: 1, against: pos - IVec3::new(0, 1, 0) });
    g.run_ticks(1);
}

/// Three smelters in a row 4 apart, so the third brought the blueprint prompt (and a preset box). Ghost mode is off.
fn prompted() -> (Game, IVec3) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.sim.factory.research.finish("Construction Drones");
    let base = g.body().eye().floor() + IVec3::new(3, 3, 3);
    for i in 0..3 {
        build(&mut g, SMELTER, base + IVec3::new(4 * i, 0, 0));
    }
    assert!(!g.repeat_prompt().is_empty());
    (g, base)
}

fn aim_at(g: &mut Game, block: IVec3) {
    g.target = Some(RayHit { block, normal: IVec3::new(0, 1, 0), id: 0 });
}

fn taught(g: &Game) -> bool {
    g.sim.taught & mask(BLUEPRINT_BIT) != 0
}

fn kinds(g: &Game) -> Vec<f32> {
    g.lesson_marker().chunks(4).map(|c| c[3]).collect()
}

/// Accepted: B pressed while the prompt shows, a second later the walk-through has taken over.
fn accepted() -> (Game, IVec3) {
    let (mut g, base) = prompted();
    g.toggle_ghost_mode();
    g.skip_time(1.5);
    (g, base)
}

#[test]
fn accepting_the_prompt_starts_the_walk_through_and_saving_a_blueprint_teaches_it() {
    let (mut g, _) = accepted();
    assert_eq!(g.repeat_prompt(), "", "the prompt gives way");
    assert_eq!(g.lesson_now(), LESSON);
    assert_eq!(g.lesson_head(), "Good to know");
    let text = g.lesson_text();
    assert!(text.contains("[Enter]"), "the box is preset: {text}");
    assert_eq!(kinds(&g), vec![PROMPT_COPY as f32], "one Enter prompt over the box");
    g.copy_blueprint();
    g.skip_time(2.0);
    assert!(taught(&g));
    assert_ne!(g.lesson_now(), LESSON);
    assert!(g.lesson_marker().is_empty());
}

#[test]
fn the_stage_follows_the_keys_corner_by_corner() {
    let (mut g, base) = accepted();
    g.library.selection = None; // the player cleared the preset box
    assert!(g.lesson_text().contains("[Z]") && g.lesson_text().contains("opposite corner"));
    assert_eq!(kinds(&g), vec![PROMPT_CORNER as f32; 2], "Z over both corners of the suggested box");
    aim_at(&mut g, base);
    g.mark_blueprint_corner();
    assert!(g.lesson_text().starts_with("Blueprint: now aim"), "{}", g.lesson_text());
    let far = g.lesson_marker();
    assert_eq!((far.len(), far[3]), (4, PROMPT_CORNER as f32), "only the far corner is asked for");
    aim_at(&mut g, base + IVec3::new(8, 1, 0));
    g.mark_blueprint_corner();
    assert!(g.lesson_text().contains("[Enter]"));
    assert_eq!(kinds(&g), vec![PROMPT_COPY as f32]);
    g.copy_blueprint();
    g.skip_time(2.0);
    assert!(taught(&g));
}

#[test]
fn the_far_prompt_sits_over_the_corner_away_from_the_marked_one() {
    let (mut g, base) = accepted();
    g.library.selection = None;
    let both = g.lesson_marker();
    aim_at(&mut g, base);
    g.mark_blueprint_corner();
    let far = g.lesson_marker();
    let (a, b) = (&both[..3], &both[4..7]);
    // The marked corner is the one near the first smelter (the suggested box's lowest corner).
    assert_eq!(&far[..3], b, "far from {a:?}");
}

#[test]
fn the_card_waits_for_ghost_mode_and_comes_back_with_it() {
    let (mut g, _) = accepted();
    assert_eq!(g.lesson_now(), LESSON);
    g.toggle_ghost_mode();
    assert_eq!((g.lesson_now(), g.lesson_text()), (0, String::new()), "no card outside ghost mode");
    g.skip_time(3.0);
    assert!(!taught(&g), "leaving ghost mode is not finishing");
    g.toggle_ghost_mode();
    assert_eq!(g.lesson_now(), LESSON);
    assert_eq!(kinds(&g).len(), 2, "the leaving forgot the box: both corners again");
}

#[test]
fn without_accepting_nothing_starts_and_the_prompt_stays() {
    let (mut g, _) = prompted();
    g.skip_time(5.0);
    assert_eq!(g.lesson_now(), 0);
    assert!(!g.repeat_prompt().is_empty());
    assert!(!taught(&g));
}

#[test]
fn h_teaches_it() {
    let (mut g, _) = accepted();
    g.lesson_dismiss();
    g.run_ticks(2);
    assert!(taught(&g));
    g.skip_time(3.0);
    assert_ne!(g.lesson_now(), LESSON);
}

#[test]
fn walking_away_drops_it_without_teaching() {
    let (mut g, _) = accepted();
    assert_eq!(g.lesson_now(), LESSON);
    g.body_mut().pos.x += 200.0;
    g.skip_time(2.0);
    assert!(!taught(&g));
    g.body_mut().pos.x -= 200.0;
    g.skip_time(2.0);
    assert_eq!(g.lesson_now(), 0, "the prompt is gone and the lesson was dropped");
}

#[test]
fn a_world_that_was_taught_it_never_shows_it() {
    let (mut g, _) = prompted();
    g.act(Action::LessonTaught { bit: BLUEPRINT_BIT });
    g.run_ticks(2);
    g.toggle_ghost_mode();
    g.skip_time(3.0);
    assert_eq!(g.lesson_now(), 0);
    assert!(!g.repeat_prompt().is_empty(), "the plain prompt is still there");
}

#[test]
fn the_cards_are_short() {
    for text in [super::FIRST, super::FAR, super::COPY] {
        assert!(text.split_whitespace().count() <= 20, "{text}");
    }
}
