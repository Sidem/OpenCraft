use super::{FAR, FIRST, NOT_HELD};
use crate::action::Action;
use crate::item::PLANNER;
use crate::lessons::{mask, PLANNER_BIT};
use crate::math::IVec3;
use crate::need_lesson::NEED_BASE;
use crate::raycast::RayHit;
use crate::tests::run_until_ready;
use crate::Game;

const LESSON: u32 = NEED_BASE + PLANNER_BIT as u32;

fn taught(g: &Game) -> bool {
    g.sim.taught & mask(PLANNER_BIT) != 0
}

/// A fresh world with a planner in the pack (not in hand), a second on for the scan to see it.
fn crafted() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.give(PLANNER.0, 1);
    g.run_ticks(1);
    let empty = g.inventory().slots.iter().take(9).position(|s| s.item != PLANNER && s.count == 0).unwrap();
    g.select_slot(empty as u32); // the new item lands in the selected slot: look away from it
    g.run_ticks(1);
    assert!(!g.planner_held());
    g
}

fn hold(g: &mut Game) {
    let slot = g.inventory().slots.iter().position(|s| s.item == PLANNER).unwrap();
    g.select_slot(slot as u32);
    g.run_ticks(1);
    assert!(g.planner_held());
}

fn click_corner(g: &mut Game, block: IVec3) {
    g.target = Some(RayHit { block, normal: IVec3::new(0, 1, 0), id: 1 });
    g.using = true;
    g.update_planner();
}

#[test]
fn the_card_follows_the_planner_from_the_pack_to_both_corners_then_the_first_site_teaches_it() {
    let mut g = crafted();
    g.skip_time(1.5);
    assert_eq!(g.lesson_now(), 0, "a moment to look at the new item first");
    g.skip_time(2.0);
    assert_eq!((g.lesson_now(), g.lesson_head().as_str()), (LESSON, "Good to know"));
    assert_eq!(g.lesson_text(), NOT_HELD);
    assert!(g.lesson_marker().is_empty(), "card only");

    hold(&mut g);
    assert_eq!(g.lesson_text(), FIRST);
    click_corner(&mut g, IVec3::new(900, 70, -900));
    assert_eq!(g.lesson_text(), FAR);
    click_corner(&mut g, IVec3::new(904, 72, -897));
    assert!(!taught(&g), "marking the corners is not yet a site");

    g.mark_site(900, -900, 904, -897, 70, 2);
    g.skip_time(1.5);
    assert!(taught(&g));
    assert_eq!((g.lesson_now(), g.lesson_text()), (0, String::new()));
}

#[test]
fn h_teaches_it_and_it_never_returns() {
    let mut g = crafted();
    g.skip_time(4.0);
    assert_eq!(g.lesson_now(), LESSON);
    g.lesson_dismiss();
    g.run_ticks(2);
    assert!(taught(&g));
    g.skip_time(5.0);
    assert_eq!(g.lesson_now(), 0);
}

#[test]
fn a_world_with_a_site_learns_without_a_card() {
    let mut g = crafted();
    g.mark_site(900, -900, 904, -897, 70, 2);
    g.skip_time(5.0);
    assert!(taught(&g));
    assert_eq!(g.lesson_now(), 0, "it never showed");
}

#[test]
fn without_a_planner_nothing_starts_and_losing_it_drops_the_card_without_teaching() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.skip_time(5.0);
    assert_eq!(g.lesson_now(), 0);
    assert!(!taught(&g));

    g.give(PLANNER.0, 1);
    g.run_ticks(1);
    hold(&mut g);
    g.skip_time(3.0);
    assert_eq!(g.lesson_now(), LESSON);
    g.act(Action::DropSelected { count: 1 });
    g.run_ticks(2);
    assert_eq!(g.inventory().count(PLANNER), 0);
    g.skip_time(1.5);
    assert_eq!(g.lesson_now(), 0);
    assert!(!taught(&g), "dropping the planner is not learning");
}

#[test]
fn a_world_that_was_taught_it_never_shows_it() {
    let mut g = crafted();
    g.act(Action::LessonTaught { bit: PLANNER_BIT });
    g.skip_time(5.0);
    assert_eq!(g.lesson_now(), 0);
}

#[test]
fn the_cards_are_short() {
    for text in [NOT_HELD, FIRST, FAR] {
        assert!(text.split_whitespace().count() <= 20, "{text}");
    }
}
