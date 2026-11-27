//! Tests for the read-only factory beat survey (`factory/beat.rs`).

use super::super::beat::{
    family_of, BEAT_FLOATS, BEAT_RADIUS, FAMILY_ASSEMBLY, FAMILY_CHEMISTRY, FAMILY_COUNT, FAMILY_MINING, FAMILY_POWER,
    FAMILY_REFINING, FAMILY_RESEARCH, FAMILY_SHAPING, FAMILY_SMELTING, FAMILY_STRIDE,
};
use super::super::*;
use crate::block::{
    ASSEMBLER, BELT, CHEMICAL_PLANT, COAL_ORE, CONSTRUCTOR, CRUSHER, GENERATOR, IRON_ORE, LAB, MINER, POLE, SMELTER,
    SOLAR_PANEL, STORAGE,
};
use crate::math::{IVec3, Vec3};
use crate::world::World;

#[test]
fn families_cover_active_machines_and_skip_passive_blocks() {
    assert_eq!(family_of(GENERATOR), Some(FAMILY_POWER));
    assert_eq!(family_of(MINER), Some(FAMILY_MINING));
    assert_eq!(family_of(SMELTER), Some(FAMILY_SMELTING));
    assert_eq!(family_of(CRUSHER), Some(FAMILY_REFINING));
    assert_eq!(family_of(CONSTRUCTOR), Some(FAMILY_SHAPING));
    assert_eq!(family_of(ASSEMBLER), Some(FAMILY_ASSEMBLY));
    assert_eq!(family_of(CHEMICAL_PLANT), Some(FAMILY_CHEMISTRY));
    assert_eq!(family_of(LAB), Some(FAMILY_RESEARCH));
    for passive in [BELT, STORAGE, POLE, SOLAR_PANEL] {
        assert_eq!(family_of(passive), None, "passive block {passive} should stay silent");
    }
}

#[test]
fn beat_near_enforces_hard_distance_cap_and_tracks_health() {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let eye = Vec3::new(0.5, 65.5, 0.5);

    // Two smelters near the camera: one actively working, one starved (NoInput).
    let near_a = IVec3::new(3, 65, 0);
    let near_b = IVec3::new(5, 65, 0);
    // One smelter outside the hard distance cap from both the origin and near_b.
    let far = IVec3::new(BEAT_RADIUS as i32 + 14, 65, 0);
    for pos in [near_a, near_b, far] {
        f.place(&mut world, SMELTER, pos, 0, pos - IVec3::new(0, 1, 0), 0);
    }
    // Feed ore + fuel to near_a and far so both start working; leave near_b starved.
    for pos in [near_a, far] {
        let p = f.processors.iter_mut().find(|p| p.pos == pos).unwrap();
        p.input.add(ItemId::block(IRON_ORE), 4);
        p.fuel.add(ItemId::block(COAL_ORE), 4);
    }
    let mut events = Vec::new();
    f.update(&mut world, 1, &mut events);

    let data = f.beat_near(eye);
    assert_eq!(data.len(), BEAT_FLOATS);
    let s = &data[FAMILY_SMELTING * FAMILY_STRIDE..(FAMILY_SMELTING + 1) * FAMILY_STRIDE];
    assert!(s[0] > 0.5, "working smelter near camera has strong presence: {}", s[0]);
    assert!((s[1] - 3.0).abs() < 0.05, "nearest working distance is 3 blocks: {}", s[1]);
    assert!((s[2] - 3.0).abs() < 0.05, "relative x points to +3: {}", s[2]);
    assert!((s[4] - 0.5).abs() < 1e-4, "1 working + 1 starved gives 0.5 health: {}", s[4]);

    // Move well past BEAT_RADIUS from all smelters: complete silence across every family.
    let distant = f.beat_near(Vec3::new(200.0, 65.5, 200.0));
    for fam in 0..FAMILY_COUNT {
        assert_eq!(distant[fam * FAMILY_STRIDE], 0.0, "family {fam} must be silent past BEAT_RADIUS");
    }

    // Move near `far` only: it is working alone with 1.0 health.
    let at_far = f.beat_near(Vec3::new(f64::from(far.x) + 0.5, 65.5, 0.5));
    let sf = &at_far[FAMILY_SMELTING * FAMILY_STRIDE..(FAMILY_SMELTING + 1) * FAMILY_STRIDE];
    assert!(sf[0] > 0.7, "working smelter right at eye has high strength: {}", sf[0]);
    assert!((sf[4] - 1.0).abs() < 1e-4, "healthy lone smelter has health 1.0: {}", sf[4]);
}

#[test]
fn switched_off_machine_does_not_penalize_health() {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let eye = Vec3::new(0.5, 65.5, 0.5);
    let a = IVec3::new(2, 65, 0);
    let b = IVec3::new(4, 65, 0);
    for pos in [a, b] {
        f.place(&mut world, SMELTER, pos, 0, pos - IVec3::new(0, 1, 0), 0);
    }
    let pa = f.processors.iter_mut().find(|p| p.pos == a).unwrap();
    pa.input.add(ItemId::block(IRON_ORE), 4);
    pa.fuel.add(ItemId::block(COAL_ORE), 4);
    f.set_enabled(b, false);
    let mut events = Vec::new();
    f.update(&mut world, 1, &mut events);

    let data = f.beat_near(eye);
    let s = &data[FAMILY_SMELTING * FAMILY_STRIDE..(FAMILY_SMELTING + 1) * FAMILY_STRIDE];
    assert!((s[4] - 1.0).abs() < 1e-4, "switched-off machine should not count as stalled: {}", s[4]);
}
