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
    w.update_streaming(spawn);
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
    w.update_streaming(Vec3::new(0.5, 100.0, 0.5));
    drain(&mut w);
    let p = IVec3::new(3, 200, 3);
    assert!(w.set_block(p, STONE));
    w.update_streaming(Vec3::new(5000.0, 100.0, 0.5));
    let (_, unloads) = drain(&mut w);
    assert!(unloads > 0);
    assert_eq!(w.get_block(p), None);
    w.update_streaming(Vec3::new(0.5, 100.0, 0.5));
    drain(&mut w);
    assert_eq!(w.get_block(p), Some(STONE));
}
