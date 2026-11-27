use super::*;
use crate::block::{POLE, STONE};
use crate::inventory::Stack;
use crate::math::IVec3;
use crate::tests::run_until_ready;
use crate::Game;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A game on a flat stone platform (its ground level returned), the line hook in the left hand, nothing in the way.
fn hook_in_hand() -> (Game, i32) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    for x in -8..=40 {
        for z in -6..=6 {
            for h in y - 2..=y + 16 {
                g.sim.world.set_block_anywhere_later(v(x, h, z), if h <= y { STONE } else { AIR });
            }
        }
    }
    g.sim.factory.by_hand = true;
    g.run_ticks(3);
    let me = g.local.0 as usize;
    g.sim.players[me].as_mut().expect("local player").inventory.offhand = Stack { item: LINE_HOOK, count: 1 };
    g.body_mut().pos = Vec3::new(2.5, (y + 1) as f64, 0.5);
    (g, y)
}

/// Turns the view towards `at`.
fn look_at(g: &mut Game, at: Vec3) {
    let d = at - g.body().eye();
    let d = d * (1.0 / d.length());
    let body = g.body_mut();
    body.pitch = d.y.asin();
    body.yaw = d.x.atan2(-d.z);
}

#[test]
fn the_ray_meets_a_segment_where_it_passes_closest() {
    let (s, u, gap) =
        ray_segment(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::new(5.0, 1.0, -2.0), Vec3::new(5.0, 1.0, 2.0));
    assert!((s - 5.0).abs() < 1e-9 && (u - 0.5).abs() < 1e-9 && (gap - 1.0).abs() < 1e-9, "{s} {u} {gap}");
    // Behind the eye: the ray starts at it.
    let (s, _, gap) =
        ray_segment(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::new(-3.0, 0.0, 0.0), Vec3::new(-3.0, 0.0, 1.0));
    assert!(s == 0.0 && (gap - 3.0).abs() < 1e-9);
}

#[test]
fn the_hook_pulls_the_body_up_to_a_block_holds_it_while_g_is_down_and_jump_lets_go() {
    let (mut g, y) = hook_in_hand();
    let roof = v(2, y + 12, -3);
    g.sim.world.set_block_anywhere(roof, STONE);
    look_at(&mut g, roof.as_vec3() + Vec3::new(0.5, 0.0, 0.5));
    g.left.using = true;
    g.run_ticks(60);
    let up = g.body().pos.y - (y + 1) as f64;
    assert!(matches!(g.left.hook, Hook::Block { .. }) && up > 7.0, "pulled up {up}: {:?}", g.left.hook);
    let held = g.body().pos.y;
    g.run_ticks(30);
    assert!((g.body().pos.y - held).abs() < 0.2, "hangs there while G is held");
    g.body_mut().input.jump = true;
    g.run_ticks(1);
    g.body_mut().input.jump = false;
    assert_eq!(g.left.hook, Hook::Off, "jump lets go");
    g.run_ticks(90);
    assert!(g.body().pos.y < held - 3.0, "and the body falls");
}

#[test]
fn clipped_to_a_wire_the_body_rides_from_pole_to_pole_until_jump() {
    let (mut g, y) = hook_in_hand();
    let poles = [v(4, y + 1, 0), v(12, y + 1, 0), v(20, y + 1, 0)];
    for p in poles {
        g.sim.world.set_block(p, POLE);
        g.sim.factory.place(&mut g.sim.world, POLE, p, 0, p - v(0, 1, 0), 0);
    }
    g.act(crate::action::Action::Connect { pole: poles[0], to: poles[1] });
    g.act(crate::action::Action::Connect { pole: poles[1], to: poles[2] });
    g.run_ticks(2);
    let mut first = None;
    g.sim.factory.for_each_wire(Vec3::ZERO, 1e9, |a, b, _, real| {
        if real && a.x.min(b.x) < 6.0 {
            first = Some((a + b) * 0.5);
        }
    });
    let mid = first.expect("a wire between the first two poles");
    look_at(&mut g, mid);
    g.left.using = true;
    g.run_ticks(1);
    g.left.using = false;
    assert!(matches!(g.left.hook, Hook::Wire { .. }), "clipped on: {:?}", g.left.hook);
    g.run_ticks(240);
    let pos = g.body().pos;
    assert!(pos.x > 17.0, "rode on past the middle pole to the last: {pos:?}");
    // Short poles: the hanging feet brush the ground, and never sink into it.
    assert!(pos.y >= (y + 1) as f64 - 1e-6 && pos.y <= mid.y, "under the wire: {pos:?}");
    g.body_mut().input.jump = true;
    g.run_ticks(1);
    assert_eq!(g.left.hook, Hook::Off);
}
