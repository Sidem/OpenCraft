use super::*;
use crate::block::{AIR, DIRT, GRASS, WATER};
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
    drain_keep(w);
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

/// One host frame's streaming work; the mesh events it produced for chunk `c` (earlier events dropped).
fn meshed(w: &mut World, c: IVec3) -> usize {
    w.events.clear();
    drain_keep(w);
    w.events.iter().filter(|e| matches!(e, Event::Mesh(m) if m.pos == c)).count()
}

fn drain_keep(w: &mut World) {
    loop {
        w.begin_work();
        if !w.work_step() {
            break;
        }
    }
}

#[test]
fn growth_in_a_chunk_shares_one_remesh_after_the_wait() {
    let (mut w, p) = world();
    let (c, q) = (chunk_of(p), p + IVec3::new(1, 0, 0));
    for cell in [p, q] {
        w.set_block_anywhere(cell, DIRT);
    }
    w.advance_clock(1.0);
    drain(&mut w);
    assert!(w.grow_block_anywhere(p, GRASS), "the block changes at once");
    assert_eq!(w.get_block(p), Some(GRASS));
    assert_eq!(meshed(&mut w, c), 0, "the chunk waits");
    w.advance_clock(GROWTH_WAIT / 2.0);
    assert!(w.grow_block_anywhere(q, GRASS));
    assert_eq!(meshed(&mut w, c), 0, "still waiting");
    w.advance_clock(GROWTH_WAIT / 2.0 + 0.01);
    assert_eq!(meshed(&mut w, c), 1, "both growths, one remesh, timed from the first");
    w.advance_clock(GROWTH_WAIT * 2.0);
    assert_eq!(meshed(&mut w, c), 0, "nothing left over");
}

#[test]
fn a_player_edit_meshes_a_growth_held_chunk_at_once() {
    let (mut w, p) = world();
    let (c, q) = (chunk_of(p), p + IVec3::new(1, 0, 0));
    // Dirt to grass changes no light, which would mark the chunk again for its own reasons.
    for cell in [p, q] {
        w.set_block_anywhere(cell, DIRT);
    }
    w.advance_clock(1.0);
    drain(&mut w);
    assert!(w.grow_block_anywhere(p, GRASS));
    assert!(w.set_block_anywhere(q, GRASS), "an immediate edit remeshes now");
    assert!(w.events.iter().any(|e| matches!(e, Event::Mesh(m) if m.pos == c)));
    w.advance_clock(GROWTH_WAIT * 2.0);
    assert_eq!(meshed(&mut w, c), 0, "that mesh already showed the growth");
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
