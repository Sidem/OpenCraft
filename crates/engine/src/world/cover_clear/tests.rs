use std::collections::BTreeMap;

use super::*;
use crate::math::Vec3;
use crate::world::{Event, MeshData};

/// One drawn plant: its world column and its 16 vertex words.
type Plant = (i32, i32, [u32; 16]);
type Seen = BTreeMap<(i32, i32, i32), Vec<Plant>>;

fn all_set(c: &ClearedCover, xs: std::ops::RangeInclusive<i32>, zs: std::ops::RangeInclusive<i32>) -> bool {
    zs.into_iter().all(|z| xs.clone().all(|x| c.is_cleared(x, z)))
}

#[test]
fn a_column_is_cleared_once_and_neighbours_stay_untouched() {
    let mut c = ClearedCover::default();
    assert!(c.rows(0, 0).is_empty(), "an untouched chunk column has no data");
    assert!(c.set(5, 7) && !c.set(5, 7), "the second clear of a column changes nothing");
    assert!(c.is_cleared(5, 7) && !c.is_cleared(6, 7) && !c.is_cleared(5, 8) && !c.is_cleared(37, 7));
    // Negative columns land in chunk column -1 at local 31, 30.
    assert!(c.set(-1, -2) && c.is_cleared(-1, -2) && !c.is_cleared(31, -2));
    assert_eq!(c.rows(-1, -1)[30], 1 << 31);
    assert_eq!(c.columns.len(), 2);
}

#[test]
fn a_patch_is_three_by_three_columns_and_crosses_chunk_borders() {
    let mut w = World::new(5, 2);
    // Centred on the corner of four chunk columns: x 31|32, z 31|32.
    assert!(w.clear_cover(IVec3::new(32, 40, 32)));
    assert!(all_set(&w.cleared, 31..=33, 31..=33), "the whole patch");
    assert!(!w.cleared.is_cleared(30, 32) && !w.cleared.is_cleared(34, 32) && !w.cleared.is_cleared(32, 34));
    assert_eq!(w.cleared.columns.len(), 4, "four chunk columns hold data");
    assert!(!w.clear_cover(IVec3::new(32, 40, 32)), "the same patch again is nothing");
    assert!(w.patch_cleared(IVec3::new(32, 0, 32)) && !w.patch_cleared(IVec3::new(33, 0, 32)));
    assert!(!w.clear_cover(IVec3::new(0, -1, 0)) && !w.clear_cover(IVec3::new(0, WORLD_HEIGHT, 0)), "off the world");
}

#[test]
fn the_cleared_set_has_canonical_bytes_whatever_the_order_of_the_clears() {
    let centres = [IVec3::new(100, 30, -40), IVec3::new(-3, 30, 5), IVec3::new(31, 30, 31), IVec3::new(0, 30, 0)];
    let bytes = |order: &[usize]| {
        let mut w = World::new(5, 2);
        for &i in order {
            w.clear_cover(centres[i]);
        }
        let mut b = ByteWriter::default();
        w.cleared.write_state(&mut b);
        b.bytes
    };
    let forward = bytes(&[0, 1, 2, 3]);
    assert_eq!(forward, bytes(&[3, 2, 1, 0]));
    let back = ClearedCover::read_state(&mut ByteReader::new(&forward)).expect("reads back");
    let mut again = ByteWriter::default();
    back.write_state(&mut again);
    assert_eq!(again.bytes, forward);
    assert!(all_set(&back, 99..=101, -41..=-39) && all_set(&back, -4..=-2, 4..=6));
    // Short, unsorted and empty-column data is refused.
    for cut in [0, 3, 4, 20, forward.len() - 1] {
        assert!(ClearedCover::read_state(&mut ByteReader::new(&forward[..cut])).is_none(), "cut at {cut}");
    }
    let entry = 8 + 4 * ROWS;
    let mut unsorted = forward.clone();
    let (first, second) = (4..4 + entry, 4 + entry..4 + 2 * entry);
    let swap = unsorted[first.clone()].to_vec();
    unsorted.copy_within(second.clone(), first.start);
    unsorted[second].copy_from_slice(&swap);
    assert!(ClearedCover::read_state(&mut ByteReader::new(&unsorted)).is_none());
    let mut hollow = ByteWriter::default();
    hollow.count(1);
    hollow.i32(0);
    hollow.i32(0);
    (0..ROWS).for_each(|_| hollow.u32(0));
    assert!(ClearedCover::read_state(&mut ByteReader::new(&hollow.bytes)).is_none());
}

#[test]
fn a_world_saved_before_version_55_reads_with_no_cleared_cover() {
    let mut w = World::new(5, 2);
    w.set_block_anywhere(IVec3::new(3, 200, 3), crate::block::STONE);
    let mut now = ByteWriter::default();
    w.write_state(&mut now);
    // The old layout is the edited chunks alone: the cleared set is a count of none at the end.
    let old = &now.bytes[..now.bytes.len() - 4];
    let mut r = ByteReader::new(old);
    r.version = COVER_SAVED - 1;
    let mut back = World::new(5, 2);
    back.read_state(&mut r).expect("the old layout reads");
    assert!(r.is_done() && back.cleared == ClearedCover::default());
    assert_eq!(back.block_anywhere(IVec3::new(3, 200, 3)), Some(crate::block::STONE), "its edits came along");

    // The current layout keeps a patch through write and read.
    w.clear_cover(IVec3::new(8, 50, 9));
    let mut with = ByteWriter::default();
    w.write_state(&mut with);
    assert_ne!(with.bytes, now.bytes, "a world with a clear saves differently");
    let mut loaded = World::new(5, 2);
    loaded.read_state(&mut ByteReader::new(&with.bytes)).expect("reads");
    assert!(loaded.patch_cleared(IVec3::new(8, 50, 9)) && loaded.cleared == w.cleared);
}

fn settle(w: &mut World, seen: &mut Seen) {
    loop {
        w.begin_work();
        if !w.work_step() {
            break;
        }
    }
    while let Some(e) = w.events.pop_front() {
        if let Event::Mesh(m) = e {
            seen.insert((m.pos.x, m.pos.y, m.pos.z), plants_of(&m));
        }
    }
}

/// The plants of a mesh: the cover is added last, 16 words a plant, on faces 6 and 7 (`mesher::push_plant`).
fn plants_of(m: &MeshData) -> Vec<Plant> {
    let start = m.opaque_quads as usize * 4;
    let cutout = &m.verts[start..start + m.cutout_quads as usize * 4];
    let first = cutout.chunks(4).position(|q| matches!(q[0] >> 18 & 7, 6 | 7)).map_or(cutout.len(), |i| i * 4);
    assert_eq!((cutout.len() - first) % 16, 0, "only whole plants follow the first one");
    cutout[first..]
        .chunks_exact(16)
        .map(|p| {
            let words: [u32; 16] = p.try_into().expect("16 words");
            ((m.pos.x << 5) + (words[0] & 63) as i32, (m.pos.z << 5) + (words[0] >> 12 & 63) as i32, words)
        })
        .collect()
}

/// A world column well inside its chunk, so a patch around it stays in one chunk column.
fn inner(v: i32) -> bool {
    (2..30).contains(&(v & 31))
}

fn in_patch(centre: IVec3, (x, z, _): &Plant) -> bool {
    (x - centre.x).abs() <= PATCH_RADIUS && (z - centre.z).abs() <= PATCH_RADIUS
}

/// The plant column whose patch holds the most plants, among columns accepted by `fits`.
fn busiest_patch(seen: &Seen, fits: impl Fn(i32, i32) -> bool) -> (IVec3, usize) {
    let all: Vec<&Plant> = seen.values().flatten().collect();
    let count = |c: IVec3| all.iter().filter(|p| in_patch(c, p)).count();
    all.iter()
        .filter(|p| fits(p.0, p.1))
        .map(|p| IVec3::new(p.0, 40, p.1))
        .map(|c| (c, count(c)))
        .fold((IVec3::ZERO, 0), |best, c| if c.1 > best.1 { c } else { best })
}

/// Clears `centre`'s patch in a meshed world and checks the cover that is drawn afterwards: exactly the plants of the
/// patch are gone, every other column's plants are the very same words. Returns how many went.
fn clear_and_compare(w: &mut World, seen: &mut Seen, centre: IVec3) -> usize {
    let expected: Seen = seen
        .iter()
        .map(|(&k, plants)| (k, plants.iter().filter(|p| !in_patch(centre, p)).copied().collect()))
        .collect();
    let before: usize = seen.values().map(Vec::len).sum();
    assert!(w.clear_cover(centre));
    settle(w, seen);
    assert_eq!(*seen, expected, "the patch's plants are gone and no other plant moved");
    before - seen.values().map(Vec::len).sum::<usize>()
}

fn meadow() -> (World, Seen) {
    let mut w = World::new(1337, 3);
    let spawn = Vec3::new(0.5, w.generator().height_at(0, 0) as f64 + 1.0, 0.5);
    w.update_streaming(spawn, &[]);
    let mut seen = Seen::new();
    settle(&mut w, &mut seen);
    (w, seen)
}

#[test]
fn clearing_a_patch_removes_exactly_the_plants_of_its_nine_columns() {
    let (mut w, mut seen) = meadow();
    let total: usize = seen.values().map(Vec::len).sum();
    assert!(total > 500, "a meadow to cut: {total} plants");
    let (centre, in_it) = busiest_patch(&seen, |x, z| inner(x) && inner(z));
    assert!(in_it >= 3, "a patch with plants in it: {in_it}");
    let gone = clear_and_compare(&mut w, &mut seen, centre);
    assert_eq!(gone, in_it);
    assert_eq!(seen.values().map(Vec::len).sum::<usize>(), total - in_it);
    // Cutting again changes nothing, and neither does a patch where nothing grows.
    let waiting = w.dirty_count();
    assert!(!w.clear_cover(centre));
    assert!(w.events.is_empty() && w.dirty_count() == waiting, "no remesh for a repeat");
}

#[test]
fn a_patch_across_a_chunk_border_clears_both_sides() {
    let (mut w, mut seen) = meadow();
    let (centre, in_it) = busiest_patch(&seen, |x, z| x & 31 == 31 && inner(z));
    assert!(in_it >= 2, "a border patch with plants: {in_it}");
    let both = seen.values().flatten().filter(|p| in_patch(centre, p)).map(|p| p.0 >> 5).collect::<Vec<_>>();
    assert!(both.contains(&(centre.x >> 5)) && both.contains(&((centre.x >> 5) + 1)), "{both:?}");
    assert_eq!(clear_and_compare(&mut w, &mut seen, centre), in_it);
}

#[test]
fn chunks_that_load_later_draw_the_cleared_cover_too() {
    let (mut w, mut seen) = meadow();
    let (centre, in_it) = busiest_patch(&seen, |x, z| inner(x) && inner(z));
    assert!(in_it >= 3);
    // Out of range, the patch is cleared in the core without a chunk to remesh; coming back, the chunk meshes bare.
    w.update_streaming(Vec3::new(5000.0, 100.0, 0.5), &[]);
    settle(&mut w, &mut seen);
    assert!(w.clear_cover(centre));
    w.update_streaming(Vec3::new(0.5, 100.0, 0.5), &[]);
    let mut fresh = Seen::new();
    settle(&mut w, &mut fresh);
    let drawn: Vec<&Plant> = fresh.values().flatten().filter(|p| in_patch(centre, p)).collect();
    assert!(drawn.is_empty(), "{} plants in a cleared patch", drawn.len());
    assert!(fresh.values().map(Vec::len).sum::<usize>() > 500, "the rest of the meadow is back");
}
