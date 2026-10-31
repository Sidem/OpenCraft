use super::*;
use crate::block::{LOADING_DOCK, STONE, UNLOADING_DOCK};
use crate::bytes::{ByteReader, ByteWriter};

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

const E: u8 = 64;

/// Rail nodes (cell, heading) with tracks between the listed pairs.
fn nodes(list: &[(IVec3, u8)], pairs: &[(usize, usize)]) -> Factory {
    let mut f = Factory::default();
    for &(c, yaw) in list {
        f.place(&mut World::new(1, 2), RAIL, c, yaw, c, 0);
    }
    for &(a, b) in pairs {
        f.lay_track(list[a].0, list[b].0);
    }
    assert_eq!(f.tracks.len(), pairs.len(), "every track fits");
    f
}

#[test]
fn a_dock_fits_only_a_plain_straight_node() {
    let a = v(0, 0, 0);
    let (b, c, d) = (v(12, 0, 0), v(24, 0, 0), v(24, 0, 8));
    let mut f = nodes(&[(a, E), (b, E), (c, E), (d, E), (v(0, 0, 20), 32)], &[(0, 1), (1, 2)]);
    assert_eq!(f.dock_fit(a), DockFit::Ok, "an end of the line");
    assert_eq!(f.dock_fit(b), DockFit::Ok, "a node in the middle");
    assert_eq!(f.dock_fit(v(5, 0, 0)), DockFit::NoNode);
    assert_eq!(f.dock_fit(v(0, 0, 20)), DockFit::Diagonal);
    // A second track leaving b's east end makes it a junction.
    f.lay_track(b, d);
    assert_eq!(f.dock_fit(b), DockFit::Junction);
    assert!(f.set_signal(c, true));
    assert_eq!(f.dock_fit(c), DockFit::Signal);
    assert!(f.place_train(a));
    assert_eq!(f.dock_fit(a), DockFit::Train);
}

#[test]
fn a_dock_keeps_the_node_heading_and_tracks_and_gives_one_track_each_end() {
    let (a, b, c, d) = (v(0, 0, 0), v(12, 0, 0), v(24, 0, 0), v(24, 0, 8));
    let mut f = nodes(&[(a, E), (b, E), (c, E), (d, E)], &[(0, 1), (1, 2)]);
    f.swap_rail_piece(&mut World::new(1, 2), b, UNLOADING_DOCK);
    assert_eq!(f.dock_piece(b).map(|i| f.processors[i].dir), Some(1));
    assert_eq!(f.rail_yaw(b), Some(E));
    assert!(f.track_between(a, b) && f.track_between(b, c));
    assert_eq!(f.track_fit(b, d), Some(super::super::Fit::DockEnd), "its east end has a track");
    f.swap_rail_piece(&mut World::new(1, 2), b, RAIL);
    assert_eq!(f.dock_piece(b), None);
    assert_eq!(f.rail_yaw(b), Some(E));
    assert!(f.track_between(a, b) && f.track_between(b, c));
}

/// Height of the stone floor the splitting tests lie on (above any generated ground); nodes stand at `FLOOR + 1`.
const FLOOR: i32 = 200;

/// A world with clear air over a stone floor at `FLOOR`.
fn floor() -> World {
    let mut w = World::new(1, 2);
    for x in -4..=40 {
        for z in -4..=12 {
            for y in FLOOR..=FLOOR + 8 {
                w.set_block_anywhere_later(v(x, y, z), if y == FLOOR { STONE } else { AIR });
            }
        }
    }
    w
}

#[test]
fn a_straight_track_is_cut_where_aimed_between_its_nodes_and_no_other_track_is() {
    let y = FLOOR + 1;
    let (a, b, d, e) = (v(0, y, 0), v(12, y, 0), v(12, y, 8), v(0, y, 8));
    let mut f = nodes(&[(a, E), (b, E), (d, 32), (e, 32)], &[(0, 1)]);
    let through = |f: &Factory, c: IVec3| f.tracks_through(c).collect::<Vec<_>>();
    assert_eq!(through(&f, v(5, y, 0))[0].1, [v(5, y, 0), v(5, y, 0)], "a level track: one cell");
    assert!(through(&f, v(5, y, 1)).is_empty(), "beside the track");
    assert!(through(&f, a).is_empty() && through(&f, b).is_empty(), "not on a node");
    assert!(through(&f, v(5, y + 3, 0)).is_empty(), "too far above it");
    f.lay_track(d, e);
    assert!(through(&f, v(6, y, 8)).is_empty(), "a diagonal track is not cut");
}

#[test]
fn on_a_ramp_the_dock_may_stand_in_either_cell_the_track_passes_between() {
    let y = FLOOR + 1;
    let (a, b) = (v(0, y, 0), v(10, y + 3, 0));
    let f = nodes(&[(a, E), (b, E)], &[(0, 1)]);
    let (t, spots) = f.tracks_through(v(5, y + 1, 0)).next().expect("on the ramp");
    assert_eq!(spots, [v(5, y + 1, 0), v(5, y + 2, 0)], "1.5 up: between two cells");
    let mut w = floor();
    // 1.5 up in 5 blocks leaves 1.5 up in the other 5: one half is steeper than 1 in 3 whichever cell it takes.
    for p in spots {
        for h in y..=y + 3 {
            w.set_block_anywhere(v(5, h, 0), if h < p.y { STONE } else { AIR });
        }
        assert_eq!(f.split_fit(t, p, |c| w.block_anywhere_or_generate(c)), Err(SplitWhy::Fit(Fit::TooSteep)));
    }
    assert_eq!(f.tracks_through(v(4, y, 0)).next().map(|s| s.1), Some([v(4, y + 1, 0), v(4, y + 2, 0)]), "1.2 up");
}

#[test]
fn splitting_a_track_keeps_both_halves_and_the_core_refuses_what_does_not_fit() {
    let y = FLOOR + 1;
    let (a, b, mid) = (v(0, y, 0), v(12, y, 0), v(6, y, 0));
    let mut w = floor();
    let mut f = nodes(&[(a, E), (b, E)], &[(0, 1)]);
    let (t, _) = f.tracks_through(mid).next().expect("straight");
    let fit = |f: &Factory, w: &mut World, p: IVec3| f.split_fit(t, p, |c| w.block_anywhere_or_generate(c));
    assert_eq!(fit(&f, &mut w, mid), Ok(()));
    assert_eq!(fit(&f, &mut w, v(1, y, 0)), Err(SplitWhy::Fit(Fit::TooClose)));
    assert_eq!(fit(&f, &mut w, v(6, y + 2, 0)), Err(SplitWhy::Spot), "in the air");
    w.set_block_anywhere(v(6, FLOOR, 0), AIR);
    assert_eq!(fit(&f, &mut w, mid), Err(SplitWhy::Spot), "no ground under it");
    w.set_block_anywhere(v(6, FLOOR, 0), STONE);
    f.split_track(&mut w, t, mid, LOADING_DOCK);
    assert!(f.track_between(a, mid) && f.track_between(mid, b) && !f.track_between(a, b));
    assert_eq!((f.rail_yaw(mid), f.track_count(mid)), (Some(E), 2), "a rail piece with a track at each end");
    assert!(f.dock_piece(mid).is_some());
    assert!(f.end_taken(mid, E, v(3, y, 0)) && f.end_taken(mid, E, v(9, y, 0)), "both its ends hold their track");
    // Cutting it again, next to the dock, leaves a stretch too short.
    let (t, _) = f.tracks_through(v(8, y, 0)).next().expect("the east half");
    assert_eq!((t.a, t.b), (mid, b));
    assert_eq!(f.split_fit(t, v(8, y, 0), |c| w.block_anywhere_or_generate(c)), Err(SplitWhy::Fit(Fit::TooClose)));
    // A train on the track stops the cut.
    let mut g = nodes(&[(a, E), (b, E)], &[(0, 1)]);
    assert!(g.place_train(a));
    let (t, _) = g.tracks_through(mid).next().expect("straight");
    assert_eq!(g.split_fit(t, mid, |c| w.block_anywhere_or_generate(c)), Err(SplitWhy::Train));
}

#[test]
fn an_old_dock_loses_its_2x2_cells_but_keeps_its_cargo() {
    let at = v(10, 70, 10);
    let mut f = nodes(&[(at, E)], &[]);
    f.swap_rail_piece(&mut World::new(1, 2), at, LOADING_DOCK);
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let old = Footprint { size: [2, 2, 1], ports: &[] };
    let extra: Vec<IVec3> = old.cells(at, 1).into_iter().skip(1).collect();
    assert_eq!(extra.len(), 3);
    let load = |version: u32| {
        let mut world = World::new(1, 2);
        for &c in &extra {
            world.set_block_anywhere(c, MACHINE_PART);
        }
        let mut r = ByteReader::new(&w.bytes);
        r.version = version;
        let f = Factory::read_state(&mut world, &mut r).unwrap();
        assert!(f.dock_piece(at).is_some(), "the dock stays, at its anchor");
        extra.iter().map(|&c| world.block_anywhere_or_generate(c)).collect::<Vec<_>>()
    };
    assert_eq!(load(63), [AIR; 3], "an old dock's other cells are cleared");
    assert_eq!(load(64), [MACHINE_PART; 3], "a new save is left as it was written");
}
