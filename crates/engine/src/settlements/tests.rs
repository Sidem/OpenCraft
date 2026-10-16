use super::*;
use crate::action::Action;
use crate::block::{AIR, TOWN_DEPOT};
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::ItemId;
use crate::sim::{PlayerId, Sim};

const SEED: u32 = 2024;
const P: PlayerId = PlayerId(0);

/// Columns on a grid in square rings round spawn, nearest ring first, from `SPAWN_CLEAR` out to 2,400 blocks.
fn rings() -> impl Iterator<Item = (i32, i32)> {
    const STEP: i32 = 24;
    (SPAWN_CLEAR / STEP..=2400 / STEP).flat_map(|r| {
        let side = 2 * r + 1;
        (0..side * side)
            .map(move |k| (k % side - r, k / side - r))
            .filter(move |c| c.0.abs().max(c.1.abs()) == r)
            .map(|(i, j)| (i * STEP, j * STEP))
    })
}

/// The first column (nearest ring first) where `want` holds, as the cell above its ground.
fn find(gen: &WorldGen, want: impl Fn(i32, i32) -> bool) -> IVec3 {
    let (x, z) = rings().find(|&(x, z)| want(x, z)).expect("found within 2,400 blocks");
    IVec3::new(x, gen.height_at(x, z) + 1, z)
}

fn biome(gen: &WorldGen, x: i32, z: i32) -> Biome {
    gen.biome_at(x, z, gen.height_at(x, z))
}

#[test]
fn every_kind_has_sites_and_a_site_takes_the_first_kind_that_fits() {
    let gen = WorldGen::new(SEED);
    let towns = Settlements::default();
    for (k, kind) in KINDS.iter().enumerate() {
        // Where the kind's own place is, the site is that kind or an earlier row (water comes first).
        let place = find(&gen, |x, z| match kind.needs {
            Needs::Water(w) => gen.open_water(x, z) == Some(w),
            Needs::Biome(b) => biome(&gen, x, z) == b,
        });
        let got = towns.site(&gen, place).expect("a kind's place fits");
        assert!(got as usize <= k, "{}: {got}", kind.name);
        // And somewhere it is exactly this kind.
        find(&gen, |x, z| towns.site(&gen, IVec3::new(x, 0, z)) == Ok(k as u8));
    }
}

#[test]
fn a_dry_plain_fits_no_town() {
    let gen = WorldGen::new(SEED);
    let towns = Settlements::default();
    let dry = |x: i32, z: i32| (-12..=12).step_by(4).all(|d| gen.open_water(x + d, z).is_none());
    let plain = find(&gen, |x, z| biome(&gen, x, z) == Biome::Plains && dry(x, z) && dry(x, z + 8) && dry(x, z - 8));
    assert_eq!(towns.site(&gen, plain), Err(Unfit::NoKind));
    assert!(Unfit::NoKind.text().contains("the sea or a river"));
}

#[test]
fn towns_keep_apart_and_off_the_spawn_plains() {
    let gen = WorldGen::new(SEED);
    let mut towns = Settlements::default();
    assert_eq!(towns.site(&gen, IVec3::new(120, 70, -90)), Err(Unfit::NearSpawn(150)));
    let harbour = find(&gen, |x, z| gen.open_water(x, z) == Some(Water::Sea));
    towns.found(harbour, 0);
    // Further out along x, so further from spawn too.
    let out = |d: i32| harbour + IVec3::new(d * harbour.x.signum().max(1), 0, 0);
    assert_eq!(towns.site(&gen, out(100)), Err(Unfit::NearTown(towns.towns[0], 100)));
    assert!(towns.site(&gen, out(SPACING - 1)).is_err());
    assert!(!matches!(towns.site(&gen, out(SPACING)), Err(Unfit::NearTown(..))));
    assert!(Unfit::NearTown(towns.towns[0], 100).text().contains("your harbour is 100 blocks away"));
}

#[test]
fn a_depot_founds_a_town_only_at_a_fitting_site_and_breaking_it_ends_the_town() {
    let mut sim = Sim::new(SEED, 2);
    let gen = WorldGen::new(SEED);
    let depot = ItemId::block(TOWN_DEPOT);
    sim.apply(P, Action::Give { item: depot, count: 2 });
    let place = |sim: &mut Sim, pos: IVec3| {
        sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing: 0, against: pos - IVec3::new(0, 1, 0) })
    };
    // In the air over the spawn plains: the cells are free, but no town fits.
    let near = IVec3::new(60, 200, 60);
    place(&mut sim, near);
    assert_eq!(sim.world.block_anywhere_or_generate(near), AIR);
    assert_eq!(sim.player(P).unwrap().inventory.count(depot), 2, "nothing was used");

    let shore = find(&gen, |x, z| gen.open_water(x, z) == Some(Water::Sea));
    let at = IVec3::new(shore.x, 200, shore.z);
    place(&mut sim, at);
    assert_eq!(sim.world.block_anywhere_or_generate(at), TOWN_DEPOT);
    assert_eq!(sim.settlements.towns, vec![Town { pos: at, kind: 0 }]);
    // A second depot too near the first is refused.
    place(&mut sim, at + IVec3::new(40, 0, 0));
    assert_eq!(sim.settlements.towns.len(), 1);
    assert_eq!(sim.player(P).unwrap().inventory.count(depot), 1);

    // The town survives a save.
    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut back = Sim::new(SEED, 2);
    back.read_state(&mut ByteReader::new(&w.bytes)).expect("reads");
    assert_eq!(back.settlements.towns, sim.settlements.towns);
    assert_eq!(back.state_hash(), sim.state_hash());

    let part = *sim.factory.footprint_at(at).expect("a footprint").2.last().unwrap();
    sim.apply(P, Action::BreakBlock { pos: part });
    assert_eq!(sim.world.block_anywhere_or_generate(at), AIR);
    assert!(sim.settlements.towns.is_empty(), "breaking the depot ends its town");
}

#[test]
fn a_save_from_before_towns_loads_with_none_and_a_bad_kind_is_refused() {
    let sim = Sim::new(SEED, 2);
    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut empty = ByteWriter::default();
    Settlements::default().write_state(&mut empty);
    // Save 47 ended with the opening scene: cut the empty town list off the end.
    let old = &w.bytes[..w.bytes.len() - empty.bytes.len()];
    let mut back = Sim::new(SEED, 2);
    let mut r = ByteReader::new(old);
    r.version = 47;
    back.read_state(&mut r).expect("a save 47 loads");
    assert!(back.settlements.towns.is_empty());
    assert_eq!(r.u8(), None, "every byte read");

    let mut bad = ByteWriter::default();
    Settlements { towns: vec![Town { pos: IVec3::new(400, 70, 0), kind: KINDS.len() as u8 }] }.write_state(&mut bad);
    assert!(Settlements::read_state(&mut ByteReader::new(&bad.bytes)).is_none());
}
