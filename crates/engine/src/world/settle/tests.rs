use super::*;
use crate::block::{AIR, WATER};
use crate::math::Vec3;
use crate::world::Event;

/// A streamed-in world and a cell of air high above its ground.
fn world() -> (World, IVec3) {
    let mut w = World::new(5, 2);
    w.update_streaming(Vec3::new(0.5, w.generator().height_at(0, 0) as f64 + 1.0, 0.5), &[]);
    drain(&mut w);
    (w, IVec3::new(3, 200, 3))
}

/// One host frame's streaming work; the mesh events it produced.
fn drain(w: &mut World) -> usize {
    loop {
        w.begin_work();
        if !w.work_step() {
            break;
        }
    }
    let events = std::mem::take(&mut w.events);
    events.iter().filter(|e| matches!(e, Event::Mesh(_))).count()
}

#[test]
fn a_hole_filled_again_before_it_settles_costs_nothing() {
    let (mut w, p) = world();
    assert!(w.set_block_anywhere_later(p, WATER));
    w.advance_clock(1.0);
    assert!(drain(&mut w) > 0, "a change that stays is meshed");
    let unmeshable = w.dirty_count(); // the ring's edge chunks, which lack neighbours

    assert!(w.set_block_anywhere_later(p, AIR));
    w.advance_clock(0.05);
    assert!(w.set_block_anywhere_later(p, WATER));
    w.advance_clock(1.0);
    assert_eq!(drain(&mut w), 0, "the meshes already show water there");
    assert_eq!((w.dirty_count(), w.settling_count()), (unmeshable, 0));
}

#[test]
fn a_change_that_stays_waits_then_reaches_the_meshes() {
    let (mut w, p) = world();
    assert!(w.set_block_anywhere_later(p, WATER));
    assert_eq!((drain(&mut w), w.settling_count()), (0, 1), "still waiting");
    w.advance_clock(1.0);
    assert!(drain(&mut w) > 0);
    assert_eq!(w.settling_count(), 0);
}

#[test]
fn meshing_a_chunk_passes_on_a_waiting_edit_first() {
    let (mut w, p) = world();
    assert!(w.set_block_anywhere_later(p, WATER));
    let c = chunk_of(p);
    assert!(w.remesh(c));
    assert_eq!(w.settling_count(), 0);
    let water = w.events.iter().any(|e| matches!(e, Event::Mesh(m) if m.pos == c && m.liquid_quads > 0));
    assert!(water, "the mesh shows the water the block holds");
}
