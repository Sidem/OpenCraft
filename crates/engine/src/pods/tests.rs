//! The pods in flight: sending, how many fit, falling, and the bytes.

use super::*;
use crate::item::{IRON_PLATE, SCREW};

fn cargo() -> Vec<Stack> {
    vec![Stack { item: IRON_PLATE, count: 40 }, Stack { item: SCREW, count: 7 }]
}

#[test]
fn a_pod_lands_a_flight_after_it_leaves_and_at_most_sixty_four_are_up() {
    let mut pods = Pods::default();
    pods.send(IVec3::new(1, 2, 3), IVec3::new(9, 2, 9), 2, cargo(), 100);
    assert_eq!((pods.list[0].launched, pods.list[0].lands), (100, 100 + FLIGHT_TICKS));
    assert_eq!(pods.inbound(2), 1);
    assert_eq!(pods.inbound(0), 0);
    for _ in 0..MAX_IN_FLIGHT * 2 {
        pods.send(IVec3::ZERO, IVec3::ZERO, 0, cargo(), 0);
    }
    assert_eq!((pods.list.len(), pods.has_room()), (MAX_IN_FLIGHT, false));
}

#[test]
fn a_pod_falls_only_in_its_last_stretch_and_lower_each_moment() {
    let mut pods = Pods::default();
    pods.send(IVec3::ZERO, IVec3::new(5, 0, 5), 0, cargo(), 0);
    assert!(pods.falling(0.0).is_empty());
    let start = FLIGHT_TICKS as f64 - FALL_TICKS as f64;
    let heights: Vec<f64> = [start, start + 100.0, FLIGHT_TICKS as f64 - 1.0, FLIGHT_TICKS as f64, 5000.0]
        .iter()
        .map(|&t| pods.falling(t)[0].1)
        .collect();
    assert!((heights[0] - FALL_HEIGHT).abs() < 1e-9);
    assert!(heights.windows(2).all(|w| w[0] >= w[1]), "{heights:?}");
    assert_eq!(heights[3], 0.0, "on the pad at touchdown, and waiting there if it cannot land");
}

#[test]
fn pods_round_trip_through_bytes_and_damage_is_refused() {
    let mut pods = Pods::default();
    pods.send(IVec3::new(1, 2, 3), IVec3::new(9, 2, 9), 3, cargo(), 77);
    pods.send(IVec3::new(-4, 5, 6), IVec3::new(8, 8, 8), 0, Vec::new(), 90);
    let mut w = ByteWriter::default();
    pods.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let back = Pods::read_state(&mut r).unwrap();
    assert_eq!(r.u8(), None, "every byte read");
    assert_eq!(back.list.len(), 2);
    assert_eq!(
        (back.list[0].from, back.list[0].to, back.list[0].channel),
        (IVec3::new(1, 2, 3), IVec3::new(9, 2, 9), 3)
    );
    assert_eq!((back.list[0].launched, back.list[0].lands), (77, 77 + FLIGHT_TICKS));
    assert_eq!(back.list[0].cargo, cargo());
    assert!(back.list[1].cargo.is_empty());
    let mut count = ByteWriter::default();
    count.count(MAX_IN_FLIGHT + 1);
    assert!(Pods::read_state(&mut ByteReader::new(&count.bytes)).is_none(), "too many pods is damage");
    assert!(Pods::read_state(&mut ByteReader::new(&w.bytes[..w.bytes.len() - 3])).is_none(), "a cut file is damage");
}
