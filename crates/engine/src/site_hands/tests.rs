use crate::item::PLANNER;
use crate::math::IVec3;
use crate::raycast::RayHit;
use crate::Game;

fn aim(g: &mut Game, block: IVec3) {
    g.target = Some(RayHit { block, normal: IVec3::new(0, 1, 0), id: 1 });
}

fn click(g: &mut Game) {
    g.using = true;
    g.update_planner();
}

fn hold_planner() -> Game {
    let mut g = Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    g.give(PLANNER.0, 1);
    g.run_ticks(1);
    let slot = g.inventory().slots.iter().position(|s| s.item == PLANNER).unwrap();
    g.select_slot(slot as u32);
    g.run_ticks(1);
    g
}

#[test]
fn two_clicks_ask_for_a_new_site_and_a_third_asks_for_it() {
    let mut g = hold_planner();
    assert!(g.planner_held() && g.reach() > 60.0);
    assert!(g.planner_label().unwrap().contains("Right-click a corner"));
    let a = IVec3::new(900, 70, -900);
    aim(&mut g, a);
    click(&mut g);
    assert!(!g.using && g.take_site_request().is_empty());
    assert!(g.planner_label().unwrap().contains("Corner at 900, 70, -900"));
    assert_eq!(g.site_boxes().len(), 7, "the box being marked");

    aim(&mut g, IVec3::new(904, 72, -897));
    click(&mut g);
    assert_eq!(g.take_site_request(), vec![0, 900, -900, 904, -897, 70, 72]);
    assert!(g.take_site_request().is_empty());

    g.mark_site(900, -900, 904, -897, 70, 2);
    g.run_ticks(1);
    assert_eq!(g.sim.factory.sites.list.len(), 1);
    assert_eq!(g.site_boxes().len(), 14, "its extent and its level");
    aim(&mut g, IVec3::new(902, 60, -898));
    click(&mut g);
    assert_eq!(g.take_site_request(), vec![1, g.sim.factory.sites.list[0].id as i32]);
}

#[test]
fn clicking_the_same_column_drops_the_corner_and_putting_it_away_clears_it() {
    let mut g = hold_planner();
    aim(&mut g, IVec3::new(5, 70, 5));
    click(&mut g);
    aim(&mut g, IVec3::new(5, 75, 5));
    click(&mut g);
    assert!(g.take_site_request().is_empty() && g.planner.corner.is_none());

    click(&mut g);
    assert!(g.planner.corner.is_some());
    g.select_slot(8);
    g.run_ticks(1);
    if !g.planner_held() {
        g.update_planner();
        assert!(g.planner.corner.is_none() && g.site_boxes().is_empty() && g.planner_label().is_none());
    }
}
