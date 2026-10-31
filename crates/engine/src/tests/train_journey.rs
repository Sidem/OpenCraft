//! The journey (DEV_PLAN T5): a bot that does only what the HUD says builds a freight line on uneven ground, and the
//! clicks it took are counted. Its intent is a player's (a loading dock here, an unloading dock over there, a train
//! with two wagons); every move it makes is the one the label names, aimed at the cells the outlines mark. A label
//! that names no move, or a click that changes nothing, is a dead end and fails the test.
//!
//! HEAD before the train work (cd89c09) took 7 clicks for the same line on any ground: 2 rail nodes, 2 docks beside
//! them, a locomotive and 2 wagons, with the track floating over the dip and running through the hump.

use super::*;
use crate::block::{AIR, DIRT, LOADING_DOCK, RAIL, UNLOADING_DOCK};
use crate::factory::Kind;
use crate::item::{ItemId, IRON_INGOT, LOCOMOTIVE, WAGON};
use crate::math::{IVec3, Vec3};

const RAILS: ItemId = ItemId(RAIL as u16);
const UP: IVec3 = IVec3::new(0, 1, 0);
const FILL_AMBER: i32 = 0xfbbf24;
const BLOCKED_RED: i32 = 0xff4a3d;
const SIDES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
];

/// The bot and what it counted.
struct Bot {
    g: Game,
    clicks: u32,
    earthwork: u32,
    labels: Vec<String>,
}

impl Bot {
    fn label(&self) -> String {
        self.g.line_label()
    }

    fn hold(&mut self, item: ItemId) {
        let slot =
            self.g.inventory().slots.iter().position(|s| s.item == item && s.count > 0).expect("in the backpack");
        self.g.select_slot(slot as u32);
        self.g.run_ticks(1);
    }

    /// Looks from `eye` at `at` (the ray the player's crosshair casts).
    fn look(&mut self, eye: Vec3, at: Vec3) {
        let height = self.g.body().eye().y - self.g.body().pos.y;
        self.g.body_mut().pos = eye - Vec3::new(0.0, height, 0.0);
        let d = at - eye;
        self.g.set_look(d.x.atan2(-d.z), (d.y / d.length()).asin());
        self.g.update_target();
    }

    /// Looks down at a point from a little behind and above it, as a player walking the line does.
    fn look_down_at(&mut self, at: Vec3) {
        self.look(at + Vec3::new(-2.0, 2.2, 0.6), at);
    }

    /// Items in the backpack: every building click uses one up.
    fn carried(&self) -> u32 {
        self.g.inventory().slots.iter().map(|s| s.count).sum()
    }

    /// One press and release of the use button (with Shift when `shift`). A click that builds nothing (uses up
    /// nothing in hand) is a dead end.
    fn click(&mut self, shift: bool) {
        let before = self.carried();
        self.g.body_mut().input.sprint = shift;
        for on in [true, false] {
            self.g.set_using(on);
            self.g.update_placing(1.0 / 60.0);
            self.g.sim.step();
        }
        self.g.body_mut().input.sprint = false;
        self.clicks += 1;
        assert!(self.carried() < before, "a dead end: the click built nothing (label {:?})", self.labels.last());
    }

    /// The cells the outlines mark in `colour`.
    fn outlined(&self, colour: i32) -> Vec<IVec3> {
        self.g
            .placement_box()
            .chunks_exact(7)
            .filter(|b| b[6] == colour)
            .map(|b| IVec3::new(b[0], b[1], b[2]))
            .collect()
    }

    /// Places dirt into `cell` against a solid neighbour: one click.
    fn fill(&mut self, cell: IVec3) {
        self.hold(DIRT.into());
        for n in SIDES {
            let wall = cell - n;
            if !crate::block::SOLID[self.g.sim.world.get_block(wall).unwrap_or(AIR) as usize] {
                continue;
            }
            // From above the cell (open: it needs filling, and clear of the bot's own body), looking down at the face.
            let centre = cell.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
            let lean = if n.y == 0 { n.as_vec3() * 0.3 } else { Vec3::new(0.3, 0.0, 0.2) };
            self.look(centre + Vec3::new(0.0, 2.4, 0.0) + lean, centre - n.as_vec3() * 0.5);
            if self.g.target.is_some_and(|h| h.block == wall && h.block + h.normal == cell) {
                self.click(false);
                self.earthwork += 1;
                return;
            }
        }
        panic!("no face to place against for {cell:?}");
    }

    /// Mines `cell` by hand from an open side: one press, held until it breaks.
    fn clear(&mut self, cell: IVec3) {
        let centre = cell.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        let open =
            SIDES.into_iter().find(|&n| self.g.sim.world.get_block(cell + n) == Some(AIR)).expect("an open side");
        self.look(centre + open.as_vec3() * 1.6 + Vec3::new(0.1, 0.2, 0.1), centre);
        assert_eq!(self.g.target.map(|h| h.block), Some(cell));
        self.g.mining = true;
        for _ in 0..600 {
            self.g.update_mining(1.0 / 60.0);
            self.g.sim.step();
            if self.g.sim.world.get_block(cell) == Some(AIR) {
                break;
            }
        }
        self.g.mining = false;
        assert_eq!(self.g.sim.world.get_block(cell), Some(AIR), "mined");
        self.clicks += 1;
        self.earthwork += 1;
    }

    /// Holding `item` and aiming with `aim`, does what the label says until it says to click, then clicks. Earthwork
    /// in between follows the outlines: amber cells filled, red ones cleared.
    fn follow(&mut self, item: ItemId, aim: impl Fn(&mut Bot)) {
        for _ in 0..40 {
            self.hold(item);
            aim(self);
            let label = self.label();
            self.labels.push(label.clone());
            let text = label.lines().nth(1).unwrap_or("");
            if text.contains("fill") || text.contains("clear") {
                let (fill, clear) = (self.outlined(FILL_AMBER), self.outlined(BLOCKED_RED));
                assert!(!fill.is_empty() || !clear.is_empty(), "the label asks for earthwork the outlines do not show");
                if let Some(&cell) = clear.iter().find(|c| self.g.sim.world.get_block(**c) != Some(AIR)) {
                    self.clear(cell);
                } else if let Some(&cell) = fill.first() {
                    self.fill(cell);
                }
            } else if text.contains("Shift-click: every wagon") {
                return self.click(true);
            } else if text.starts_with("click to") {
                return self.click(false);
            } else {
                panic!("a dead end: {label:?}");
            }
        }
        panic!("no end to the earthwork: {:?}", self.labels.last());
    }
}

#[test]
fn a_bot_following_the_labels_builds_a_freight_line_on_uneven_ground() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    // Ground at y, a hump two blocks high across the line, a dip two deep, and a step up a block at the far end.
    for x in -6..=40 {
        for z in -8..=8 {
            let top = match x {
                12..=13 => y + 2,
                20..=21 => y - 2,
                28.. => y + 1,
                _ => y,
            };
            for h in y - 4..=y + 8 {
                g.sim.world.set_block_anywhere_later(IVec3::new(x, h, z), if h <= top { DIRT } else { AIR });
            }
        }
    }
    for (item, n) in [(RAILS, 12), (LOADING_DOCK.into(), 1), (UNLOADING_DOCK.into(), 1), (LOCOMOTIVE, 1)] {
        g.give(item.0, n);
    }
    g.give(WAGON.0, 2);
    g.give(DIRT.into(), 32);
    g.run_ticks(2);
    let mut bot = Bot { g, clicks: 0, earthwork: 0, labels: Vec::new() };
    let start = IVec3::new(2, y + 1, 0);
    let far = IVec3::new(32, y + 2, 0);
    let top_of = |c: IVec3| c.as_vec3() + Vec3::new(0.5, 0.0, 0.5);

    // A rail node where the line starts, its track running east (the way the bot faces).
    bot.follow(RAILS, |b| b.look(top_of(start) + Vec3::new(-3.0, 1.6, 0.0), top_of(start)));
    // The loading dock onto that node, then the unloading dock laid in line at the far end.
    bot.follow(LOADING_DOCK.into(), |b| b.look_down_at(top_of(start) + Vec3::new(0.0, 0.1, 0.0)));
    bot.follow(UNLOADING_DOCK.into(), |b| b.look_down_at(top_of(far - UP) + Vec3::new(0.0, 1.0, 0.0)));
    // The locomotive onto the loading dock, then both wagons.
    bot.follow(LOCOMOTIVE, |b| b.look_down_at(top_of(start) + Vec3::new(0.0, 0.1, 0.0)));
    bot.follow(WAGON, |b| b.look_down_at(top_of(start) + Vec3::new(0.0, 0.1, 0.0)));

    let builds = bot.clicks - bot.earthwork;
    println!("journey: {builds} building clicks + {} earthwork, labels: {:#?}", bot.earthwork, bot.labels);
    assert_eq!(builds, 5, "1 node, 2 docks, a locomotive and Shift for both wagons (HEAD: 7)");
    assert!(bot.earthwork > 0, "the ground was uneven");

    // The line works: ingots fed into the loading dock reach a box beside the unloading dock.
    let g = &mut bot.g;
    let f = &mut g.sim.factory;
    assert_eq!(f.count(Kind::Process), 2, "two docks");
    assert_eq!(f.train_by_id(1).map(|t| t.cars), Some(2), "a train with two wagons");
    let (feed, out) = (start + IVec3::new(0, 0, 2), far + IVec3::new(0, 0, 2));
    f.add_storage(feed);
    f.stock(feed, IRON_INGOT, 40);
    f.add_belt(start + IVec3::new(0, 0, 1), 0);
    f.add_belt(far + IVec3::new(0, 0, 1), 2);
    f.add_storage(out);
    for _ in 0..60 * 120 {
        g.sim.step();
        if g.sim.factory.box_slots(out).is_some_and(|s| s.iter().any(|s| s.item == IRON_INGOT)) {
            return;
        }
    }
    panic!("no ingots reached the far box");
}
