use super::*;
use crate::action::Action;
use crate::block::{self, COAL_MINER, MINER, SMELTER, STONE};
use crate::item::{
    self, COPPER_WIRE, EMPTY_CANISTER, IRON_PLATE, IRON_ROD, UNDERPASS_MK2, UNDERPASS_MK3, UNDERPASS_MK4,
};
use crate::recipes::MACHINE_RECIPES;
use crate::settlements::discounted;
use crate::sim::Sim;

const P: PlayerId = PlayerId(0);

fn row(item: ItemId) -> usize {
    RECIPES.iter().position(|r| r.output == item).unwrap()
}

fn held(sim: &Sim, item: ItemId) -> u32 {
    sim.player(P).unwrap().inventory.count(item)
}

/// The user's case: a Miner crafted where a Coal Miner was meant. Its parts and the stone a new player has at hand
/// must make the Coal Miner, with no more mining.
#[test]
fn a_wrong_miner_becomes_a_coal_miner() {
    let mut sim = Sim::new(7, 2);
    let miner = ItemId::block(MINER);
    for (item, n) in [(IRON_PLATE, 5), (IRON_ROD, 4), (COPPER_WIRE, 6), (ItemId::block(STONE), 8)] {
        sim.apply(P, Action::Give { item, count: n });
    }
    sim.apply(P, Action::Craft { recipe: row(miner) as u16, times: 1 });
    while held(&sim, miner) == 0 {
        sim.step();
    }
    sim.apply(P, Action::TakeApart { recipe: row(miner) as u16, times: 1 });
    assert_eq!(held(&sim, miner), 0);
    assert_eq!(held(&sim, SCRAP_METAL), 1, "a little of the iron comes back as scrap");
    let coal = ItemId::block(COAL_MINER);
    sim.apply(P, Action::Craft { recipe: row(coal) as u16, times: 1 });
    for _ in 0..60 * 30 {
        sim.step();
    }
    assert_eq!(held(&sim, coal), 1, "KEEP_PERCENT is too low for the parts to make a Coal Miner");
}

#[test]
fn only_what_is_held_comes_apart_in_whole_batches() {
    let mut sim = Sim::new(7, 2);
    let belt = ItemId::block(block::BELT);
    sim.apply(P, Action::Give { item: belt, count: 7 });
    sim.apply(P, Action::TakeApart { recipe: row(belt) as u16, times: 5 });
    assert_eq!(held(&sim, belt), 3, "one batch of four; three are not a batch");
    sim.apply(P, Action::TakeApart { recipe: row(ItemId::block(MINER)) as u16, times: 1 });
    sim.apply(P, Action::TakeApart { recipe: u16::MAX, times: 1 });
    assert_eq!(held(&sim, belt), 3);
}

/// Red team: scrap melts to iron, so scrap from stone would make iron out of stone (a Smelter is all stone).
#[test]
fn scrap_comes_only_from_iron() {
    assert_eq!(parts(row(ItemId::block(SMELTER))).unwrap().scrap, 0);
}

/// Every route to an item (hand or machine, a workshop's half price too) must have used at least as much of each raw
/// material as taking the item apart gives back. Independent of `parts`'s own check, which trusts the cheapest route.
#[test]
fn no_route_makes_an_item_for_less_than_it_gives_back() {
    for (i, r) in RECIPES.iter().enumerate() {
        let Some(p) = parts(i) else { continue };
        let mut back = Content::new();
        for &(item, n) in &p.items {
            add(&mut back, content(item).unwrap(), n as u64, 1);
        }
        add(&mut back, content(IRON_INGOT).unwrap(), p.scrap as u64, SCRAP_PER_INGOT);
        let hand = RECIPES.iter().filter(|h| h.output == r.output);
        let mut routes: Vec<(Vec<(ItemId, u32)>, u32)> =
            hand.map(|h| (gear_cost(h, true).collect(), h.count)).collect();
        for m in MACHINE_RECIPES {
            if let Some(&(_, n)) = m.outputs.iter().find(|o| o.0 == r.output) {
                routes.push((m.inputs.to_vec(), n));
            }
        }
        for (inputs, made) in routes {
            let mut paid = Content::new();
            for (item, n) in inputs {
                add(&mut paid, content(item).unwrap(), n as u64 * r.count as u64, 1);
            }
            for &(raw, m) in &back {
                let p = paid.iter().find(|e| e.0 == raw).map_or(0, |e| e.1);
                assert!(
                    m * made as u64 <= p,
                    "{} gives back more {} than a route takes",
                    item::name(r.output),
                    item::name(raw)
                );
            }
        }
    }
}

/// What cannot be taken apart, besides tools, and why. A new entry here means new content opened a loop: read it.
#[test]
fn only_these_cannot_be_taken_apart() {
    let refused: Vec<ItemId> = (0..RECIPES.len())
        .filter(|&i| parts(i).is_none() && tools::tool(RECIPES[i].output).is_none())
        .map(|i| RECIPES[i].output)
        .collect();
    let expected = [
        // The upgrader makes Mk2+ pieces from the piece below and kits; by hand they are belts of their Mk.
        UNDERPASS_MK2,
        UNDERPASS_MK3,
        UNDERPASS_MK4,
        // Refining and the wafer line give canisters back as a byproduct: taken apart they would turn oil into steel.
        EMPTY_CANISTER,
    ];
    let names = |v: &[ItemId]| v.iter().map(|&i| item::name(i)).collect::<Vec<_>>();
    assert_eq!(names(&refused), names(&expected));
}

#[test]
fn workshop_gear_gives_back_from_its_half_price() {
    for (i, r) in RECIPES.iter().enumerate().filter(|(_, r)| discounted(r.output)) {
        let Some(p) = parts(i) else { continue }; // the jetpack is a tool
        for (item, n) in gear_cost(r, true) {
            let back = p.items.iter().find(|e| e.0 == item).map_or(0, |e| e.1);
            assert!(back <= n, "{}", item::name(r.output));
        }
    }
}

#[test]
fn tools_never_come_apart() {
    for t in tools::TOOLS {
        assert!(parts(row(t.item)).is_none(), "{}", item::name(t.item));
    }
}
