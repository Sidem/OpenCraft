use super::*;
use crate::factory::{copy_near, INSTANCE_FLOATS, ROUND};
use crate::item::XRAY_GOGGLES;
use crate::math::IVec3;
use crate::tests::run_until_ready;

fn wear_goggles(g: &mut Game) {
    let me = g.local.0 as usize;
    g.sim.players[me].as_mut().expect("local player").inventory.worn[6] = XRAY_GOGGLES;
}

/// A game with a belt line 6 blocks from the player and another 90 blocks away (inside the view distance, outside the
/// X-ray radius), one frame drawn.
fn base_with_near_and_far_belts() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let feet = IVec3::new(g.player_x().floor() as i32, g.player_y().floor() as i32, g.player_z().floor() as i32);
    for i in 0..6 {
        g.sim.factory.add_belt(feet + IVec3::new(6 + i, 0, 0), 1);
        g.sim.factory.add_belt(feet + IVec3::new(90 + i, 0, 0), 1);
    }
    g.update(1.0 / 60.0);
    g
}

#[test]
fn copy_near_keeps_what_is_in_the_radius_and_reports_the_cap() {
    let rec = |x: f32, round: bool| {
        let mut r = vec![0.0; INSTANCE_FLOATS];
        r[0] = x;
        r[11] = if round { ROUND as f32 } else { 0.0 };
        r
    };
    let all: Vec<f32> = [rec(3.0, false), rec(-10.0, true), rec(40.0, false), rec(27.9, false)].concat();
    let (mut out, mut round) = (Vec::new(), Vec::new());
    assert!(!copy_near(&all, 28.0, 10, &mut out, &mut round));
    assert_eq!(
        (out.len() / INSTANCE_FLOATS, round.len() / INSTANCE_FLOATS),
        (3, 1),
        "40 is out, the round body is also in `round`"
    );
    assert!(copy_near(&all, 28.0, 2, &mut out, &mut round), "a third record in range hits the cap");
    assert_eq!(out.len() / INSTANCE_FLOATS, 2);
}

#[test]
fn nothing_is_copied_without_worn_goggles_or_while_switched_off() {
    let mut g = base_with_near_and_far_belts();
    assert!(g.instance_count() + g.static_box_total() > 0, "the belts are drawn the usual way");
    assert_eq!((g.xray_worn(), g.xray_active(), g.xray_count()), (false, false, 0));
    assert!(g.toggle_xray().contains("Head slot"), "the key says what is missing");
    assert!(g.xray_enabled(), "and changes nothing");

    wear_goggles(&mut g);
    g.update(1.0 / 60.0);
    assert!(g.xray_active() && g.xray_count() > 0, "goggles on at once, with the view on by default");

    assert_eq!(g.toggle_xray(), "");
    g.update(1.0 / 60.0);
    assert!(!g.xray_active() && g.xray_count() == 0 && g.xray_round_count() == 0, "switched off: no work, no records");
    assert_eq!(g.toggle_xray(), "");
    g.update(1.0 / 60.0);
    assert!(g.xray_active() && g.xray_count() > 0);
}

#[test]
fn the_key_says_so_without_goggles_and_otherwise_flips_the_switch() {
    let mut g = Game::new(2024, 3);
    assert!(!g.toggle_xray().is_empty(), "no goggles: a line to show, and nothing changes");
    assert!(g.xray_enabled(), "the choice starts on");
    wear_goggles(&mut g);
    assert_eq!(g.toggle_xray(), "");
    assert!(!g.xray_enabled());
    assert_eq!(g.toggle_xray(), "");
    assert!(g.xray_enabled());
}

#[test]
fn only_the_factory_records_within_the_radius_are_copied() {
    let mut g = base_with_near_and_far_belts();
    wear_goggles(&mut g);
    g.update(1.0 / 60.0);

    // Counted a second way: ask the factory for its records and keep the near ones.
    let eye = g.render_eye;
    let mut all = Vec::new();
    g.sim.factory.write_instances(&mut all, eye, 0.0, g.sim.world.view_distance());
    let near = all
        .chunks_exact(INSTANCE_FLOATS)
        .filter(|r| r[0] * r[0] + r[1] * r[1] + r[2] * r[2] <= RADIUS * RADIUS)
        .count();
    assert!(near > 0 && near < all.len() / INSTANCE_FLOATS, "the far belts are drawn, but not X-rayed");
    assert_eq!(g.xray_count(), near);
    assert!(g
        .xray
        .boxes
        .chunks_exact(INSTANCE_FLOATS)
        .all(|r| r[0] * r[0] + r[1] * r[1] + r[2] * r[2] <= RADIUS * RADIUS));
    assert_eq!(
        g.xray_round_count(),
        g.xray.boxes.chunks_exact(INSTANCE_FLOATS).filter(|r| r[11] as u32 & ROUND != 0).count()
    );
}

#[test]
fn the_strategy_view_shows_no_xray() {
    let mut g = base_with_near_and_far_belts();
    wear_goggles(&mut g);
    g.update(1.0 / 60.0);
    assert!(g.xray_count() > 0);
    g.set_view_mode(2);
    g.update(1.0 / 60.0);
    assert!(g.strategy.active());
    assert!(!g.xray_active() && g.xray_count() == 0, "its record list is compacted, so the span would be wrong");
}
