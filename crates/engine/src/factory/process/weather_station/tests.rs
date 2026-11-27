//! The weather station: measuring while its pole has power, and only then, for `RADIUS` blocks; its status line says
//! which; a switched-off one measures nothing.

use super::*;
use crate::block::{COAL_ORE, GENERATOR, POLE};
use crate::factory::Factory;
use crate::math::IVec3;
use crate::world::World;

const MAST: IVec3 = IVec3::new(4, 3, 0);

fn put(f: &mut Factory, block: crate::block::BlockId, pos: IVec3) {
    f.place(&mut World::new(1, 2), block, pos, 0, pos, 0);
}

fn tick(f: &mut Factory, n: u64) {
    let (mut world, mut events) = (World::new(1, 2), Vec::new());
    for t in 0..n {
        f.update(&mut world, t, &mut events);
    }
}

fn station(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.spec.block == WEATHER_STATION).unwrap()
}

#[test]
fn it_measures_its_radius_only_on_power() {
    let mut f = Factory::default();
    put(&mut f, POLE, IVec3::new(2, 3, 0));
    put(&mut f, WEATHER_STATION, MAST);
    tick(&mut f, 3);
    assert_eq!(f.weather_stations().collect::<Vec<_>>(), [(MAST, false)]);
    assert!(!f.weather_covers(MAST.x, MAST.z));
    assert!(station(&f).status_text().contains("No power"), "{}", station(&f).status_text());

    put(&mut f, GENERATOR, IVec3::new(3, 3, 3));
    assert_eq!(f.insert(IVec3::new(3, 3, 3), COAL_ORE.into(), 64), 64);
    tick(&mut f, 3);
    assert_eq!(f.weather_stations().collect::<Vec<_>>(), [(MAST, true)]);
    assert!(station(&f).status_text().contains("Measuring"), "{}", station(&f).status_text());
    assert!(f.weather_covers(MAST.x + RADIUS, MAST.z) && f.weather_covers(MAST.x, MAST.z - RADIUS));
    assert!(!f.weather_covers(MAST.x + RADIUS, MAST.z + 1), "the edge is a circle");
    assert!(!f.weather_covers(MAST.x + RADIUS + 1, MAST.z));
    let card = f.describe(MAST).unwrap();
    assert!(!card.contains("RMB=Open"), "right-click opens nothing, so the card offers nothing: {card}");
}

#[test]
fn meteorology_opens_the_station_in_the_blue_ring() {
    use crate::research::{ids::METEOROLOGY, Ring, Unlock, TECHS};
    let tech = &TECHS[METEOROLOGY as usize];
    assert_eq!((tech.name, tech.ring), ("Meteorology", Ring::Blue));
    assert!(tech.unlocks.contains(&Unlock::Recipe(crate::item::WEATHER_STATION_ITEM)));
}
