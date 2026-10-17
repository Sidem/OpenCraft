use crate::block;
use crate::Game;

#[test]
fn the_host_reads_a_logbook_page_and_nothing_else() {
    let g = Game::new_normal(1337, 4);
    let p = g.sim.world.generator().first_logbook().expect("a first ruin");
    let page = g.logbook(p.x, p.y, p.z);
    let mut lines = page.lines();
    assert!(lines.next().unwrap().starts_with("Crew log, day "));
    assert!(lines.next().is_some_and(|t| !t.is_empty()));
    if let Some(pin) = lines.next() {
        let f: Vec<&str> = pin.split('\t').collect();
        assert_eq!(f[0], "pin");
        assert!(f[1].parse::<i32>().is_ok() && f[2].parse::<i32>().is_ok());
        assert!(block::is_ore(f[3].parse().unwrap()));
    }
    assert!(g.logbook(p.x + 1, p.y, p.z).is_empty());
    assert!(g.logbook(0, 64, 0).is_empty());
}
