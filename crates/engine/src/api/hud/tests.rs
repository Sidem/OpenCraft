//! Tests for target card action chips (`api/hud.rs`).

use super::block_controls;
use crate::block;

#[test]
fn non_tree_blocks_show_fitting_action_verbs_never_chop_tree() {
    assert_eq!(block_controls(block::SAND), "\tLMB=Dig");
    assert_eq!(block_controls(block::DIRT), "\tLMB=Dig");
    assert_eq!(block_controls(block::GRASS), "\tLMB=Dig");
    assert_eq!(block_controls(block::BAUXITE_ORE), "\tLMB=Mine");
    assert_eq!(block_controls(block::IRON_ORE), "\tLMB=Mine");
    assert_eq!(block_controls(block::STONE), "\tLMB=Mine");
    assert_eq!(block_controls(block::SPENT_ROCK), "\tLMB=Mine");
    assert_eq!(block_controls(block::LOG), "\tLMB=Chop");
    assert_eq!(block_controls(block::LEAVES), "\tLMB=Cut");
    assert_eq!(block_controls(block::CACTUS), "\tLMB=Cut");
    assert_eq!(block_controls(block::RUSTED_PLATING), "\tLMB=Salvage");
    assert_eq!(block_controls(block::PLANKS), "\tLMB=Break");
    assert_eq!(block_controls(block::GLASS), "\tLMB=Break");
    assert_eq!(block_controls(block::BEDROCK), "");
    assert_eq!(block_controls(block::LOGBOOK), "\tRMB=Read");
    assert_eq!(block_controls(block::VAULT_DOOR), "\tRMB=Unlock");
    assert_eq!(block_controls(block::SIGN), "\tLMB=Break\tRMB=Edit");
    assert_eq!(block_controls(block::WOOD_DOOR), "\tLMB=Break\tRMB=Open");
    assert_eq!(block_controls(block::WOOD_DOOR_OPEN), "\tLMB=Break\tRMB=Close");
    assert_eq!(block_controls(block::LED), "\tLMB=Break\tRMB=Adjust");
}

#[test]
fn no_plain_block_ever_claims_chop_tree() {
    for id in 1..block::BLOCK_COUNT as block::BlockId {
        if block::is_tree_wood(id) {
            continue;
        }
        let keys = block_controls(id);
        assert!(!keys.contains("Chop tree"), "block {} ({id}) got {keys:?}", block::def(id).name);
    }
}
