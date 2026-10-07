use super::*;
use crate::block::{LAMP, STONE};
use crate::math::Vec3;

fn streamed_world() -> (World, IVec3) {
    let mut w = World::new(5, 2);
    let ground = w.generator().height_at(0, 0);
    w.update_streaming(Vec3::new(0.5, ground as f64 + 1.0, 0.5), &[]);
    loop {
        w.begin_work();
        if !w.work_step() {
            break;
        }
    }
    (w, IVec3::new(0, ground + 3, 0))
}

fn blocks_of(light: u8) -> (u8, u8) {
    (light & 15, light >> 4)
}

fn settle(w: &mut World) {
    w.begin_work();
    while w.work_step() {}
}

#[test]
fn open_air_is_daylit_and_a_lamp_lights_it_when_the_work_runs() {
    let (mut w, air) = streamed_world();
    assert_eq!(blocks_of(w.light_at(air)), (15, 0));

    // The lit chunk is cached; the edit makes it stale: it reads as before until the work step lights it again.
    assert!(w.set_block(air + IVec3::new(2, 0, 0), LAMP));
    assert_eq!(blocks_of(w.light_at(air)).1, 0, "still the old light");
    settle(&mut w);
    assert_eq!(blocks_of(w.light_at(air)).1, 15, "two blocks from a lamp is full block light");
    let far = air + IVec3::new(0, 0, 25);
    assert!(blocks_of(w.light_at(far)).1 < 15, "and it fades with distance");

    // A roof over the cell takes the sky light away.
    for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        w.set_block(air + IVec3::new(dx, 1, dz), STONE);
    }
    settle(&mut w);
    assert!(blocks_of(w.light_at(air)).0 < 15);
}

#[test]
fn an_edit_that_changes_no_light_leaves_the_cache_alone() {
    let (mut w, air) = streamed_world();
    w.light_at(air);
    w.set_block(air, STONE);
    settle(&mut w);
    w.light_at(air);
    // Stone to dirt: same light behaviour, so nothing is stale and nothing is dirty.
    settle(&mut w);
    w.dirty.clear();
    w.set_block(air, crate::block::DIRT);
    assert!(w.light_cache.iter().all(|e| !e.stale));
    assert_eq!(w.dirty_count(), 0, "the chunk itself is meshed at once, no other waits");
}

#[test]
fn a_block_that_only_stops_the_sky_dirties_fewer_chunks_than_a_lamp() {
    let (mut w, air) = streamed_world();
    let high = air + IVec3::new(0, 20, 0);
    w.set_block(high, crate::block::LEAVES);
    w.dirty.clear();
    w.set_block(high, crate::block::AIR);
    let leaf = w.dirty_count();
    w.set_block(high, LAMP);
    assert!(leaf < w.dirty_count(), "{leaf} chunks for a leaf, {} for a lamp", w.dirty_count());
}

#[test]
fn a_big_spread_of_boxes_is_lit_once_not_every_frame() {
    // A factory in view spans dozens of chunks; reading them all again must not light any chunk again
    // (a cache smaller than the spread was lighting every chunk each frame: 45 ms a frame).
    let (mut w, _) = streamed_world();
    let cells: Vec<IVec3> = (-1..=1)
        .flat_map(|x| (-1..=1).flat_map(move |z| (0..6).map(move |y| IVec3::new(x * 32 + 5, y * 32 + 5, z * 32 + 5))))
        .collect();
    let lit = |w: &World| w.light_cache.iter().filter_map(|e| e.lit.as_ref().map(|b| b.as_ptr())).collect::<Vec<_>>();
    cells.iter().for_each(|&c| _ = w.light_at(c));
    let first = lit(&w);
    assert!(first.len() > 24, "only {} chunks lit", first.len());
    cells.iter().for_each(|&c| _ = w.light_at(c));
    assert!(lit(&w) == first, "the same lit chunks, none lit again");
}

#[test]
fn the_cache_evicts_the_chunk_read_longest_ago() {
    let (mut w, _) = streamed_world();
    let (keep, spare) = (IVec3::new(5, 69, 5), IVec3::new(37, 69, 5));
    w.light_at(keep);
    w.light_at(spare);
    for i in 0..LIGHT_CHUNKS as i32 {
        w.light_at(keep); // read again and again: never the stalest
        w.light_at(IVec3::new(5, 69, 5 + 32 * (i + 1)));
    }
    let has = |c: IVec3| w.light_cache.iter().any(|e| e.chunk == chunk_of(c));
    assert!(has(keep) && !has(spare));
    assert!(w.light_cache.len() <= LIGHT_CHUNKS);
}

#[test]
fn an_unlit_place_reads_as_plain_daylight() {
    let (mut w, air) = streamed_world();
    let unloaded = air + IVec3::new(4000, 0, 0);
    assert_eq!(w.light_at(unloaded), DAYLIGHT);
    assert_eq!(w.light_at(IVec3::new(0, -5, 0)), DAYLIGHT);
    assert_eq!(w.light_at(IVec3::new(0, WORLD_HEIGHT + 5, 0)), DAYLIGHT);
}
