use super::*;
use crate::item::SURVEY_SATELLITE;
use crate::math::IVec3;

/// Calls for two whole sweeps, so one that starts from the new centre ends whatever the sweep in progress was.
fn sweep_all(seen: &mut Seen, g: &WorldGen, sats: &Satellites, tick: u64, centre: (i32, i32)) {
    for _ in 0..2 * ((2 * SPAN + 1).pow(2) / PER_CALL + 2) {
        seen.step(g, sats, tick, centre);
    }
}

#[test]
fn nothing_is_found_while_no_survey_satellite_is_up() {
    let g = WorldGen::new(2024);
    let mut seen = Seen::default();
    sweep_all(&mut seen, &g, &Satellites::default(), 0, (0, 0));
    assert_eq!(seen.deposits().count(), 0);
}

#[test]
fn a_sweep_finds_veins_and_lodes_under_the_footprint_and_only_there() {
    let g = WorldGen::new(2024);
    let mut sats = Satellites::default();
    sats.launch(SURVEY_SATELLITE, IVec3::new(0, 70, 0), 0);
    let mut seen = Seen::default();
    sweep_all(&mut seen, &g, &sats, 0, (0, 0));
    let found: Vec<_> = seen.deposits().copied().collect();
    assert!(found.len() > 20, "{} found", found.len());
    assert!(found.iter().all(|d| d.tier() != Tier::Outcrop));
    // Every find lies within a column's diagonal of the footprint's edge (a column counts by its middle).
    let reach = crate::satellites::KINDS[SURVEY as usize].reach + 2 * CHUNK_SIZE;
    assert!(found.iter().all(|d| d.center.x.pow(2) + d.center.z.pow(2) <= reach * reach));
    // And the footprint's ground is not left half-swept: the nearest seeded vein is among the finds.
    let near = g.find_deposit(IVec3::new(0, 70, 0), Tier::Vein, 8).expect("a vein near spawn");
    assert!(seen.deposits().any(|d| d.key == near.key), "{:?}", near.center);
}

#[test]
fn finds_are_kept_until_they_leave_the_swept_square() {
    let g = WorldGen::new(2024);
    let mut sats = Satellites::default();
    sats.launch(SURVEY_SATELLITE, IVec3::new(0, 70, 0), 0);
    let mut seen = Seen::default();
    sweep_all(&mut seen, &g, &sats, 0, (0, 0));
    let (n, changes) = (seen.deposits().count(), seen.changes);
    // The satellite flies on (an hour later it is elsewhere), the player stays: nothing is forgotten.
    sweep_all(&mut seen, &g, &sats, 216_000, (0, 0));
    assert!(seen.deposits().count() >= n);
    assert!(seen.changes >= changes);
    // The player moves a long way off: the old square's finds are dropped once the new sweep ends.
    let far = (SPAN + KEEP + 40) * CHUNK_SIZE * 2;
    sweep_all(&mut seen, &g, &sats, 216_000, (far, far));
    assert!(seen.deposits().all(|d| (d.key.cx - (far >> CHUNK_SHIFT)).abs() <= SPAN + KEEP));
}
