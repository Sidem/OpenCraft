//! Frame cost of a real save, piece by piece (ignored; needs a save):
//!   OC_SAVE=path\to\world.ocworld cargo test --release -q bench_save_frame -- --ignored --nocapture
//! Times each presentation step `Game::update` runs after the ticks, so a slow frame can be pinned on one of them.
//! `cone_keeps_every_box_on_screen` (same save) checks the view cone culls nothing on screen.
//! `bench_port_view_and_labels` needs no save: the worst cases of the placement ghost, port marks and box labels
//!   cargo test --release -q bench_port_view_and_labels -- --ignored --nocapture

use std::time::Instant;

use super::run_until_ready;
use crate::factory;
use crate::Game;

fn ms<T>(name: &str, runs: u32, mut f: impl FnMut() -> T) {
    let t = Instant::now();
    for _ in 0..runs {
        std::hint::black_box(f());
    }
    println!("{name:34} {:8.3} ms", t.elapsed().as_secs_f64() * 1000.0 / f64::from(runs));
}

#[test]
#[ignore]
fn bench_save_frame() {
    let Ok(path) = std::env::var("OC_SAVE") else { panic!("set OC_SAVE to a .ocworld file") };
    let bytes = std::fs::read(path).expect("save");
    if bytes.starts_with(&[0x1f, 0x8b]) {
        panic!("unpack the save first: this bench reads raw save bytes");
    }
    let mut g = Game::from_save(&bytes, 8).expect("load");
    run_until_ready(&mut g);
    // Warm up static box lighting & wire routing like a settled frame.
    for _ in 0..80 {
        g.update(1.0 / 60.0);
        g.begin_work();
        while g.work_step() {}
        while g.next_event() != 0 {}
        while g.next_static_box_event() != 0 {}
    }
    println!(
        "dynamic instances {} static {} chunks {}",
        g.instances.len() / factory::INSTANCE_FLOATS,
        g.static_box_total(),
        g.sim.world.loaded_count()
    );
    let eye = g.render_eye;
    let time = g.sim.tick as f64 / 60.0;
    let range = g.sim.world.view_distance();
    ms("update(1/60) whole", 20, || g.update(1.0 / 60.0));
    let mut evs = 0;
    while g.next_static_box_event() != 0 {
        evs += 1;
    }
    println!("  -> static events over 20 steady frames: {evs}");
    ms("wire_props.step", 20, || g.wire_props.step(&g.sim.world, &g.sim.factory, eye));
    let (strat, chg) = (g.strategy.active(), g.minimap.atlas.changes as usize);
    ms("static_boxes.update", 20, || {
        let atlas = &g.minimap.atlas;
        g.static_boxes.update(&g.sim.factory, &g.wire_props, &mut g.sim.world, eye, strat, chg, |cx, cz| {
            atlas.tile(cx, cz).is_some()
        });
    });
    ms("stream_around_players", 10, || g.stream_around_players());
    ms("write_item_instances", 10, || g.write_item_instances(1.0 / 60.0, eye));
    ms("write_drone_instances", 10, || g.write_drone_instances(eye, 0.5));
    let mut out = Vec::new();
    ms("factory.write_dynamic_instances", 20, || {
        out.clear();
        g.sim.factory.write_dynamic_instances(
            &g.static_boxes.index,
            &mut out,
            eye,
            time,
            range,
            &crate::view_cone::ViewCone::ALL,
        )
    });
    println!("  -> {} dynamic factory boxes", out.len() / factory::INSTANCE_FLOATS);
    ms("light_boxes (dynamic, warm)", 20, || {
        let mut reader = crate::world::LightReader::default();
        factory::light_boxes(&mut out, eye, |c| reader.read(&mut g.sim.world, c))
    });
    let mut round = Vec::new();
    ms("split_round (dynamic)", 20, || factory::split_round(&out, &mut round));
    let mut full = Vec::new();
    ms("factory.write_instances (ref)", 10, || {
        full.clear();
        g.sim.factory.write_instances(&mut full, eye, time, range)
    });
    println!("  -> {} full factory boxes", full.len() / factory::INSTANCE_FLOATS);
    ms("hide_unexplored_instances", 10, || g.hide_unexplored_instances(eye));
    ms("write_strategy_fog", 10, || g.write_strategy_fog(eye));
    ms("refresh_some", 10, || g.minimap.atlas.refresh_some(&g.sim.world));
    ms("catch_up", 10, || g.catch_up());
    ms("write_line_preview", 10, || g.write_line_preview(eye, time));
}

#[test]
#[ignore]
fn bench_port_view_and_labels() {
    use crate::factory::{box_label, port_marks};
    use crate::math::{IVec3, Vec3};
    let (eye, time) = (Vec3::new(0.5, 66.0, 8.0), 3.0);
    let (mut solid, mut ghost) = (Vec::new(), Vec::new());
    // The ghost of the machine with the most ports, and the marks of 30 machines built near, a belt in hand.
    let most = (1..=255u16).max_by_key(|&b| port_marks::port_faces(b, IVec3::ZERO, 0).len()).unwrap();
    ms("ghost + marks, most ports", 1000, || {
        solid.clear();
        ghost.clear();
        port_marks::draw_ghost(most, IVec3::new(0, 64, 0), 1, eye, time, |_| true, &mut solid, &mut ghost)
    });
    println!(
        "  -> {} solid, {} see-through",
        solid.len() / factory::INSTANCE_FLOATS,
        ghost.len() / factory::INSTANCE_FLOATS
    );
    let mut world = crate::world::World::new(1, 2);
    let mut f = factory::Factory::default();
    let many: Vec<u16> = (1..=255u16).filter(|&b| !port_marks::port_faces(b, IVec3::ZERO, 0).is_empty()).collect();
    for i in 0..30 {
        let at = IVec3::new((i % 6) * 6 - 15, 64, (i / 6) * 6 - 15);
        f.place(&mut world, many[i as usize % many.len()], at, (i % 4) as u8, at, 0);
    }
    ms("port marks of 30 machines", 1000, || {
        solid.clear();
        ghost.clear();
        f.write_port_marks(eye, time, |_| true, &mut solid, &mut ghost)
    });
    // 100 full boxes of mixed items, each seen from a corner (two labelled sides).
    let slots: Vec<_> =
        (0..48).map(|i| crate::inventory::Stack { item: crate::item::ItemId(256 + i % 7), count: 30 }).collect();
    ms("100 box labels (48 mixed slots)", 1000, || {
        solid.clear();
        for i in 0..100 {
            let item = box_label::shown(&slots, box_label::AUTO).unwrap();
            box_label::draw(&mut solid, item, Vec3::new(3.0 + f64::from(i % 10), 0.0, -3.0 - f64::from(i / 10)));
        }
    });
    println!("  -> {} boxes", solid.len() / factory::INSTANCE_FLOATS);
}

/// The view cone never drops a dynamic box that is on screen (needs a save with a factory):
///   OC_SAVE=path\to\world.ocworld cargo test --release -q cone_keeps_every -- --ignored --nocapture
/// Projects every unculled box through the host's camera (`render/mat4.ts`) for 16 views; a model part reaching past
/// `MODEL_REACH` (a quarry's drill once did) fails here.
#[test]
#[ignore]
fn cone_keeps_every_box_on_screen() {
    use crate::math::Vec3;
    use crate::view_cone::ViewCone;
    let bytes = std::fs::read(std::env::var("OC_SAVE").expect("set OC_SAVE")).expect("save");
    let mut g = Game::from_save(&bytes, 8).expect("load");
    run_until_ready(&mut g);
    let (eye, range) = (g.body().eye(), g.sim.world.view_distance());
    let (fov, aspect) = (100f64.to_radians(), 16.0 / 9.0);
    let t = (fov * 0.5).tan();
    let key = |r: &[f32]| [r[0].to_bits(), r[1].to_bits(), r[2].to_bits()];
    let (mut checked, mut culled) = (0, 0);
    for k in 0..16 {
        let (yaw, pitch) = (k as f64 * 0.785, [-1.2, -0.5, 0.0, 0.6][k % 4]);
        let (mut all, mut cut) = (Vec::new(), Vec::new());
        let cone = ViewCone::new(yaw, pitch, fov, aspect);
        g.sim.factory.write_dynamic_instances(&g.static_boxes.index, &mut all, eye, 0.0, range, &ViewCone::ALL);
        g.sim.factory.write_dynamic_instances(&g.static_boxes.index, &mut cut, eye, 0.0, range, &cone);
        let kept: std::collections::HashSet<[u32; 3]> = cut.chunks_exact(factory::INSTANCE_FLOATS).map(key).collect();
        let ((sy, cy), (sp, cp)) = (yaw.sin_cos(), pitch.sin_cos());
        let (f, r, u) = (Vec3::new(sy * cp, sp, -cy * cp), Vec3::new(cy, 0.0, sy), Vec3::new(-sy * sp, cp, cy * sp));
        for rec in all.chunks_exact(factory::INSTANCE_FLOATS) {
            let p = Vec3::new(rec[0] as f64, rec[1] as f64, rec[2] as f64);
            let z = p.dot(f);
            if z <= 0.05 {
                continue;
            }
            let margin = 0.9 * f64::from(rec[4].max(rec[5]).max(rec[6])) / (z * t); // the box's own size
            if (p.dot(r) / (z * t * aspect)).abs() <= 1.0 + margin && (p.dot(u) / (z * t)).abs() <= 1.0 + margin {
                checked += 1;
                assert!(kept.contains(&key(rec)), "on screen but culled: {:?} (yaw {yaw}, pitch {pitch})", p + eye);
            }
        }
        culled += (all.len() - cut.len()) / factory::INSTANCE_FLOATS;
    }
    println!("{checked} boxes on screen, all kept; {culled} culled over 16 views");
}
