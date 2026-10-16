use crate::block::{COAL_ORE, TOWN_DEPOT};
use crate::item::{ItemId, IRON_PLATE};
use crate::math::IVec3;
use crate::settlements::{Town, FORGE, ORDERS};
use crate::Game;

const AT: IVec3 = IVec3::new(3000, 120, -3000);

#[test]
fn the_town_panel_reads_its_depot_and_the_news_follows_a_stall_and_its_end() {
    let mut g = Game::new(2024, 2);
    g.sim.factory.place(&mut g.sim.world, TOWN_DEPOT, AT, 0, AT - IVec3::new(0, 1, 0), 0);
    let mut t = Town::new(AT, FORGE);
    (t.built, t.planned, t.due) = (1, 1, 0);
    t.delivered = vec![0; ORDERS[1].len()];
    g.sim.settlements.towns.push(t);
    let cell = AT + IVec3::new(0, 1, 0);

    let p = g.town_panel(cell.x, cell.y, cell.z);
    assert_eq!(p[..5], [u32::from(FORGE), 1, 5, 0, ORDERS[1].len() as u32]);
    let upkeep_at = 5 + 3 * ORDERS[1].len();
    assert_eq!(p[upkeep_at] as usize * 3 + upkeep_at + 1, p.len(), "upkeep lines fill the rest");
    assert!(g.town_heading(cell.x, cell.y, cell.z).starts_with("Basalt Forge, a camp, "));
    assert!(g.town_perk(cell.x, cell.y, cell.z).starts_with("Perk: 20 iron or copper ore a minute"));
    assert!(g.town_panel(0, 0, 0).is_empty());
    assert_eq!(g.town_states(), vec![0]);

    // No upkeep in the depot: it stalls, and the news names what it is short of.
    g.sim.step();
    assert_eq!(g.town_states(), vec![1]);
    let news = g.town_news(0);
    assert!(news.starts_with("Your basalt forge has stalled: short of Coal Ore"), "{news}");

    // Its upkeep in the store: supplied again.
    for (item, n) in g.sim.settlements.towns[0].upkeep() {
        g.sim.factory.depot_store(AT).unwrap().add(item, n);
    }
    g.sim.step();
    assert_eq!((g.town_states(), g.town_news(0)), (vec![0], "Your basalt forge is supplied again".to_string()));

    // Taking a depot's output by hand takes the perk's goods and leaves the town's store.
    let store = g.sim.factory.depot_store(AT).unwrap();
    store.add(ItemId::block(COAL_ORE), 10);
    g.sim.factory.depot_perk(AT).unwrap().1.add(IRON_PLATE, 6);
    let mut taken = Vec::new();
    g.sim.factory.take_contents(AT, |item, n| {
        taken.push((item, n));
        n
    });
    assert_eq!(taken, vec![(IRON_PLATE, 6)]);
    assert_eq!(g.sim.factory.depot_held(AT).unwrap().count(ItemId::block(COAL_ORE)), 10);
}
