use std::collections::HashMap;

use crate::block::{is_tree, BlockId, SOLID};
use crate::chunk::{Chunk, CHUNK_MASK, CHUNK_SHIFT};
use crate::factory::INSTANCE_FLOATS;
use crate::item::TELESCOPE;
use crate::math::{IVec3, Rng, Vec3};
use crate::raycast::raycast;
use crate::worldgen::{WorldGen, WORLD_HEIGHT};
use crate::Game;

use super::beacons::push_beacon;
use super::ground::{cast, column_top, RANGE};
use super::{NO_HEIGHT, START_STEP, ZOOMS, ZOOM_MAX_RATE};

const DT: f64 = 1.0 / 60.0;

fn hold_telescope() -> Game {
    let mut g = Game::new(2024, 6);
    crate::tests::run_until_ready(&mut g);
    g.give(TELESCOPE.0, 1);
    g.update(DT);
    let slot = (0..9).find(|&i| g.slot_item(i) == TELESCOPE.0).expect("in the hotbar");
    g.select_slot(slot);
    g.update(DT);
    assert!(g.telescope_held());
    g
}

/// Right-click once: the button goes down for a tick and up again.
fn click_use(g: &mut Game) {
    g.set_using(true);
    g.run_ticks(2);
    g.set_using(false);
    g.run_ticks(1);
}

fn direction(yaw: f64, pitch: f64) -> Vec3 {
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    Vec3::new(sy * cp, sp, -cy * cp)
}

/// A random standing eye near the spawn, on the loaded ground.
fn random_eye(g: &Game, rng: &mut Rng) -> Vec3 {
    let (x, z) = (rng.range(-60.0, 60.0), rng.range(-60.0, 60.0));
    let (top, loaded) = column_top(&g.sim.world, x.floor() as i32, z.floor() as i32);
    assert!(loaded);
    Vec3::new(x, f64::from(top) + 2.6, z)
}

/// Ground truth from a world kept as it generates: the top block of a column that is solid and no tree.
struct Generated {
    gen: WorldGen,
    chunks: HashMap<IVec3, Chunk>,
}

impl Generated {
    fn new(seed: u32) -> Self {
        Generated { gen: WorldGen::new(seed), chunks: HashMap::new() }
    }

    fn block(&mut self, p: IVec3) -> BlockId {
        let c = IVec3::new(p.x >> CHUNK_SHIFT, p.y >> CHUNK_SHIFT, p.z >> CHUNK_SHIFT);
        let gen = &mut self.gen;
        let chunk = self.chunks.entry(c).or_insert_with(|| gen.generate(c));
        chunk.get((p.x & CHUNK_MASK) as usize, (p.y & CHUNK_MASK) as usize, (p.z & CHUNK_MASK) as usize)
    }

    fn ground(&mut self, p: IVec3) -> bool {
        let b = self.block(p);
        SOLID[b as usize] && !is_tree(b)
    }

    fn top(&mut self, x: i32, z: i32) -> i32 {
        (0..WORLD_HEIGHT).rev().find(|&y| self.ground(IVec3::new(x, y, z))).unwrap_or(0)
    }
}

#[test]
fn right_click_raises_the_telescope_and_left_click_pins_what_the_crosshair_meets_once() {
    let mut g = hold_telescope();
    g.set_look(0.0, -0.05);
    assert!(!g.scope_on() && g.telescope_label().unwrap().contains("Right-click"));
    click_use(&mut g);
    assert!(g.scope_on(), "one click raises it, holding the button does not flip it back");
    g.update(DT);
    let aim = g.scope.aim.expect("ground ahead of a level view");
    // Held down, the left button pins once.
    g.set_mining(true);
    g.run_ticks(6);
    assert_eq!(g.take_pin_request(), vec![aim.x, aim.y, aim.z]);
    assert!(g.take_pin_request().is_empty());
    g.set_mining(false);
    g.run_ticks(1);
    g.set_mining(true);
    g.run_ticks(1);
    assert_eq!(g.take_pin_request().len(), 3, "a new press pins again");
    g.set_mining(false);
    click_use(&mut g);
    assert!(!g.scope_on());
    g.set_mining(true);
    g.run_ticks(3);
    assert!(g.take_pin_request().is_empty(), "nothing pins with the telescope lowered");
}

#[test]
fn the_left_button_does_not_mine_while_looking_through_it() {
    let mut g = hold_telescope();
    g.set_look(0.0, -1.2);
    g.set_mining(true);
    g.run_ticks(5);
    assert!(g.mine_progress > 0.0, "lowered, it mines like any held item");
    g.set_mining(false);
    g.run_ticks(1);
    click_use(&mut g);
    g.set_mining(true);
    g.run_ticks(5);
    assert!(g.mine_block.is_none() && g.mine_progress == 0.0);
}

#[test]
fn the_wheel_zooms_only_while_looking_and_stops_at_the_ends() {
    let mut g = hold_telescope();
    assert!(!g.scope_wheel(-1), "lowered, the wheel changes the slot");
    click_use(&mut g);
    assert_eq!(g.scope.step, START_STEP);
    for _ in 0..10 {
        assert!(g.scope_wheel(-1));
    }
    assert_eq!(g.scope.step, ZOOMS.len() - 1, "wheel up zooms in, to the last step");
    for _ in 0..10 {
        g.scope_wheel(1);
    }
    assert_eq!(g.scope.step, 0);
}

#[test]
fn the_view_zooms_in_and_out_gently() {
    let mut g = hold_telescope();
    assert_eq!(g.scope_zoom(), 1.0);
    click_use(&mut g);
    g.update(-0.5); // a frame timed before the clock it started from must not panic or move the view
    assert_eq!(g.scope_zoom(), 1.0);
    let mut last = g.scope_zoom().ln();
    let mut biggest = 0.0f64;
    for frame in 0..180 {
        g.update(DT);
        let now = g.scope_zoom().ln();
        biggest = biggest.max((now - last).abs());
        last = now;
        if frame == 0 {
            assert!(now < 0.03, "the first frame moves a hair, not a jump");
        }
    }
    assert!(biggest <= f64::from(ZOOM_MAX_RATE) * DT + 1e-6, "{biggest}");
    assert!((g.scope_zoom() - f64::from(ZOOMS[START_STEP])).abs() < 1e-3, "settles on the step");
    click_use(&mut g);
    for _ in 0..240 {
        g.update(DT);
    }
    assert_eq!(g.scope_zoom(), 1.0, "and back to the normal view");
}

/// Random rays over loaded ground: the pin's column surface against what an exact walk over the blocks meets.
#[test]
fn pins_on_loaded_ground_sit_on_the_surface() {
    let g = hold_telescope();
    let world = &g.sim.world;
    let mut rng = Rng::new(11);
    let (mut rays, mut same_column, mut within_one, mut within_three, mut own_surface) = (0, 0, 0, 0, 0);
    while rays < 500 {
        let eye = random_eye(&g, &mut rng);
        let dir = direction(rng.range(0.0, std::f64::consts::TAU), rng.range(-0.6, -0.02));
        let Some(aim) = cast(world, eye, dir, RANGE).filter(|a| a.loaded) else { continue };
        // The exact answer: the first ground block the line enters, and the top of its column.
        let Some(hit) = raycast(eye, dir, 400.0, |p| world.get_block(p).filter(|&b| SOLID[b as usize] && !is_tree(b)))
        else {
            continue;
        };
        rays += 1;
        let top_of = |x: i32, z: i32| {
            (0..WORLD_HEIGHT)
                .rev()
                .find(|&y| world.get_block(IVec3::new(x, y, z)).is_some_and(|b| SOLID[b as usize] && !is_tree(b)))
        };
        own_surface += usize::from(top_of(aim.x, aim.z) == Some(aim.y));
        let truth = top_of(hit.block.x, hit.block.z).unwrap();
        same_column += usize::from((aim.x, aim.z) == (hit.block.x, hit.block.z));
        within_one += usize::from((aim.y - truth).abs() <= 1);
        within_three += usize::from((aim.y - truth).abs() <= 3);
    }
    eprintln!(
        "loaded ground, 500 rays: own column surface {own_surface}, same column as the block walk {same_column}, \
         height within 1 block of its surface {within_one}, within 3 {within_three}"
    );
    assert_eq!(own_surface, 500);
    assert!(within_one >= 475, "within 1 block: {within_one} of 500");
}

/// Random rays at ground that is not loaded: generate it afterwards and compare.
#[test]
fn pins_on_ground_not_loaded_yet_match_the_generated_surface() {
    let g = hold_telescope();
    let world = &g.sim.world;
    let mut truth = Generated::new(2024);
    let mut rng = Rng::new(12);
    let (mut rays, mut own_surface, mut within_one, mut within_three, mut walked, mut walk_within_three) =
        (0, 0, 0, 0, 0, 0);
    let (mut nearest, mut farthest, mut beyond_1000, mut beyond_3000) = (f64::MAX, 0.0f64, 0, 0);
    while rays < 500 {
        // From the ground or from a hill or tower up to 60 blocks over it, looking a little below or above the horizon.
        let mut eye = random_eye(&g, &mut rng);
        eye.y += rng.range(0.0, 60.0);
        let dir = direction(rng.range(0.0, std::f64::consts::TAU), rng.range(-0.04, 0.01));
        let Some(aim) = cast(world, eye, dir, RANGE).filter(|a| !a.loaded) else { continue };
        rays += 1;
        nearest = nearest.min(aim.dist);
        farthest = farthest.max(aim.dist);
        beyond_1000 += usize::from(aim.dist > 1000.0);
        beyond_3000 += usize::from(aim.dist > 3000.0);
        let top = truth.top(aim.x, aim.z);
        own_surface += usize::from(aim.y == top);
        within_one += usize::from((aim.y - top).abs() <= 1);
        within_three += usize::from((aim.y - top).abs() <= 3);
        // An exact walk over the generated blocks, over the last 60 blocks before the hit.
        let from = eye + dir * (aim.dist - 60.0).max(0.0);
        if truth.ground(from.floor()) {
            continue;
        }
        let hit = raycast(from, dir, 120.0, |p| {
            let b = truth.block(p);
            (SOLID[b as usize] && !is_tree(b)).then_some(b)
        });
        if let Some(hit) = hit {
            walked += 1;
            let walk_top = truth.top(hit.block.x, hit.block.z);
            walk_within_three += usize::from((aim.y - walk_top).abs() <= 3);
        }
    }
    eprintln!(
        "ground not loaded, 500 rays hitting {nearest:.0} to {farthest:.0} blocks away ({beyond_1000} beyond 1,000, \
         {beyond_3000} beyond 3,000): height equals the generated surface {own_surface}, within 1 block {within_one}, \
         within 3 {within_three}; against an exact block walk over the generated chunks, within 3 blocks: \
         {walk_within_three} of {walked}"
    );
    assert!(beyond_1000 >= 100, "the rays reach far: {beyond_1000} beyond 1,000 blocks");
    assert!(within_three >= 490, "within 3 blocks: {within_three} of 500");
    assert!(walk_within_three * 100 >= walked * 90, "{walk_within_three} of {walked}");
}

#[test]
fn a_line_of_sight_into_the_sky_or_the_ground_finds_nothing() {
    let g = hold_telescope();
    let eye = Vec3::new(0.5, 90.0, 0.5);
    assert!(cast(&g.sim.world, eye, direction(1.0, 0.4), RANGE).is_none(), "up");
    let (top, _) = column_top(&g.sim.world, 0, 0);
    let buried = Vec3::new(0.5, f64::from(top) - 3.0, 0.5);
    assert!(cast(&g.sim.world, buried, direction(1.0, 0.0), RANGE).is_none(), "from inside the ground");
}

#[test]
fn a_column_beyond_the_far_plane_is_drawn_nearer_at_the_same_angle() {
    let reach = 300.0;
    let (rel, rgb) = (Vec3::new(2400.0, -20.0, -1800.0), 0xd9a066);
    let mut out = Vec::new();
    push_beacon(&mut out, rel, reach, rgb);
    assert_eq!(out.len(), INSTANCE_FLOATS);
    let foot = Vec3::new(f64::from(out[0]), f64::from(out[1]), f64::from(out[2]))
        - Vec3::new(0.0, f64::from(out[5]) / 2.0, 0.0);
    assert!((foot.length() - reach).abs() < 0.01, "its foot stands at the reach: {}", foot.length());
    let (a, b) = (foot * (1.0 / foot.length()), rel * (1.0 / rel.length()));
    assert!((a - b).length() < 1e-4, "on the same line of sight");
    // Scaled with its distance, it covers the angle a real column would.
    let real_height =
        (super::beacons::HEIGHT_MIN + rel.length() * super::beacons::HEIGHT_PER_BLOCK).min(super::beacons::HEIGHT_MAX);
    assert!((f64::from(out[5]) / reach - real_height / rel.length()).abs() < 1e-4);
    assert!(out[15] >= 3.0, "a tinted beacon: fog does not hide it");
    // A near column is where it stands.
    let mut near = Vec::new();
    push_beacon(&mut near, Vec3::new(10.0, 0.0, 0.0), reach, rgb);
    assert!((f64::from(near[0]) - 10.0).abs() < 1e-4);
}

#[test]
fn beacons_wear_their_colour_at_full_brightness() {
    let flag = |rgb| {
        let mut out = Vec::new();
        push_beacon(&mut out, Vec3::new(5.0, 0.0, 0.0), 300.0, rgb);
        out[15] as u32 - 3
    };
    let (r, g, b) = (|k: u32| k & 15, |k: u32| k >> 4 & 15, |k: u32| k >> 8);
    let k = flag(0x2000ff);
    assert_eq!((r(k), g(k), b(k)), (2, 0, 15), "the brightest channel is full");
    let grey = flag(0x303030);
    assert_eq!((r(grey), g(grey), b(grey)), (15, 15, 15), "a dark colour becomes a bright one");
    assert_eq!(flag(0), 0xfff, "black is white");
}

#[test]
fn only_a_raised_telescope_draws_pins_and_a_pin_without_a_height_stands_on_the_ground() {
    let mut g = hold_telescope();
    let (top, _) = column_top(&g.sim.world, 12, 7);
    g.set_scope_pins(&[12, NO_HEIGHT, 7, 0xff0000, 30, 100, 30, 0x00ff00]);
    g.update(DT);
    assert_eq!(g.instance_count(), 0, "nothing shows while it is lowered");
    click_use(&mut g);
    g.update(DT);
    assert_eq!(g.instance_count(), 2);
    let eye = g.render_eye;
    let boxes = g.instances.chunks_exact(INSTANCE_FLOATS).filter(|b| b[15] >= 3.0).collect::<Vec<_>>();
    assert_eq!(boxes.len(), 2);
    let foot = |b: &[f32]| eye.y + f64::from(b[1]) - f64::from(b[5]) / 2.0;
    assert!((foot(boxes[0]) - f64::from(top + 1)).abs() < 0.01, "found its ground");
    assert!((foot(boxes[1]) - 101.0).abs() < 0.01, "kept the height it was given");
    g.set_scope_pins(&[]);
    g.update(DT);
    assert_eq!(g.instance_count(), 0);
}

#[test]
fn at_most_the_limit_of_pins_are_kept() {
    let mut g = hold_telescope();
    let flat: Vec<i32> = (0..300).flat_map(|i| [i, 70, 0, 0xffffff]).collect();
    g.set_scope_pins(&flat);
    assert_eq!(g.scope.pins.len(), g.scope_pin_limit() as usize);
}

/// How long 200 pins at 60 far-off places cost a frame while looking, and what the aim costs.
#[test]
#[ignore]
fn bench_scope_frame() {
    use std::time::Instant;
    let mut g = hold_telescope();
    click_use(&mut g);
    let mut rng = Rng::new(5);
    let mut flat = Vec::new();
    for i in 0..200 {
        let far = if i % 3 == 0 { 40.0 } else { 3000.0 };
        let (x, z) = (rng.range(-far, far) as i32, rng.range(-far, far) as i32);
        flat.extend([x, if i % 2 == 0 { NO_HEIGHT } else { 70 }, z, 0xff8800]);
    }
    g.set_scope_pins(&flat);
    g.set_look(0.3, -0.01);
    let frames = 300;
    let timed = |g: &mut Game| {
        let t = Instant::now();
        for _ in 0..frames {
            g.update(DT);
        }
        t.elapsed().as_secs_f64() * 1e6 / f64::from(frames)
    };
    let both = timed(&mut g);
    g.set_scope_pins(&[]);
    let aim_only = timed(&mut g);
    click_use(&mut g);
    for _ in 0..60 {
        g.update(DT);
    }
    let lowered = timed(&mut g);
    g.set_scope_pins(&flat);
    click_use(&mut g);
    g.set_look(0.3, -0.4);
    let near_aim = timed(&mut g);
    let beacons_only = {
        let t = Instant::now();
        for _ in 0..frames {
            g.instances.clear();
            g.write_scope_beacons(g.render_eye);
        }
        t.elapsed().as_secs_f64() * 1e6 / f64::from(frames)
    };
    // The aim at its worst: lines of sight that run the whole range or end far away, from up to 60 blocks over the ground.
    let (mut total, mut worst, mut far_hits) = (0.0f64, 0.0f64, 0);
    for _ in 0..200 {
        let mut eye = random_eye(&g, &mut rng);
        eye.y += rng.range(0.0, 60.0);
        let dir = direction(rng.range(0.0, std::f64::consts::TAU), rng.range(-0.01, 0.01));
        let t = Instant::now();
        let aim = cast(&g.sim.world, eye, dir, RANGE);
        let us = t.elapsed().as_secs_f64() * 1e6;
        (total, worst) = (total + us, worst.max(us));
        far_hits += usize::from(aim.is_some_and(|a| a.dist > 1000.0));
    }
    eprintln!(
        "per frame (µs): level view with 200 pins {both:.0}, level view without pins {aim_only:.0}, lowered {lowered:.0}, \
         aiming at near ground with 200 pins {near_aim:.0}; writing the 200 columns alone {beacons_only:.0}; the aim over \
         200 near-level lines of sight ({far_hits} ending beyond 1,000 blocks): mean {:.0}, worst {worst:.0}",
        total / 200.0
    );
}
