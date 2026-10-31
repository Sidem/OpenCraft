use super::*;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

#[test]
fn a_first_heading_lands_on_an_axis_or_a_diagonal_and_nothing_between() {
    for yaw in 0..=255u8 {
        let h = i32::from(first_heading(yaw));
        assert_eq!(h % COMPASS, 0, "heading {yaw} gave {h}");
        assert!(turn(i32::from(yaw), h) <= AXIS_STEPS.max(COMPASS / 2), "heading {yaw} moved too far to {h}");
    }
    assert_eq!(first_heading(64), 64);
    assert_eq!(first_heading(64 + 17), 64, "17 steps off east: east");
    assert_eq!(first_heading(255), 0, "just west of north wraps to north");
    assert_eq!(first_heading(32), 32, "a diagonal looked at squarely stays one");
    assert_eq!(first_heading(64 - 24), 32, "24 steps off east is nearer the diagonal");
}

#[test]
fn a_chord_near_an_axis_lies_on_it_and_the_node_takes_the_axis() {
    let a = v(10, 5, 10);
    // East, 12 blocks, 2 off to the south (9.5 degrees): onto the axis, facing east.
    let s = snap_chord(a, 64, v(22, 5, 12)).expect("snaps");
    assert_eq!((s.pos, s.yaw), (v(22, 5, 10), Some(64)));
    // From a node facing a little off the axis the chord still goes onto the axis, and the node faces it true.
    let s = snap_chord(a, 70, v(22, 5, 12)).expect("snaps");
    assert_eq!((s.pos, s.yaw), (v(22, 5, 10), Some(64)));
    // North and west work too, and the height is the aim's.
    assert_eq!(snap_chord(a, 0, v(9, 8, -3)).map(|s| (s.pos, s.yaw)), Some((v(10, 8, -3), Some(0))));
    assert_eq!(snap_chord(a, 192, v(-4, 5, 11)).map(|s| (s.pos, s.yaw)), Some((v(-4, 5, 10), Some(192))));
}

#[test]
fn a_chord_near_the_nodes_own_heading_goes_straight_on_without_forcing_a_heading() {
    let a = v(0, 0, 0);
    // Heading 32 (north-east): a chord a few degrees off it goes onto its line.
    let s = snap_chord(a, 32, v(11, 0, -9)).expect("snaps");
    assert_eq!(s.yaw, None, "the heading follows the chord: straight");
    assert_eq!((s.pos.x, s.pos.z), (10, -10), "on the diagonal");
    // Facing backwards along the line counts too (a track leaves either side).
    assert_eq!(snap_chord(a, 32, v(-9, 0, 11)).map(|s| (s.pos.x, s.pos.z)), Some((-10, 10)));
}

#[test]
fn a_bend_is_left_alone() {
    let a = v(0, 0, 0);
    assert_eq!(snap_chord(a, 64, v(12, 0, 8)), None, "34 degrees off east");
    assert_eq!(snap_chord(a, 64, v(12, 0, 4)), None, "18 degrees off east");
    assert_eq!(snap_chord(a, 64, v(1, 0, 0)), None, "too short to read");
}

#[test]
fn snapping_never_moves_a_node_further_than_the_chord_was_long() {
    let a = v(3, 0, -7);
    for ya in (0..=255u8).step_by(5) {
        for x in -30..=30 {
            for z in -30..=30 {
                let aim = v(a.x + x, 0, a.z + z);
                let Some(s) = snap_chord(a, ya, aim) else { continue };
                let run = |p: IVec3| f64::from(p.x - a.x).hypot(f64::from(p.z - a.z));
                assert!(run(s.pos) <= run(aim) + 1e-9, "{ya} {aim:?} -> {:?}", s.pos);
                assert!(run(s.pos) >= 1.0, "{ya} {aim:?} -> {:?} stands on the anchor", s.pos);
            }
        }
    }
}
