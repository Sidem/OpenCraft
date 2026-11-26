//! Tools: which tool class each block wants, and the legacy tools as items (a stack is its uses, it moves whole).
//! Mining itself is the Tool Belt's: `tool_belt/tests.rs`.

use super::*;
use crate::action::Action;
use crate::block::{DIRT, GRASS, IRON_ORE, LEAVES, LOG, SAND, STONE};
use crate::sim::{PlayerId, Sim, SimEvent};

const A: PlayerId = PlayerId(0);

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
    for t in &TOOLS {
        assert_eq!(crate::item::stack_size(t.item), t.tier.uses, "a tool's stack is its uses");
    }
}

#[test]
fn a_dropped_old_tool_goes_whole() {
    let mut sim = Sim::new(7, 2);
    sim.apply(A, Action::Give { item: IRON_AXE, count: 100 });
    sim.apply(A, Action::DropSelected { count: 1 });
    assert_eq!(sim.events, vec![SimEvent::Thrown { player: A, item: IRON_AXE, count: 100 }]);
}
