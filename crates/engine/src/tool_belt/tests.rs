//! The Tool Belt: speeds and ore by tier, mining with an empty hand, the crouched wide swings (never another kind of
//! block or a machine), the hand's head and the save-87 migration of old tools. Felling by tier: `sim/felling/tests.rs`.

use super::*;
use crate::action::Action;
use crate::block::{DIRT, GRANITE, IRON_ORE, LEAVES, LOG, PLANKS, STONE, STORAGE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{IRON_PLATE, STEEL_PICKAXE, STONE_PICKAXE};
use crate::sim::tests::{outcrop, SEED};
use crate::sim::SimEvent;
use crate::tests::run_until_ready;
use crate::Game;

const P: PlayerId = PlayerId(0);
const SKY: i32 = 200;

fn wear(sim: &mut Sim, belt: ItemId) {
    sim.players[0].as_mut().unwrap().inventory.worn[crate::equipment::BELT] = belt;
}

#[test]
fn each_tier_is_faster_on_its_blocks_and_keeps_more_ore() {
    assert_eq!(BELTS.map(|t| t.speed), [2.0, 4.0, 6.0, 8.0, 10.0]);
    assert_eq!((speed(ItemId::NONE, STONE), speed(TOOL_BELT, STONE), speed(TOOL_BELT_MK5, LOG)), (1.0, 2.0, 10.0));
    assert_eq!((speed(TOOL_BELT_MK2, DIRT), speed(TOOL_BELT_MK5, LEAVES)), (4.0, 1.0), "no class: bare hands");
    assert_eq!(BELTS.map(|t| ore_yield(t.item)), [3, 4, 5, 5, 5]);
    assert_eq!(ore_yield(ItemId::NONE), HAND_YIELD);
    assert_eq!(BELTS.map(|t| t.reach), [Reach::One, Reach::One, Reach::Column, Reach::Face, Reach::Vein]);
    assert_eq!(
        (head(TOOL_BELT_MK2, LOG), head(TOOL_BELT, STONE), head(TOOL_BELT_MK4, DIRT)),
        (IRON_AXE, STONE_PICKAXE, STEEL_SHOVEL)
    );
    assert_eq!((head(ItemId::NONE, STONE), head(TOOL_BELT, LEAVES)), (ItemId::NONE, ItemId::NONE));
    for (i, t) in BELTS.iter().enumerate() {
        assert_eq!(crate::equipment::gear(t.item).map(|g| g.slot), Some(crate::equipment::BELT), "Mk{}", i + 1);
    }
}

/// Ticks the local player takes to break `block` set below them, with nothing in the selected slot.
fn ticks_to_break(g: &mut Game, block: BlockId) -> u32 {
    let p = IVec3::new(0, SKY, 0);
    g.sim.world.set_block_anywhere(p, block);
    g.teleport(0.5, (SKY + 2) as f64, 0.5);
    g.set_look(0.0, -1.55);
    g.set_mining(true);
    for t in 1..900 {
        g.update(1.0 / 60.0);
        if g.sim.world.get_block(p) == Some(AIR) {
            g.set_mining(false);
            (0..30).for_each(|_| g.update(1.0 / 60.0));
            return t;
        }
    }
    panic!("never broke");
}

#[test]
fn an_empty_hand_mines_stone_and_dirt_at_the_belts_speed() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    crate::tests::fly(&mut g);
    g.select_slot(8);
    assert!(g.inventory().selected_stack().is_empty());
    for block in [STONE, DIRT] {
        wear(&mut g.sim, ItemId::NONE);
        let bare = ticks_to_break(&mut g, block);
        wear(&mut g.sim, TOOL_BELT);
        let mk1 = ticks_to_break(&mut g, block);
        wear(&mut g.sim, TOOL_BELT_MK2);
        let mk2 = ticks_to_break(&mut g, block);
        let name = block::def(block).name;
        assert!(bare >= 20 && mk1 * 2 <= bare + 2 && mk2 * 4 <= bare + 4, "{name}: {bare} / {mk1} / {mk2} ticks");
        assert!(mk1 * 2 >= bare - 2, "{name}: Mk1 is twice as fast, not more");
    }
    assert!(g.inventory().selected_stack().is_empty(), "nothing was held, and nothing wore");
}

#[test]
fn the_belt_keeps_more_ore_by_hand() {
    let (top, _, _) = outcrop();
    let kept = |belt: ItemId| {
        let mut sim = Sim::new(SEED, 2);
        wear(&mut sim, belt);
        sim.apply(P, Action::BreakBlock { pos: top });
        sim.events.iter().find_map(|e| match e {
            SimEvent::HandMined { kept, .. } => Some(*kept),
            _ => None,
        })
    };
    assert_eq!(
        (kept(ItemId::NONE), kept(TOOL_BELT), kept(TOOL_BELT_MK2), kept(TOOL_BELT_MK3)),
        (Some(3), Some(3), Some(4), Some(5))
    );
}

/// A sim with `blocks` set high in the sky and `P` wearing `belt`.
fn site(belt: ItemId, blocks: &[(IVec3, BlockId)]) -> Sim {
    let mut sim = Sim::new(7, 2);
    for &(p, b) in blocks {
        sim.world.set_block_anywhere(p, b);
    }
    wear(&mut sim, belt);
    sim
}

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, SKY + y, z)
}

fn air(sim: &mut Sim, p: IVec3) -> bool {
    sim.world.block_anywhere_or_generate(p) == AIR
}

#[test]
fn mk3_digs_a_tunnel_of_two_of_one_tool_class() {
    // Hit from the side: the block and the one below; granite is stone's class, dirt is not.
    let mut sim = site(TOOL_BELT_MK3, &[(at(0, 1, 0), STONE), (at(0, 0, 0), GRANITE), (at(0, -1, 0), STONE)]);
    sim.apply(P, Action::MineWide { pos: at(0, 1, 0), face: 0 });
    assert!(air(&mut sim, at(0, 1, 0)) && air(&mut sim, at(0, 0, 0)) && !air(&mut sim, at(0, -1, 0)));
    let mut sim = site(TOOL_BELT_MK3, &[(at(0, 1, 0), STONE), (at(0, 0, 0), DIRT)]);
    sim.apply(P, Action::MineWide { pos: at(0, 1, 0), face: 0 });
    assert!(air(&mut sim, at(0, 1, 0)) && !air(&mut sim, at(0, 0, 0)), "dirt wants a spade");
    // Hit from above: the next one down.
    let mut sim = site(TOOL_BELT_MK3, &[(at(0, 1, 0), STONE), (at(0, 0, 0), STONE)]);
    sim.apply(P, Action::MineWide { pos: at(0, 1, 0), face: 2 });
    assert!(air(&mut sim, at(0, 0, 0)));
    // A Mk2 swing takes only the block.
    let mut sim = site(TOOL_BELT_MK2, &[(at(0, 1, 0), STONE), (at(0, 0, 0), STONE)]);
    sim.apply(P, Action::MineWide { pos: at(0, 1, 0), face: 0 });
    assert!(air(&mut sim, at(0, 1, 0)) && !air(&mut sim, at(0, 0, 0)));
}

#[test]
fn a_wide_swing_never_takes_a_machine() {
    // Planks hit, a storage box (wood too, and a machine) below it.
    let mut sim = site(TOOL_BELT_MK3, &[(at(0, 1, 0), PLANKS), (at(0, -1, 0), STONE)]);
    sim.apply(P, Action::Give { item: STORAGE.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos: at(0, 0, 0), slot: 0, facing: 0, against: at(0, -1, 0) });
    assert!(sim.factory.occupies(at(0, 0, 0)), "the box stands");
    sim.apply(P, Action::MineWide { pos: at(0, 1, 0), face: 0 });
    assert!(air(&mut sim, at(0, 1, 0)));
    assert_eq!(sim.world.block_anywhere_or_generate(at(0, 0, 0)), STORAGE, "the box was not taken");
    // Hitting the machine itself breaks only it.
    sim.world.set_block_anywhere(at(1, 0, 0), STORAGE);
    wear(&mut sim, TOOL_BELT_MK5);
    sim.apply(P, Action::MineWide { pos: at(0, 0, 0), face: 0 });
    assert!(air(&mut sim, at(0, 0, 0)) && !air(&mut sim, at(1, 0, 0)));
}

#[test]
fn mk4_takes_the_three_by_three_face_of_one_block_only() {
    // A wall in the plane x = 0, hit from +x: stone but for one granite and one iron ore cell.
    let mut blocks = Vec::new();
    for y in -2..=2 {
        for z in -2..=2 {
            blocks.push((at(0, y, z), STONE));
        }
    }
    blocks.push((at(0, 1, 1), GRANITE));
    blocks.push((at(0, -1, 0), IRON_ORE));
    blocks.push((at(-1, 0, 0), STONE));
    let mut sim = site(TOOL_BELT_MK4, &blocks);
    sim.apply(P, Action::MineWide { pos: at(0, 0, 0), face: 0 });
    for y in -2i32..=2 {
        for z in -2i32..=2 {
            let inside = y.abs() <= 1 && z.abs() <= 1 && (y, z) != (1, 1) && (y, z) != (-1, 0);
            assert_eq!(air(&mut sim, at(0, y, z)), inside, "({y}, {z})");
        }
    }
    assert!(!air(&mut sim, at(-1, 0, 0)), "nothing behind the face");
}

#[test]
fn mk5_cuts_up_to_thirty_two_joined_blocks_of_its_kind() {
    // A line of 40 stone along x with granite at its side; the cut starts at one end.
    let mut blocks: Vec<_> = (0..40).map(|x| (at(x, 0, 0), STONE)).collect();
    blocks.extend((0..40).map(|x| (at(x, 0, 1), GRANITE)));
    let mut sim = site(TOOL_BELT_MK5, &blocks);
    sim.apply(P, Action::MineWide { pos: at(0, 0, 0), face: 2 });
    let cut = (0..40).filter(|&x| air(&mut sim, at(x, 0, 0))).count();
    assert_eq!(cut, VEIN_MAX);
    assert!((0..40).all(|x| !air(&mut sim, at(x, 0, 1))), "no granite taken");
    let cells = wide_cells(Reach::Vein, at(0, 0, 0), 2, |_| STONE, |_| true);
    assert_eq!(cells.len(), VEIN_MAX - 1, "the cap holds in open stone too");
}

#[test]
fn the_cells_are_the_same_on_every_core() {
    let blocks: Vec<_> = (0..6).flat_map(|x| (0..6).map(move |z| (at(x, 0, z), STONE))).collect();
    let (mut a, mut b) = (site(TOOL_BELT_MK5, &blocks), site(TOOL_BELT_MK5, &blocks));
    for sim in [&mut a, &mut b] {
        sim.queue(0, P, Action::MineWide { pos: at(2, 0, 2), face: 2 });
        sim.step();
    }
    assert_eq!(a.state_hash(), b.state_hash());
}

#[test]
fn the_action_travels_and_comes_back() {
    let action = Action::MineWide { pos: at(-3, 5, 9), face: 5 };
    let mut w = ByteWriter::default();
    action.write(&mut w);
    assert_eq!(Action::read(&mut ByteReader::new(&w.bytes)), Some(action));
}

#[test]
fn old_tools_become_a_worn_belt_when_a_version_86_save_loads() {
    let load = |inv: &Inventory| {
        let mut w = ByteWriter::default();
        inv.write_state(&mut w);
        let mut r = ByteReader::new(&w.bytes);
        r.version = 86;
        let back = Inventory::read_state(&mut r).expect("a version 86 inventory reads");
        assert!(r.is_done());
        back
    };
    let mut inv = Inventory::default();
    inv.slots[4] = Stack { item: STEEL_PICKAXE, count: 1337 };
    inv.offhand = Stack { item: IRON_AXE, count: 20 };
    let back = load(&inv);
    assert_eq!(
        (back.belt(), back.count(STEEL_PICKAXE), back.offhand.item),
        (TOOL_BELT_MK3, 1337, IRON_AXE),
        "kept as items"
    );

    // The worn rig counts as Mk2; a better tool still wins.
    inv = Inventory::default();
    inv.worn[crate::equipment::BELT] = MINING_RIG;
    assert_eq!(load(&inv).belt(), TOOL_BELT_MK2);
    inv.cursor = Stack { item: STEEL_SHOVEL, count: 3 };
    assert_eq!(load(&inv).belt(), TOOL_BELT_MK3);
    inv = Inventory::default();
    inv.slots[0] = Stack { item: STONE_PICKAXE, count: 87 };
    assert_eq!(load(&inv).belt(), TOOL_BELT);
    assert_eq!(load(&Inventory::default()).belt(), ItemId::NONE, "no tools, no belt");

    // A current save keeps what is worn, and a worn rig is no longer valid gear.
    let mut now = Inventory::default();
    now.worn[crate::equipment::BELT] = TOOL_BELT_MK4;
    now.slots[0] = Stack { item: STEEL_PICKAXE, count: 5 };
    let mut w = ByteWriter::default();
    now.write_state(&mut w);
    assert_eq!(Inventory::read_state(&mut ByteReader::new(&w.bytes)).map(|i| i.belt()), Some(TOOL_BELT_MK4));
    now.worn[crate::equipment::BELT] = MINING_RIG;
    let mut w = ByteWriter::default();
    now.write_state(&mut w);
    assert!(Inventory::read_state(&mut ByteReader::new(&w.bytes)).is_none());
}

#[test]
fn a_worn_belt_is_raised_in_place_with_plates_then_kits() {
    let mut inv = Inventory::default();
    inv.worn[crate::equipment::BELT] = TOOL_BELT;
    inv.add(IRON_PLATE, 5);
    assert!(!inv.upgrade_worn(crate::equipment::BELT, |_| true), "one plate short");
    inv.add(IRON_PLATE, 1);
    assert!(inv.upgrade_worn(crate::equipment::BELT, |_| true));
    assert_eq!((inv.belt(), inv.count(IRON_PLATE)), (TOOL_BELT_MK2, 0));
    inv.add(crate::item::STEEL_PLATE, 6);
    assert!(!inv.upgrade_worn(crate::equipment::BELT, |next| next != TOOL_BELT_MK3), "not researched");
    assert!(inv.upgrade_worn(crate::equipment::BELT, |_| true));
    assert_eq!(inv.belt(), TOOL_BELT_MK3);
}
