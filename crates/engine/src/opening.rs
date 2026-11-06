//! The opening scene of a new normal world, the hook for its first minute: a few steps in front of the
//! player, on a stone-brick slab, a small line has stopped (a box of ore, belts, a stone furnace with coal,
//! a box for the ingots). One belt has come loose and lies on the grass in front of the gap it left (outlined
//! by the HUD), with ore waiting at its edge. Picking the loose belt up, putting it back and turning it with R
//! (it runs the way the player faced) starts the line. Its load is the iron for two coal miners, and the box at the
//! end already holds their stone and the coal to light them. Then the guide (`opening/miners.rs`) walks the player
//! through crafting them and setting one on the starter coal patch and one on the starter iron patch.
//!
//! Built once by `Sim::build_opening` (from `Game::new_normal`, before the first tick), with plain world
//! edits and `Sim::put_block`. `Opening` (core state, saved since version 47) remembers where it is, so the
//! HUD can say what to do next (`Opening::step`, read-only). Old and creative worlds have none.
//!
//! Invariants: the scene is a pure function of the generator. Progress only moves forward (`Opening::advance`,
//! once a tick, saved since version 57): once the line has run, the tips for putting the belt back never return,
//! however the player later rebuilds the scene; once both coal miners burn, or an electric miner exists, nothing does.
//! Scenes from before version 67 have no patches (`coal`, `iron`) and end when the ingots are taken, as they did.
//! To change the scene: the constants below (cells in the scene's frame: `r` to the right, `d` away from
//! the player, `y` up from the ground the player starts on) and `STOCK`.

use std::f64::consts::FRAC_PI_2;

use crate::block::{self, BlockId, AIR, BELT, COAL_ORE, DIRT, GRASS, IRON_ORE, SMELTER, STONE, STONE_BRICKS, STORAGE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::{Factory, Kind, DIRS};
use crate::inventory::{add_to_slots, Inventory};
use crate::item::{ItemId, COPPER_INGOT, IRON_INGOT};
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim};
use crate::worldgen::WorldGen;

/// Where the scene lies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Opening {
    /// The cell the loose belt belongs in.
    pub gap: IVec3,
    /// Where the loose belt lies.
    pub loose: IVec3,
    /// The box the ore comes from and the box the ingots collect in.
    pub source: IVec3,
    pub out: IVec3,
    pub progress: Progress,
    /// The top blocks of the starter coal and iron patches, where the coal miners go (`opening/miners.rs`). None in
    /// scenes from before version 67.
    pub coal: Option<IVec3>,
    pub iron: Option<IVec3>,
}

/// How far the world's opening has got. It never goes back.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Progress {
    /// The line is still to be mended.
    Start,
    /// The gap belt has been put back the right way round; the ingots are still to be taken.
    Fixed,
    /// The ingots are taken: two coal miners are still to be set up (`opening/miners.rs`).
    Mining,
    /// Both coal miners burn, or someone has an electric miner: nothing more to teach.
    Over,
}

/// `Progress` by its saved byte (Mining came last, in version 67).
const PROGRESS_BYTES: [Progress; 4] = [Progress::Start, Progress::Fixed, Progress::Over, Progress::Mining];

/// What the opening asks of the player now.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    /// Break the loose belt.
    Pick,
    /// Walk over the dropped belt.
    Collect,
    /// Put it in the gap.
    Place,
    /// Turn it (R) until it runs the way the line does.
    Rotate,
    /// Take the ingots from the box at the end.
    Take,
    /// Craft a coal miner.
    Craft,
    /// Set it against the starter coal patch.
    PlaceCoal,
    /// Give the coal miner on coal its first fuel.
    FuelCoal,
    /// Set the second against the starter iron patch.
    PlaceIron,
    /// Give the coal miner on iron fuel.
    FuelIron,
    Done,
    /// Craft the second coal miner, the first standing on coal.
    CraftIron,
    /// Give the lit coal miner on coal a box beside it (or a belt away) to fill.
    BoxCoal,
}

/// The steps by number for the host (`api/opening.rs`): 0 is done. Append only.
pub const STEPS: [Step; 13] = [
    Step::Done,
    Step::Pick,
    Step::Collect,
    Step::Place,
    Step::Rotate,
    Step::Take,
    Step::Craft,
    Step::PlaceCoal,
    Step::FuelCoal,
    Step::PlaceIron,
    Step::FuelIron,
    Step::CraftIron,
    Step::BoxCoal,
];

/// What a floating prompt over the scene shows (`Opening::marker`; drawn by `web/src/ui/key-prompt.ts`).
pub const PROMPT_HOLD_LEFT: u8 = 1;
pub const PROMPT_RIGHT_CLICK: u8 = 2;
pub const PROMPT_KEY_R: u8 = 3;
pub const PROMPT_OPEN: u8 = 4;
/// 5 is the stall lesson's (`stall_lesson.rs`).
pub const PROMPT_COAL: u8 = 6;
pub const PROMPT_IRON: u8 = 7;
/// The copper patch while the Go electric tip shows (`first_steps.rs`).
pub const PROMPT_COPPER: u8 = 8;

impl Step {
    /// The HUD tip for this step (empty when done). `*words*` are highlighted and `[keys]` drawn as keys
    /// (`web/src/ui/hints.ts`).
    pub fn text(self) -> &'static str {
        match self {
            Step::Pick => "*The line has stopped:* a belt came loose. Hold [Left mouse] on it to pick it up.",
            Step::Collect => "*Got it loose.* Walk over the belt to pick it up.",
            Step::Place => "*Put it back:* select the belt and [Right-click] the gap outlined in blue.",
            Step::Rotate => {
                "*Wrong way round!* A belt runs the way you face. Look at it and press [R] to turn it until its \
                 arrows point to the furnace."
            }
            Step::Take => {
                "*It runs again!* Ingots collect in the box at the end, where stone and coal wait too: [Right-click] \
                 it and press *Take all*."
            }
            Step::Craft => {
                "*Build a coal miner:* press [E], open the glowing *Production* tab and click the *Coal Miner* (its \
                 iron parts queue up by themselves). You have enough for two. It burns coal instead of power."
            }
            Step::PlaceCoal => {
                "*Coal first:* walk to the coal patch (dark speckled rock) where the marker floats ([M] shows it on \
                 the map), select the coal miner and [Right-click] a coal block to set it against it."
            }
            Step::BoxCoal => {
                "*Catch the coal:* a coal miner keeps what it digs until it is full, then stops. Give it a *storage \
                 box* to fill: take the box at the start of the line, its work done (hold [Left mouse] on it), then \
                 [Right-click] the ground right beside the miner, where the outline shows."
            }
            Step::CraftIron => {
                "*Another coal miner, for iron:* press [E] and craft a second *Coal Miner* in the *Production* tab. \
                 Your first one digs coal; this one will dig iron."
            }
            Step::FuelCoal => {
                "*Light it:* [Right-click] the coal miner and put coal ore in its fuel slot. On coal it keeps its \
                 own fire going, so one is enough: keep the rest for your second miner."
            }
            Step::PlaceIron => {
                "*Now iron:* walk to the iron patch (rock speckled rusty brown) where the marker floats ([M] shows it \
                 on the map) and [Right-click] an iron ore block with your second coal miner."
            }
            Step::FuelIron => {
                "*Fuel it:* [Right-click] it and give it coal. It digs iron, not coal, so keep it fed: the box beside \
                 the miner on coal fills up with coal you can take."
            }
            Step::Done => "",
        }
    }
}
/// How far the slab and the cleared ground reach: `r` from..=to, `d` from..=to.
const PAD_R: (i32, i32) = (-6, 5);
const PAD_D: (i32, i32) = (1, 7);
/// The line runs along this `d`, from `LINE_R` to the right, on a slab one cell wider all round.
const LINE_D: i32 = 5;
const LINE_R: i32 = -3;
/// The line from left to right: a box, belts, the gap, the furnace, a belt and the box for the ingots.
const LINE: [BlockId; 8] = [STORAGE, BELT, BELT, AIR, BELT, SMELTER, BELT, STORAGE];
/// Where in `LINE` the gap is, and where the loose belt lies (`d`) in front of it.
const GAP: i32 = 3;
const LOOSE_D: i32 = 3;
/// Cells cleared above the ground, and filled below it where the ground dips.
const CLEAR: i32 = 12;
const FILL: i32 = 4;
/// The source box: the iron for two coal miners, and no copper, so nothing tempts a miner that needs power
/// (`the_scene_pays_for_two_coal_miners` checks it against the recipe).
const STOCK: [(BlockId, u32); 1] = [(IRON_ORE, 22)];
/// Waiting in the box for the ingots: the stone for two coal miners and coal to light them and keep the one on iron
/// going for a few minutes.
const OUT_STOCK: [(BlockId, u32); 2] = [(STONE, 16), (COAL_ORE, 8)];
/// Coal in the furnace, more than the load needs; what is left is the player's.
const COAL: u32 = 8;
/// Looking down at the loose belt (the gap is straight ahead, the loose belt in line before it), so the first
/// thing aimed at is what to pick up.
pub const PITCH: f64 = -0.42;

impl Opening {
    /// What the player should do now, from the progress and what the scene and their inventory show. The belt
    /// steps only show before the line was first mended, and nothing shows once the coal miners burn.
    pub fn step(&self, inv: &Inventory, f: &Factory) -> Step {
        if self.progress == Progress::Over || self.ended(f, inv.count(block::MINER.into()) > 0) {
            return Step::Done;
        }
        if self.progress == Progress::Start {
            let is_belt = |pos| f.tiered_at(pos).is_some_and(|t| t.0 == BELT);
            if !is_belt(self.gap) {
                return if is_belt(self.loose) {
                    Step::Pick
                } else if inv.count(BELT.into()) > 0 {
                    Step::Place
                } else {
                    Step::Collect
                };
            }
            if f.placed_as(self.gap).is_some_and(|(dir, _)| dir != self.run()) {
                return Step::Rotate;
            }
        }
        let holds = |pos| f.box_slots(pos).is_some_and(|s| s.iter().any(|s| s.count > 0));
        let holding = inv.count(IRON_INGOT) > 0 || inv.count(COPPER_INGOT) > 0;
        if self.progress < Progress::Mining && !holding && (holds(self.source) || holds(self.out)) {
            Step::Take
        } else {
            self.mining_step(inv, f)
        }
    }

    /// Where the floating prompt for `step` goes (the middle of the cell it points at) and what it shows
    /// (`PROMPT_*`).
    pub fn marker(&self, step: Step, f: &Factory) -> Option<(IVec3, u8)> {
        match step {
            Step::Pick => Some((self.loose, PROMPT_HOLD_LEFT)),
            Step::Place => Some((self.gap, PROMPT_RIGHT_CLICK)),
            Step::Rotate => Some((self.gap, PROMPT_KEY_R)),
            Step::Take => Some((self.out, PROMPT_OPEN)),
            Step::PlaceCoal | Step::FuelCoal | Step::PlaceIron | Step::FuelIron | Step::BoxCoal => {
                self.mining_marker(step, f)
            }
            Step::Collect | Step::Craft | Step::CraftIron | Step::Done => None,
        }
    }

    /// Whether the opening is over whatever its progress: an electric miner stands or is `held` (the player went
    /// their own way), both coal miners burn, or (a scene from before the guide) any miner stands.
    fn ended(&self, f: &Factory, held: bool) -> bool {
        let old = self.coal.is_none() && f.count(Kind::Miner) > 0;
        held || old || f.electric_miners() > 0 || self.miners_done(f)
    }

    /// The way the line runs (a `DIRS` index): from the source box towards the ingot box.
    pub fn run(&self) -> u8 {
        let d = self.out - self.source;
        let unit = IVec3::new(d.x.signum(), 0, d.z.signum());
        DIRS.iter().position(|&x| x == unit).unwrap_or(0) as u8
    }

    /// Moves the progress forward from the core state, once a tick. `inventories` is every player's. Cheap
    /// once over.
    pub fn advance<'a>(&mut self, f: &Factory, inventories: impl Iterator<Item = &'a Inventory> + Clone) {
        if self.progress == Progress::Over {
            return;
        }
        let has = |item: ItemId| inventories.clone().any(|inv| inv.count(item) > 0);
        if self.ended(f, has(block::MINER.into())) {
            self.progress = Progress::Over;
        } else if self.progress == Progress::Start {
            if f.placed_as(self.gap).is_some_and(|(dir, _)| dir == self.run()) {
                self.progress = Progress::Fixed;
            }
        } else if self.progress == Progress::Fixed && (has(IRON_INGOT) || has(COPPER_INGOT)) {
            self.progress = if self.coal.is_some() { Progress::Mining } else { Progress::Over };
        }
    }

    /// A scene from before progress was saved (version 57): when neither the gap nor the loose belt is a belt, the
    /// player is past the belt steps, or took the belts away to use elsewhere.
    pub fn migrate(&mut self, f: &Factory) {
        let is_belt = |pos| f.tiered_at(pos).is_some_and(|t| t.0 == BELT);
        if !is_belt(self.gap) && !is_belt(self.loose) {
            self.progress = Progress::Over;
        }
    }

    pub fn write_state(this: &Option<Opening>, w: &mut ByteWriter) {
        w.bool(this.is_some());
        if let Some(o) = this {
            [o.gap, o.loose, o.source, o.out].into_iter().for_each(|p| w.ivec3(p));
            w.u8(PROGRESS_BYTES.iter().position(|&p| p == o.progress).unwrap_or(0) as u8);
            for patch in [o.coal, o.iron] {
                w.bool(patch.is_some());
                if let Some(p) = patch {
                    w.ivec3(p);
                }
            }
        }
    }

    /// Before version 57 there was no progress byte: the scene starts at `Start` (see `migrate`). Before version 67
    /// there were no patches.
    pub fn read_state(r: &mut ByteReader) -> Option<Option<Opening>> {
        if !r.bool()? {
            return Some(None);
        }
        let (gap, loose, source, out) = (r.ivec3()?, r.ivec3()?, r.ivec3()?, r.ivec3()?);
        let progress = if r.version >= 57 { PROGRESS_BYTES.get(r.u8()? as usize).copied()? } else { Progress::Start };
        let mut patch = || -> Option<Option<IVec3>> {
            if r.version < 67 || !r.bool()? {
                return Some(None);
            }
            Some(Some(r.ivec3()?))
        };
        let (coal, iron) = (patch()?, patch()?);
        Some(Some(Opening { gap, loose, source, out, progress, coal, iron }))
    }
}

impl Sim {
    /// Builds the opening scene in front of the spawn column (0, 0), facing whichever way the ground is
    /// flattest, and returns the yaw that looks at it. Call once, on a new world before its first tick.
    pub fn build_opening(&mut self) -> f64 {
        let gen = self.world.generator();
        let ground = gen.height_at(0, 0);
        let k = (0..4).min_by_key(|&k| roughness(gen, k, ground)).unwrap_or(0);
        let patches = [COAL_ORE, IRON_ORE].map(|ore| gen.starter_patch(ore).map(|top| (top, ore)));
        let [coal, iron] = patches.map(|p| p.map(|(top, ore)| miners::open_top(&mut self.world, top, ore)));
        let (fwd, right) = (DIRS[k], DIRS[(k + 1) % 4]);
        let at = |r: i32, d: i32, y: i32| right * r + fwd * d + IVec3::new(0, ground + y, 0);
        let slab_r = (LINE_R - 1, LINE_R + LINE.len() as i32);
        for r in PAD_R.0..=PAD_R.1 {
            for d in PAD_D.0..=PAD_D.1 {
                for y in 1..=CLEAR {
                    self.world.set_block_anywhere(at(r, d, y), AIR);
                }
                for y in -FILL..0 {
                    if !block::SOLID[self.world.block_anywhere_or_generate(at(r, d, y)) as usize] {
                        self.world.set_block_anywhere(at(r, d, y), DIRT);
                    }
                }
                let slab = (slab_r.0..=slab_r.1).contains(&r) && (LINE_D - 1..=LINE_D + 1).contains(&d);
                self.world.set_block_anywhere(at(r, d, 0), if slab { STONE_BRICKS } else { GRASS });
            }
        }
        let run = ((k + 1) % 4) as u8;
        let line = |i: i32| at(LINE_R + i, LINE_D, 1);
        for (i, &b) in LINE.iter().enumerate().filter(|(_, &b)| b != AIR) {
            let facing = if b == BELT { run } else { k as u8 };
            self.put_block(PlayerId(0), line(i as i32), b.into(), facing, line(i as i32));
        }
        let loose = at(LINE_R + GAP, LOOSE_D, 1);
        self.put_block(PlayerId(0), loose, BELT.into(), ((k + 2) % 4) as u8, loose);
        let last = line(LINE.len() as i32 - 1);
        for (pos, stock) in [(line(0), &STOCK[..]), (last, &OUT_STOCK[..])] {
            if let Some(slots) = self.factory.box_slots_mut(pos) {
                for &(item, n) in stock {
                    add_to_slots(slots, item.into(), n);
                }
            }
        }
        let furnace = LINE.iter().position(|&b| b == SMELTER).unwrap_or(0) as i32;
        self.factory.insert(line(furnace), ItemId::from(COAL_ORE), COAL);
        // Building it is not news: no sounds or drops on the first tick.
        self.events.clear();
        self.opening =
            Some(Opening { gap: line(GAP), loose, source: line(0), out: last, progress: Progress::Start, coal, iron });
        k as f64 * FRAC_PI_2
    }
}

/// How far the scene's ground facing `DIRS[k]` lies from `ground`, summed over its columns.
fn roughness(gen: &WorldGen, k: usize, ground: i32) -> i32 {
    let (fwd, right) = (DIRS[k], DIRS[(k + 1) % 4]);
    let mut sum = 0;
    for r in PAD_R.0..=PAD_R.1 {
        for d in PAD_D.0..=PAD_D.1 {
            let c = right * r + fwd * d;
            sum += (gen.height_at(c.x, c.z) - ground).abs();
        }
    }
    sum
}

mod miners;
pub(crate) use miners::{box_spot, on as miners_on};
#[cfg(test)]
mod tests;
