use super::*;
use crate::block::{LOADING_DOCK, UNLOADING_DOCK};
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
