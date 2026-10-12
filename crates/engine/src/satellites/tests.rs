//! Satellites: positions that depend on the tick alone, coverage that comes and goes, the cap per kind, saving, and the
//! directions the sky draws.

use super::*;
use crate::item::IRON_PLATE;

const PAD: IVec3 = IVec3::new(0, 70, 0);

fn up(pad: IVec3, tick: u64) -> Satellites {
    let mut s = Satellites::default();
    s.launch(SURVEY_SATELLITE, pad, tick);
    s
}

/// Ticks after which the first heading is back at the same x (96 mm a tick; the sky is 4,096,000 mm across).
const LAP: u64 = 128_000;

#[test]
fn a_satellite_starts_straight_above_its_pad_and_flies_off() {
    let s = up(IVec3::new(100, 70, -300), 600);
    let sat = s.list()[0];
    assert_eq!(sat.position(600), [100_000, (WRAP_MM - 300_000)]);
    assert_eq!(sat.position(660), [100_000 + 96 * 60, WRAP_MM - 300_000 + 28 * 60]);
    assert!(sat.covers(100, -300, 600));
    assert!(!sat.covers(2100, -300, 600), "two thousand blocks away is outside the footprint");
}

#[test]
fn a_place_is_covered_for_part_of_each_lap_and_the_sky_repeats() {
    let s = up(PAD, 0);
    // The track passes (1000, 292) after 10,417 ticks; sample the whole lap.
    let covered: Vec<bool> = (0..LAP).step_by(60).map(|t| s.covering(0, 1000, 300, t) > 0).collect();
    let on = covered.iter().filter(|&&c| c).count();
    assert!(on > 0 && on < covered.len(), "covered {on} of {} seconds", covered.len());
    // One lap on, the sky is where it was in x; the place wraps with it.
    let sat = s.list()[0];
    assert_eq!(sat.position(0)[0], sat.position(LAP)[0]);
    assert_eq!(sat.covers(0, 0, 0), sat.covers(WRAP, WRAP, 0), "a place and the one a wrap away share a sky");
}

#[test]
fn a_position_is_a_pure_function_of_the_tick() {
    let (a, b) = (up(PAD, 5), up(PAD, 5));
    for t in [5, 6, 1000, 999_999, 40_000_000] {
        assert_eq!(a.list()[0].position(t), b.list()[0].position(t));
        let p = a.list()[0].position(t);
        assert!(p.iter().all(|&v| (0..WRAP_MM).contains(&v)), "{p:?} stays on the torus");
    }
}

#[test]
fn successive_launches_fly_different_headings() {
    let mut s = Satellites::default();
    for t in 0..3 {
        s.launch(SURVEY_SATELLITE, PAD, t);
    }
    let headings: Vec<u8> = s.list().iter().map(|x| x.heading).collect();
    assert_eq!(headings, [0, 1, 2]);
}

#[test]
fn only_a_satellite_item_goes_up_and_a_full_kind_retires_its_oldest() {
    let mut s = Satellites::default();
    s.launch(IRON_PLATE, PAD, 0);
    assert!(s.list().is_empty(), "plates are not satellites");
    for t in 0..MAX_PER_KIND as u64 + 3 {
        s.launch(SURVEY_SATELLITE, PAD, t + 1);
    }
    assert_eq!(s.count(0), MAX_PER_KIND);
    assert_eq!(s.list()[0].launched, 4, "the first three burned up");
    assert_eq!(s.launched as usize, MAX_PER_KIND + 3, "the launch count never goes down");
}

#[test]
fn satellites_survive_a_save_and_damage_is_refused() {
    let mut s = up(PAD, 7);
    s.launch(SURVEY_SATELLITE, IVec3::new(-50, 70, 90), 99);
    let mut w = ByteWriter::default();
    s.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let back = Satellites::read_state(&mut r).unwrap();
    assert!(r.is_done());
    assert_eq!((back.list(), back.launched), (s.list(), s.launched));

    let mut bad = w.bytes.clone();
    bad[8] = 9; // the first satellite's kind
    assert!(Satellites::read_state(&mut ByteReader::new(&bad)).is_none());
    assert!(Satellites::read_state(&mut ByteReader::new(&w.bytes[..w.bytes.len() - 1])).is_none());
}

#[test]
fn the_sky_shows_what_is_above_the_horizon() {
    let s = up(PAD, 0);
    let overhead = s.sky_directions(0.0, 0.0, 0);
    assert_eq!(overhead.len(), 4);
    assert!((overhead[1] - 1.0).abs() < 1e-6 && overhead[0].abs() < 1e-6, "straight up: {overhead:?}");
    let near = s.sky_directions(500.0, 0.0, 0);
    assert!(near[0] < 0.0 && near[1] > 0.0, "a satellite west of the viewer shows to the west and above");
    assert!(s.sky_directions(2048.0, 2048.0, 0).is_empty(), "the far corner of the wrapped world is too far to see");
}
