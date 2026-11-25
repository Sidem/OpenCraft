use crate::block::{
    BlockId, AMBER_TILES, CLAY, CLAY_BRICKS, COAL_ORE, GLASS_KILN, GRANITE, KAOLIN, PORCELAIN, SMELTER, TERRACOTTA,
    TURQUOISE_TILES,
};
use crate::factory::tests::run;
use crate::factory::Factory;
use crate::item::{ItemId, PIGMENT_GREEN, PIGMENT_RED};
use crate::math::IVec3;
use crate::research::ids::{CLAY_AND_KILNS, PORCELAIN_AND_GLAZES};
use crate::research::{Ring, TECHS};
use crate::world::World;

const AT: IVec3 = IVec3::ZERO;

fn machine(block: BlockId) -> Factory {
    let mut f = Factory::default();
    f.place(&mut World::new(1, 2), block, AT, 0, AT - IVec3::new(0, 1, 0), 0);
    f
}

fn research(f: &mut Factory, tech: u8) {
    f.research.open_ring(TECHS[tech as usize].ring);
    (0..TECHS[tech as usize].units).for_each(|_| f.research.add_unit(tech));
}

fn made(f: &Factory, block: BlockId, item: BlockId) -> u32 {
    f.processor_at(AT, block).out.count(ItemId::block(item))
}

#[test]
fn the_clay_techs_sit_in_the_building_row_of_their_rings() {
    assert_eq!(TECHS[CLAY_AND_KILNS as usize].ring, Ring::Green);
    assert_eq!(TECHS[PORCELAIN_AND_GLAZES as usize].ring, Ring::Blue);
}

#[test]
fn clay_and_kilns_lets_a_smelter_fire_bricks_and_the_kiln_terracotta() {
    let mut smelter = machine(SMELTER);
    assert_eq!(smelter.insert(AT, CLAY.into(), 2), 0, "bricks are locked");
    research(&mut smelter, CLAY_AND_KILNS);
    assert_eq!(smelter.insert(AT, CLAY.into(), 2), 2);
    smelter.insert(AT, COAL_ORE.into(), 1);
    run(&mut smelter, 6.05, |_| {});
    assert_eq!(made(&smelter, SMELTER, CLAY_BRICKS), 2, "a clay a brick, 3 s each");

    let mut kiln = machine(GLASS_KILN);
    assert_eq!(kiln.insert(AT, CLAY.into(), 2), 0, "terracotta is locked");
    research(&mut kiln, CLAY_AND_KILNS);
    assert_eq!(kiln.insert(AT, CLAY.into(), 2), 2);
    kiln.insert(AT, COAL_ORE.into(), 1);
    run(&mut kiln, 4.05, |_| {});
    assert_eq!(made(&kiln, GLASS_KILN, TERRACOTTA), 2);
}

#[test]
fn kaolin_and_granite_fire_into_porcelain_and_a_pigment_glazes_it() {
    let mut kiln = machine(GLASS_KILN);
    assert_eq!(kiln.insert(AT, KAOLIN.into(), 2), 0, "porcelain is locked");
    research(&mut kiln, PORCELAIN_AND_GLAZES);
    kiln.insert(AT, KAOLIN.into(), 2);
    kiln.insert(AT, GRANITE.into(), 1);
    kiln.insert(AT, COAL_ORE.into(), 2);
    run(&mut kiln, 6.05, |_| {});
    assert_eq!(made(&kiln, GLASS_KILN, PORCELAIN), 2, "two kaolin and a granite: two porcelain");

    // The kiln picks the glaze from what it is fed: copper green fires turquoise, iron red fires amber.
    for (pigment, tile) in [(PIGMENT_GREEN, TURQUOISE_TILES), (PIGMENT_RED, AMBER_TILES)] {
        let mut kiln = machine(GLASS_KILN);
        research(&mut kiln, PORCELAIN_AND_GLAZES);
        assert_eq!(kiln.insert(AT, PORCELAIN.into(), 4), 4);
        assert_eq!(kiln.insert(AT, pigment, 1), 1);
        kiln.insert(AT, COAL_ORE.into(), 1);
        run(&mut kiln, 3.05, |_| {});
        assert_eq!(made(&kiln, GLASS_KILN, tile), 4);
    }
}
