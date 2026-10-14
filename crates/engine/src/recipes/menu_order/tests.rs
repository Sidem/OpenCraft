use super::*;
use crate::block;
use crate::factory::tiers::FAMILIES;
use crate::item;
use crate::recipes::{Group, GROUPS};
#[test]
fn every_tier_of_a_building_stands_next_to_its_siblings_in_the_build_menu() {
    for (g, group) in GROUPS.iter().enumerate() {
        let mut order: Vec<usize> = (0..RECIPES.len()).filter(|&r| RECIPES[r].group == *group).collect();
        order.sort_by_key(|&r| menu_order(r));
        for fam in FAMILIES {
            let at: Vec<usize> = order
                .iter()
                .enumerate()
                .filter(|(_, &r)| fam.items.contains(&RECIPES[r].output))
                .map(|(i, _)| i)
                .collect();
            let adjacent = at.windows(2).all(|w| w[1] == w[0] + 1);
            assert!(adjacent, "the tiers of block {} are split up in section {g}: {at:?}", fam.block);
            let tiers: Vec<usize> =
                at.iter().map(|&i| fam.items.iter().position(|&x| x == RECIPES[order[i]].output).unwrap()).collect();
            assert!(tiers.windows(2).all(|w| w[0] < w[1]), "block {} tiers out of order: {tiers:?}", fam.block);
        }
    }
}

#[test]
fn every_tier_of_a_family_is_in_one_section() {
    for fam in FAMILIES {
        let groups: Vec<Group> =
            fam.items.iter().filter_map(|&i| RECIPES.iter().find(|r| r.output == i)).map(|r| r.group).collect();
        assert!(groups.windows(2).all(|w| w[0] == w[1]), "block {} is spread over {groups:?}", fam.block);
    }
}

#[test]
fn power_buildings_sit_under_power_and_machine_groups_make_sense() {
    let group_of =
        |b: crate::block::BlockId| RECIPES.iter().find(|r| r.output == item::ItemId::block(b)).unwrap().group;
    for b in
        [block::DIESEL_GENERATOR, block::GENERATOR, block::BOILER, block::TURBINE, block::REACTOR, block::SOLAR_PANEL]
    {
        assert_eq!(group_of(b), Group::Power, "block {b}");
    }
    assert_eq!(group_of(block::DATACENTER), Group::Computing);
    assert_eq!(group_of(block::LAUNCH_SITE), Group::Space);
    assert_eq!(group_of(block::RESEARCH_CENTER), Group::Science);
    assert_eq!(group_of(block::DRONE_PORT), Group::Logistics);
}

#[test]
fn the_curated_sections_list_every_recipe() {
    let curated = [Group::Production, Group::Logistics, Group::Power, Group::Science, Group::Tools];
    for (r, recipe) in RECIPES.iter().enumerate().filter(|(_, r)| curated.contains(&r.group)) {
        assert!(menu_order(r) < MENU.len() * 8 + 8, "{} is not in MENU: place it", item::name(recipe.output));
    }
}
