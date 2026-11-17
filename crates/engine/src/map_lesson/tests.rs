use super::{BACK, FAR, MAP};
use crate::action::Action;
use crate::lessons::{mask, MAP_BIT};
use crate::math::Vec3;
use crate::need_lesson::NEED_BASE;
use crate::tests::run_until_ready;
use crate::Game;

const LESSON: u32 = NEED_BASE + MAP_BIT as u32;

fn taught(g: &Game) -> bool {
    g.sim.taught & mask(MAP_BIT) != 0
}

/// A fresh world with the player standing `out` blocks east of spawn, a second on.
fn stand_at(out: f64) -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    go(&mut g, out);
    g
}

fn go(g: &mut Game, out: f64) {
    let y = g.body().pos.y;
    g.body_mut().pos = Vec3::new(out, y, 0.0);
    g.run_ticks(1);
}

#[test]
fn far_from_spawn_the_card_shows_and_opening_the_map_teaches_it() {
    let mut g = stand_at(FAR + 10.0);
    g.skip_time(1.5);
    assert_eq!(g.lesson_now(), 0, "not on the first look");
    g.skip_time(4.0);
    assert_eq!((g.lesson_now(), g.lesson_head().as_str()), (LESSON, "Good to know"));
    assert_eq!(g.lesson_text(), MAP);
    assert!(g.lesson_marker().is_empty(), "card only");

    g.map_opened();
    g.run_ticks(2);
    assert!(taught(&g));
    assert_eq!((g.lesson_now(), g.lesson_text()), (0, String::new()));
}

#[test]
fn near_spawn_nothing_shows() {
    let mut g = stand_at(FAR - 10.0);
    g.skip_time(8.0);
    assert_eq!(g.lesson_now(), 0);
    assert!(!taught(&g));
}

#[test]
fn opening_the_map_early_teaches_it_before_any_card() {
    let mut g = stand_at(10.0);
    g.map_opened();
    g.run_ticks(2);
    assert!(taught(&g));
    go(&mut g, FAR + 50.0);
    g.skip_time(8.0);
    assert_eq!(g.lesson_now(), 0, "this player already knows the map");
}

#[test]
fn h_teaches_it_and_it_never_returns() {
    let mut g = stand_at(FAR + 10.0);
    g.skip_time(5.0);
    assert_eq!(g.lesson_now(), LESSON);
    g.lesson_dismiss();
    g.run_ticks(2);
    assert!(taught(&g));
    g.skip_time(5.0);
    assert_eq!(g.lesson_now(), 0);
}

#[test]
fn walking_home_drops_it_without_teaching_and_a_sprint_over_the_line_brings_nothing() {
    let mut g = stand_at(FAR + 10.0);
    g.skip_time(1.5);
    go(&mut g, FAR - BACK - 5.0);
    g.skip_time(5.0);
    assert_eq!(g.lesson_now(), 0, "dropped before the card showed");
    assert!(!taught(&g));

    go(&mut g, FAR + 10.0);
    g.skip_time(5.0);
    assert_eq!(g.lesson_now(), LESSON, "it comes back with the need");
    go(&mut g, 10.0);
    g.skip_time(2.0);
    assert_eq!(g.lesson_now(), 0, "and goes when the need does");
    assert!(!taught(&g), "going home is not learning the map");
}

#[test]
fn a_world_that_was_taught_it_never_shows_it() {
    let mut g = stand_at(FAR + 10.0);
    g.act(Action::LessonTaught { bit: MAP_BIT });
    g.skip_time(8.0);
    assert_eq!(g.lesson_now(), 0);
}

#[test]
fn the_card_is_short() {
    assert!(MAP.split_whitespace().count() <= 20, "{MAP}");
}
