use super::*;
use crate::block::{self, FACE_BOTTOM, MINER, SMELTER};
use crate::math::Vec3;
use crate::tests::{find_outcrop_block, power_up, run_until_ready};

const POWER: u32 = Cause::Power as u32 + 1;
const INPUT: u32 = Cause::Input as u32 + 1;

fn ready() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g
}

/// Puts the player at `at`, ticks included, as the camera would follow.
fn stand_at(g: &mut Game, at: IVec3) {
    g.body_mut().pos = Vec3::new(at.x as f64, at.y as f64 + 1.0, at.z as f64);
    g.render_eye = g.body().pos;
}

/// A miner on an outcrop with no pole: stalled for want of power. The player stands beside it.
fn unwired_miner(g: &mut Game) -> IVec3 {
    let (ore, _) = find_outcrop_block(g, 1);
    let m = ore + IVec3::new(0, 1, 0);
    g.sim.world.set_block(m, MINER);
    let key = g.sim.factory.deposits.lookup(&mut g.sim.world, ore);
    g.sim.factory.add_miner(m, FACE_BOTTOM as u8, key, 0);
    stand_at(g, m + IVec3::new(3, 0, 0));
    m
}

/// A smelter with nothing to smelt, a cell off the miner's: stalled for want of input.
fn idle_smelter(g: &mut Game, near: IVec3) {
    let at = near + IVec3::new(0, 0, 3);
    g.sim.factory.place(&mut g.sim.world, SMELTER, at, 0, at, 0);
}

#[test]
fn an_unwired_miner_shows_its_lesson_after_half_a_minute_and_fixing_it_teaches_it_for_good() {
    let mut g = ready();
    let m = unwired_miner(&mut g);
    g.skip_time(20.0);
    assert_eq!(g.lesson_now(), 0, "not yet: it may fix itself");
    g.skip_time(15.0);
    assert_eq!(g.lesson_now(), POWER);
    let text = g.lesson_text();
    assert!(text.starts_with("No power:") && text.contains("Miner"), "{text}");
    let marker = g.lesson_marker();
    assert_eq!((marker.len(), marker[3]), (4, PROMPT_CHECK as f32), "a prompt over the machine");

    power_up(&mut g, m);
    g.skip_time(10.0);
    assert_eq!(g.lesson_now(), 0, "it works now");
    assert_ne!(g.sim.taught & mask(stall_bit(Cause::Power)), 0, "fixing it taught it");

    // Pulling the pole down again, minutes later, brings the lesson back for no one.
    g.sim.factory.remove(m + IVec3::new(0, 2, 0));
    g.skip_time(150.0);
    assert_eq!(g.lesson_now(), 0);
}

#[test]
fn waving_a_lesson_away_teaches_it_and_starts_a_quiet_spell() {
    let mut g = ready();
    let m = unwired_miner(&mut g);
    g.skip_time(35.0);
    assert_eq!(g.lesson_now(), POWER);
    g.lesson_dismiss();
    g.skip_time(1.0);
    assert_eq!((g.lesson_now(), g.lesson_text()), (0, String::new()));
    assert_ne!(g.sim.taught & mask(stall_bit(Cause::Power)), 0);

    // A different trouble does not show during the quiet spell, then does.
    idle_smelter(&mut g, m);
    g.skip_time(QUIET_TICKS as f64 / TICK_RATE as f64 - 20.0);
    assert_eq!(g.lesson_now(), 0, "quiet");
    g.skip_time(60.0);
    assert_eq!(g.lesson_now(), INPUT);
    assert_eq!(g.lesson_now(), g.lesson_now(), "one at a time, steady");
}

#[test]
fn trouble_out_of_reach_or_at_the_opening_scene_stays_quiet() {
    let mut g = ready();
    let m = unwired_miner(&mut g);
    stand_at(&mut g, m + IVec3::new(100, 0, 0));
    g.skip_time(90.0);
    assert_eq!(g.lesson_now(), 0, "too far to be the player's business");

    let mut opening = Game::new_normal(2024, 3);
    run_until_ready(&mut opening);
    opening.skip_time(90.0);
    assert_eq!(opening.lesson_now(), 0, "the scene's furnace waits for ore by design");
}

#[test]
fn a_world_taught_every_cause_never_scans() {
    let mut g = ready();
    unwired_miner(&mut g);
    g.sim.taught = crate::lessons::STALL_MASK;
    g.skip_time(90.0);
    assert_eq!(g.lesson_now(), 0);
    assert!(g.stall.watch.is_none());
}

#[test]
fn every_cause_has_words_that_name_the_machine() {
    for (cause, text) in LESSONS.iter().enumerate() {
        assert!(text.contains("{m}") && text.contains(": "), "cause {cause}: a title, then the machine's name");
    }
    assert_eq!(block::def(MINER).name, "Miner Mk1");
}
