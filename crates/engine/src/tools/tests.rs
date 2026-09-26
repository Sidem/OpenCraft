//! Tools: which blocks they suit, break speed in the hands, wear and ore kept in the core.

use super::*;
use crate::action::Action;
use crate::block::{AIR, DIRT, GRASS, IRON_ORE, LEAVES, LOG, SAND, STONE};
use crate::item::IRON_PLATE;
use crate::math::IVec3;
use crate::sim::tests::{outcrop, SEED};
use crate::sim::{PlayerId, Sim, SimEvent};
use crate::tests::run_until_ready;
use crate::Game;

const A: PlayerId = PlayerId(0);
const SKY: i32 = 200;

#[test]
fn each_block_wants_the_tool_for_its_material() {
    assert_eq!(kind_for(STONE), Some(ToolKind::Pickaxe));
    assert_eq!(kind_for(IRON_ORE), Some(ToolKind::Pickaxe));
    assert_eq!(kind_for(LOG), Some(ToolKind::Axe));
    for b in [DIRT, GRASS, SAND] {
        assert_eq!(kind_for(b), Some(ToolKind::Shovel));
    }
    assert_eq!(kind_for(LEAVES), None);
    use crate::block::{BASALT, GLASS, GRANITE, LIMESTONE, QUARTZ_ORE, SANDSTONE};
    for b in [GRANITE, SANDSTONE, BASALT, LIMESTONE, QUARTZ_ORE, GLASS] {
        assert_eq!(kind_for(b), Some(ToolKind::Pickaxe), "{}", crate::block::def(b).name);
    }
    assert_eq!(break_speed(STONE_PICKAXE, LOG), 1.0, "the wrong tool is just a hand");
    assert_eq!(break_speed(IRON_AXE, LOG), 4.0);
    assert_eq!(break_speed(IRON_PLATE, STONE), 1.0);
    assert_eq!((ore_yield(IRON_PLATE), ore_yield(STONE_PICKAXE), ore_yield(IRON_PICKAXE)), (3, 3, 4));
    assert_eq!(ore_yield(IRON_AXE), 3, "only a pickaxe keeps more ore");
    for t in &TOOLS {
        assert_eq!(crate::item::stack_size(t.item), t.tier.uses, "a tool's stack is its uses");
    }
}

/// Ticks the local player takes to break a stone block below them while holding the selected slot.
fn ticks_to_break_stone(g: &mut Game) -> u32 {
    let p = IVec3::new(0, SKY, 0);
    g.sim.world.set_block_anywhere(p, STONE);
    g.teleport(0.5, (SKY + 2) as f64, 0.5);
    g.set_look(0.0, -1.55);
    g.set_mining(true);
    for t in 1..600 {
        g.update(1.0 / 60.0);
        if g.sim.world.get_block(p) == Some(AIR) {
            g.set_mining(false);
            for _ in 0..30 {
                g.update(1.0 / 60.0); // the break cooldown
            }
            return t;
        }
    }
    panic!("never broke");
}

/// Gives `uses` of a tool and selects its hotbar slot.
fn hold(g: &mut Game, item: ItemId, uses: u32) {
    g.give(item.0, uses);
    g.update(1.0 / 60.0);
    let slot = (0..9).find(|&i| g.slot_item(i) == item.0).expect("in the hotbar");
    g.select_slot(slot);
    g.update(1.0 / 60.0);
}

#[test]
fn a_pickaxe_breaks_stone_faster_and_wears() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.toggle_fly();
    g.select_slot(8);
    let hands = ticks_to_break_stone(&mut g);
    hold(&mut g, STONE_PICKAXE, STONE_TIER.uses);
    let stone = ticks_to_break_stone(&mut g);
    hold(&mut g, IRON_PICKAXE, IRON_TIER.uses);
    let iron = ticks_to_break_stone(&mut g);
    assert!(hands >= 60 && stone * 2 <= hands + 2 && iron * 4 <= hands + 4, "{hands} / {stone} / {iron} ticks");
    let slot = g.selected_slot();
    assert_eq!(g.slot_count(slot), IRON_TIER.uses - 1, "one use gone");
}

/// Stone at `SKY` along x, a dirt block beside them; `A` holds `uses` of `tool` in slot 0.
fn quarry(tool: ItemId, uses: u32) -> Sim {
    let mut sim = Sim::new(SEED, 2);
    for x in 0..3 {
        sim.world.set_block_anywhere(IVec3::new(x, SKY, 0), STONE);
    }
    sim.world.set_block_anywhere(IVec3::new(5, SKY, 0), DIRT);
    sim.apply(A, Action::Give { item: tool, count: uses });
    sim
}

#[test]
fn a_tool_wears_down_to_nothing_the_same_on_every_core() {
    let (mut a, mut b) = (quarry(STONE_PICKAXE, 2), quarry(STONE_PICKAXE, 2));
    for sim in [&mut a, &mut b] {
        sim.queue(0, A, Action::BreakBlock { pos: IVec3::new(0, SKY, 0) });
        sim.queue(1, A, Action::BreakBlock { pos: IVec3::new(5, SKY, 0) });
        sim.queue(2, A, Action::BreakBlock { pos: IVec3::new(1, SKY, 0) });
        sim.queue(3, A, Action::BreakBlock { pos: IVec3::new(2, SKY, 0) });
    }
    let mut worn = Vec::new();
    for t in 0..4 {
        a.step();
        b.step();
        assert_eq!(a.state_hash(), b.state_hash(), "tick {t}");
        worn.extend(a.events.drain(..).filter(|e| matches!(e, SimEvent::ToolWornOut { .. })));
        let slot = a.player(A).unwrap().inventory.slots[0];
        let uses = [1, 1, 0, 0][t];
        assert_eq!(slot.count, uses, "tick {t}: stone costs a use, dirt doesn't");
    }
    assert_eq!(worn, vec![SimEvent::ToolWornOut { player: A, item: STONE_PICKAXE }]);
    assert_eq!(a.world.block_anywhere_or_generate(IVec3::new(2, SKY, 0)), AIR, "bare hands still break");
}

#[test]
fn an_iron_pickaxe_keeps_four_ore() {
    let (top, other, _) = outcrop();
    let kept = |tool: ItemId, pos: IVec3| {
        let mut sim = quarry(tool, 10);
        sim.apply(A, Action::BreakBlock { pos });
        let drops: Vec<u32> = sim
            .events
            .iter()
            .filter_map(|e| match e {
                SimEvent::Dropped { item, count, .. } if *item == IRON_ORE.into() || item.0 < 256 => Some(*count),
                _ => None,
            })
            .collect();
        drops[0]
    };
    assert_eq!(kept(IRON_PLATE, top), 3, "bare-handed");
    assert_eq!(kept(STONE_PICKAXE, top), 3);
    assert_eq!(kept(IRON_PICKAXE, top), 4);
    assert_eq!(kept(IRON_PICKAXE, other), 4);
}

#[test]
fn a_dropped_tool_goes_whole() {
    let mut sim = quarry(IRON_AXE, 100);
    sim.apply(A, Action::DropSelected { count: 1 });
    assert_eq!(sim.events, vec![SimEvent::Thrown { player: A, item: IRON_AXE, count: 100 }]);
}
