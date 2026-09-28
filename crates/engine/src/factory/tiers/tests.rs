use super::*;
use crate::factory::belt::BELT_TIERS;
use crate::factory::MINER_TIERS;

#[test]
fn every_tier_has_numbers_and_round_trips() {
    let numbers = |block: BlockId| if block == BELT { BELT_TIERS.len() } else { MINER_TIERS.len() };
    for f in FAMILIES {
        assert_eq!(f.items.len(), numbers(f.block), "tiers of block {}", f.block);
        for (t, &item) in f.items.iter().enumerate() {
            assert_eq!(placed_by(item), Some((f.block, t as u8)));
            assert_eq!(item_of(f.block, t as u8), Some(item));
        }
        assert_eq!(item_of(f.block, f.items.len() as u8), None);
    }
}

#[test]
fn a_mk2_item_places_the_family_block() {
    assert_eq!(ItemId::block(FAST_BELT).places(), Some(BELT));
    assert_eq!(ItemId::block(MINER_MK2).places(), Some(MINER));
    assert_eq!(ItemId::block(BELT).places(), Some(BELT));
    assert_eq!(placed_by(ItemId::block(crate::block::STONE)), None);
}

/// A factory saved by version 17 (before tiers): a Mk1 belt, a fast belt and a Mk2 miner. Tier bytes
/// replaced the `fast` and `mk2` bools byte for byte (processors: `process/tests.rs`).
const V17_FACTORY: &str =
    "020000000000000000000000000000000100000000000001000000000000000000000001000100000000010000000000\
    000001000000000000000300000000000000000000000000000000000200000000000000000000000001000000000000000000000000\
    000000000000000000000000000000000000000000000000000000000000000000000000ff07000000000000000000000000000000000000\
    00000000000000000000000000";

#[test]
fn a_version_17_factory_loads_with_the_same_speeds_and_rates() {
    use crate::bytes::{ByteReader, ByteWriter};
    use crate::factory::Factory;
    use crate::math::IVec3;
    let bytes: Vec<u8> =
        (0..V17_FACTORY.len() / 2).map(|i| u8::from_str_radix(&V17_FACTORY[2 * i..2 * i + 2], 16).unwrap()).collect();
    let mut r = ByteReader::new(&bytes);
    r.version = 17;
    let f = Factory::read_state(&mut crate::world::World::new(1, 2), &mut r).unwrap();
    assert_eq!(f.belt_at(IVec3::new(0, 0, 0)).speed(), 1.0);
    assert_eq!(f.belt_at(IVec3::new(1, 0, 0)).speed(), 2.0);
    let miner = f.miner_at(IVec3::new(0, 1, 0));
    assert_eq!((miner.rate(), miner.recovery(), miner.stats().power), (2.0, 0.75, 20));
    // Today's format (a processor list since version 18) keeps the tiers.
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let g = Factory::read_state(&mut crate::world::World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((g.belt_at(IVec3::new(1, 0, 0)).speed(), g.miner_at(IVec3::new(0, 1, 0)).rate()), (2.0, 2.0));
}
