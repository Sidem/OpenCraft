use super::*;

fn drain(w: &mut World) -> (usize, usize) {
    let (mut meshes, mut unloads) = (0, 0);
    loop {
        w.begin_work();
        if !w.work_step() {
            break;
        }
    }
    while let Some(e) = w.events.pop_front() {
        match e {
            Event::Mesh(_) => meshes += 1,
            Event::Unload(_) => unloads += 1,
        }
    }
    (meshes, unloads)
}

#[test]
fn streams_meshes_and_edits() {
    let mut w = World::new(5, 2);
    let spawn = Vec3::new(0.5, w.generator().height_at(0, 0) as f64 + 1.0, 0.5);
    w.update_streaming(spawn, &[]);
    let (meshes, _) = drain(&mut w);
    assert!(meshes > 0);
    assert!(w.area_ready(1));

    // Break the block under the player: its chunk (and maybe neighbours) must remesh right away.
    let below = spawn.floor() - IVec3::new(0, 1, 0);
    assert_ne!(w.get_block(below), Some(AIR));
    assert!(w.set_block(below, AIR));
    assert_eq!(w.get_block(below), Some(AIR));
    assert!(!w.events.is_empty());
    assert!(!w.set_block(below, AIR), "no-op edits are ignored");
}

#[test]
fn edits_survive_unload() {
    let mut w = World::new(9, 2);
    w.update_streaming(Vec3::new(0.5, 100.0, 0.5), &[]);
    drain(&mut w);
    let p = IVec3::new(3, 200, 3);
    assert!(w.set_block(p, STONE));
    w.update_streaming(Vec3::new(5000.0, 100.0, 0.5), &[]);
    let (_, unloads) = drain(&mut w);
    assert!(unloads > 0);
    assert_eq!(w.get_block(p), None);
    w.update_streaming(Vec3::new(0.5, 100.0, 0.5), &[]);
    drain(&mut w);
    assert_eq!(w.get_block(p), Some(STONE));
}

#[test]
fn rewriting_a_block_is_not_an_edit_anywhere() {
    let mut w = World::new(9, 2);
    let p = IVec3::new(700, 200, 700);
    assert!(!w.set_block_anywhere(p, AIR), "already air");
    assert_eq!(w.block_anywhere(p), None, "no stored copy was made");
    assert!(w.set_block_anywhere(p, STONE));
    assert!(!w.set_block_anywhere(p, STONE));
    assert_eq!(w.block_anywhere(p), Some(STONE));
}

/// Remeshes every loaded chunk of a seed-1337 world around spawn and prints the time per chunk
/// (`cargo test --release -q bench_meshing -- --ignored --nocapture`).
#[test]
#[ignore]
fn bench_meshing() {
    let mut w = World::new(1337, 4);
    let spawn = Vec3::new(0.5, w.generator().height_at(0, 0) as f64 + 1.0, 0.5);
    w.update_streaming(spawn, &[]);
    drain(&mut w);
    let all: Vec<IVec3> = w.chunks.keys().copied().collect();
    let start = std::time::Instant::now();
    let mut meshed = 0;
    for _ in 0..3 {
        for &p in &all {
            meshed += usize::from(w.remesh(p));
        }
    }
    let per = start.elapsed().as_secs_f64() * 1000.0 / meshed as f64;
    let with_faces = w.events.iter().filter(|e| matches!(e, Event::Mesh(m) if !m.verts.is_empty())).count() / 3;
    println!("{meshed} remeshes of {} chunks ({with_faces} with faces): {per:.3} ms per chunk", all.len());
}
