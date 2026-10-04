use super::*;
use crate::block::RAIL;
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::render::INSTANCE_FLOATS;
use crate::world::World;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// North, east, south, west as heading bytes.
const N: u8 = 0;
const E: u8 = 64;

fn nodes(list: &[(IVec3, u8)]) -> Factory {
    let mut f = Factory::default();
    for &(c, yaw) in list {
        f.place(&mut World::new(1, 2), RAIL, c, yaw, c, 0);
    }
    f.relink();
    f
}

#[test]
fn nodes_join_when_the_span_grade_and_bend_fit() {
    // Straight east along the heading, 12 blocks.
    let mut f = nodes(&[(v(0, 0, 0), E), (v(12, 0, 0), E), (v(40, 0, 0), E), (v(1, 0, 0), E), (v(12, 6, 0), E)]);
    assert_eq!(f.track_fit(v(0, 0, 0), v(12, 0, 0)), Some(Fit::Ok));
    assert_eq!(f.track_fit(v(0, 0, 0), v(40, 0, 0)), Some(Fit::TooFar));
    assert_eq!(f.track_fit(v(0, 0, 0), v(1, 0, 0)), Some(Fit::TooClose));
    assert_eq!(f.track_fit(v(12, 0, 0), v(12, 6, 0)), Some(Fit::TooClose), "straight above is no track");
    assert_eq!(f.track_fit(v(0, 0, 0), v(5, 0, 5)), None, "no node there");
    f.lay_track(v(0, 0, 0), v(12, 0, 0));
    f.relink();
    assert_eq!(f.track_fit(v(12, 0, 0), v(0, 0, 0)), Some(Fit::Joined), "either way round");
    f.lay_track(v(0, 0, 0), v(40, 0, 0));
    assert_eq!(f.tracks().len(), 1, "a pair that does not fit is quietly refused");
}

#[test]
fn grade_and_bend_limits() {
    // 12 blocks east: a 4-block rise is a third; 5 is too steep.
    let f = nodes(&[(v(0, 0, 0), E), (v(12, 4, 0), E), (v(12, 5, 0), E), (v(12, 0, 0), N)]);
    assert_eq!(f.track_fit(v(0, 0, 0), v(12, 4, 0)), Some(Fit::Ok));
    assert_eq!(f.track_fit(v(0, 0, 0), v(12, 5, 0)), Some(Fit::TooSteep));
    // A node facing north cannot take a track from the west: its heading is 90 degrees off the chord.
    assert_eq!(f.track_fit(v(0, 0, 0), v(12, 0, 0)), Some(Fit::TooSharp));
    assert_eq!(fit(v(0, 0, 0), E, v(12, 0, 5), E), Fit::Ok, "a gentle S bend");
    assert_eq!(fit(v(0, 0, 0), E, v(6, 0, 6), E), Fit::TooSharp, "45 degrees in 8 blocks is under the radius");
    assert_eq!(fit(v(0, 0, 0), E, v(16, 0, 16), 128), Fit::Ok, "an arc to a node facing the chord's mirror");
}

#[test]
fn a_node_holds_four_tracks() {
    let mut list = vec![(v(0, 0, 0), E)];
    list.extend((1..=5).map(|i| (v(10, 0, i * 3 - 9), E)));
    let mut f = nodes(&list);
    for i in 1..=5 {
        f.lay_track(v(0, 0, 0), v(10, 0, i * 3 - 9));
    }
    f.relink();
    assert_eq!(f.tracks().len(), MAX_LINKS as usize);
    assert!(f.describe(v(0, 0, 0)).unwrap().contains("4 of 4 tracks"));
    assert_eq!(f.track_fit(v(0, 0, 0), v(10, 0, 6)), Some(Fit::Full));
}

#[test]
fn removing_a_node_drops_its_tracks() {
    let mut f = nodes(&[(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 0, 0), E)]);
    f.lay_track(v(0, 0, 0), v(12, 0, 0));
    f.lay_track(v(12, 0, 0), v(24, 0, 0));
    f.relink();
    assert!(f.describe(v(12, 0, 0)).unwrap().contains("2 of 4"));
    assert!(f.describe(v(0, 0, 0)).unwrap().contains("End of the track"));
    f.remove(v(12, 0, 0));
    f.relink();
    assert!(f.tracks().is_empty());
    assert!(f.describe(v(0, 0, 0)).unwrap().contains("join it to another node"));
    // Cutting by hand.
    let mut f = nodes(&[(v(0, 0, 0), E), (v(12, 0, 0), E)]);
    f.lay_track(v(0, 0, 0), v(12, 0, 0));
    f.cut_track(v(12, 0, 0), v(0, 0, 0));
    assert!(f.tracks().is_empty());
}

#[test]
fn connect_and_disconnect_actions_lay_and_cut_track_between_nodes() {
    let mut f = nodes(&[(v(0, 0, 0), E), (v(12, 0, 0), E)]);
    f.connect(v(0, 0, 0), v(12, 0, 0));
    assert!(f.track_between(v(0, 0, 0), v(12, 0, 0)));
    f.disconnect(v(0, 0, 0), v(12, 0, 0));
    assert!(!f.track_between(v(0, 0, 0), v(12, 0, 0)));
}

#[test]
fn a_straight_track_is_straight_and_a_bend_follows_the_headings() {
    let straight = Curve::new(v(0, 0, 0), E, v(12, 0, 0), E);
    assert!((straight.length() - 12.0).abs() < 1e-6);
    assert!((straight.at(0.5).z - 0.5).abs() < 1e-9 && (straight.at(0.5).x - 6.5).abs() < 1e-9);
    // Leaves along the heading, flipped to point the way of travel: a node facing west still takes an eastward track.
    let west = Curve::new(v(0, 0, 0), 192, v(12, 0, 0), 192);
    assert!((west.length() - 12.0).abs() < 1e-6);
    // An arc turning a quarter circle of radius 16: north-facing start, east-facing end.
    let bend = Curve::new(v(0, 0, 0), N, v(16, 0, -16), E);
    let len = bend.length();
    assert!((len - 16.0 * std::f64::consts::FRAC_PI_2).abs() < 1.0, "length {len} is about a quarter circle");
    let (hx, hz) = bend.heading(0.0);
    assert!(hx.abs() < 1e-9 && hz < -0.99, "leaves heading north");
    let (hx, hz) = bend.heading(1.0);
    assert!(hx > 0.99 && hz.abs() < 1e-9, "arrives heading east");
    // The height rises evenly.
    let ramp = Curve::new(v(0, 0, 0), E, v(12, 4, 0), E);
    assert!((ramp.at(0.5).y - 2.5).abs() < 1e-9);
}

#[test]
fn headings_round_trip() {
    for yaw in [0u8, 1, 37, 64, 100, 128, 200, 255] {
        let (x, z) = dir_of(yaw);
        assert_eq!(yaw_of(x, z), yaw);
    }
    let (x, z) = dir_of(E);
    assert!((x - 1.0).abs() < 1e-9 && z.abs() < 1e-9, "64 faces east");
}

#[test]
fn nodes_and_tracks_save_and_load() {
    let mut f = nodes(&[(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 3, 4), 70)]);
    f.lay_track(v(0, 0, 0), v(12, 0, 0));
    f.lay_track(v(12, 0, 0), v(24, 3, 4));
    f.relink();
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).expect("reads back");
    back.relink();
    assert_eq!(back.tracks(), f.tracks());
    assert_eq!(back.rail_yaw(v(24, 3, 4)), Some(70));
    assert!(back.describe(v(12, 0, 0)).unwrap().contains("2 of 4"));
    // A track whose end is gone is dropped when the world loads.
    let mut lone = nodes(&[(v(0, 0, 0), E)]);
    lone.tracks.push(Track { a: v(0, 0, 0), b: v(12, 0, 0) });
    lone.prune_tracks();
    assert!(lone.tracks().is_empty());
}

#[test]
fn tracks_and_nodes_draw_whole_boxes_and_tilt_with_the_grade() {
    let (mut flat, mut ramp, mut node) = (Vec::new(), Vec::new(), Vec::new());
    write_track(&mut flat, &Curve::new(v(0, 0, 0), E, v(12, 0, 0), E), Vec3::ZERO, 1.0e9);
    write_track(&mut ramp, &Curve::new(v(0, 0, 0), E, v(12, 4, 0), E), Vec3::ZERO, 1.0e9);
    write_node(&mut node, Vec3::ZERO, E, true);
    assert_eq!(flat.len() % INSTANCE_FLOATS, 0);
    assert!(ramp.len() >= flat.len(), "a slope is a little longer");
    assert!(flat.chunks_exact(INSTANCE_FLOATS).all(|b| b[12] == 0.0), "level track is not tilted");
    assert!(
        ramp.chunks_exact(INSTANCE_FLOATS).all(|b| b[12] > 0.2 && b[12] < 0.4),
        "a third climbs at about 18 degrees"
    );
    assert_eq!(node.chunks_exact(INSTANCE_FLOATS).count(), 1 + 3, "a plate, and the stub of a bare node");
    // Pieces beyond the range are not drawn.
    let mut far = Vec::new();
    write_track(&mut far, &Curve::new(v(0, 0, 0), E, v(12, 0, 0), E), Vec3::new(500.0, 0.0, 0.0), 20.0);
    assert!(far.is_empty());
}

#[test]
fn the_rail_block_is_a_thin_walk_through_machine() {
    assert!(crate::block::def(RAIL).placeable && !crate::block::SOLID[RAIL as usize]);
}
