//! The opening scene of a new normal world, the hook for its first minute: a few steps in front of the
//! player, on a stone-brick slab, a small line has stopped (a box of ore, belts, a stone furnace with coal,
//! a box for the ingots). One belt has come loose and lies on the grass in front of the gap it left (outlined
//! by the HUD), with ore waiting at its edge. Picking the loose belt up, putting it back and turning it with R
//! (it runs the way the player faced) starts the line, and its last load is the iron and copper for a first miner.
//!
//! Built once by `Sim::build_opening` (from `Game::new_normal`, before the first tick), with plain world
//! edits and `Sim::put_block`. `Opening` (core state, saved since version 47) remembers where it is, so the
//! HUD can say what to do next (`Opening::step`, read-only). Old and creative worlds have none.
//!
//! Invariants: the scene is a pure function of the generator. Progress only moves forward (`Opening::advance`,
//! once a tick, saved since version 57): once the line has run or a miner exists, the tips for putting the belt
//! back never return, however the player later rebuilds the scene.
//! To change the scene: the constants below (cells in the scene's frame: `r` to the right, `d` away from
//! the player, `y` up from the ground the player starts on) and `STOCK`.

use std::f64::consts::FRAC_PI_2;

use crate::block::{
    self, BlockId, AIR, BELT, COAL_ORE, COPPER_ORE, DIRT, GRASS, IRON_ORE, SMELTER, STONE_BRICKS, STORAGE,
};
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
}

/// How far the world's opening has got. It never goes back.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Progress {
    /// The line is still to be mended.
    Start,
    /// The gap belt has been put back the right way round; the ingots are still to be taken.
    Fixed,
    /// Someone took ingots or has a miner: nothing more to teach.
    Over,
}

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
    Done,
}

/// The steps by number for the host (`api/opening.rs`): 0 is done.
pub const STEPS: [Step; 6] = [Step::Done, Step::Pick, Step::Collect, Step::Place, Step::Rotate, Step::Take];

/// What a floating prompt over the scene shows (`Opening::marker`; drawn by `web/src/ui/key-prompt.ts`).
pub const PROMPT_HOLD_LEFT: u8 = 1;
pub const PROMPT_RIGHT_CLICK: u8 = 2;
pub const PROMPT_KEY_R: u8 = 3;
pub const PROMPT_OPEN: u8 = 4;

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
            Step::Take => "*It runs again!* Ingots collect in the box at the end: [Right-click] it to take them.",
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
/// The source box: exactly the ingots for a Miner Mk1 (14 iron, 3 copper) and a copper ingot over.
/// Copper goes in last, so it comes out first and waits at the gap.
const STOCK: [(BlockId, u32); 2] = [(IRON_ORE, 14), (COPPER_ORE, 4)];
/// Coal in the furnace, more than the load needs; what is left is the player's.
const COAL: u32 = 6;
/// Looking down at the loose belt (the gap is straight ahead, the loose belt in line before it), so the first
/// thing aimed at is what to pick up.
pub const PITCH: f64 = -0.42;

impl Opening {
    /// What the player should do now, from the progress and what the scene and their inventory show. The belt
    /// steps only show before the line was first mended, and nothing shows once they have a miner.
    pub fn step(&self, inv: &Inventory, f: &Factory) -> Step {
        if self.progress == Progress::Over || f.count(Kind::Miner) > 0 || inv.count(block::MINER.into()) > 0 {
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
        if !holding && (holds(self.source) || holds(self.out)) {
            Step::Take
        } else {
            Step::Done
        }
    }

    /// Where the floating prompt for `step` goes (the middle of the cell it points at) and what it shows
    /// (`PROMPT_*`).
    pub fn marker(&self, step: Step) -> Option<(IVec3, u8)> {
        match step {
            Step::Pick => Some((self.loose, PROMPT_HOLD_LEFT)),
            Step::Place => Some((self.gap, PROMPT_RIGHT_CLICK)),
            Step::Rotate => Some((self.gap, PROMPT_KEY_R)),
            Step::Take => Some((self.out, PROMPT_OPEN)),
            Step::Collect | Step::Done => None,
        }
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
        if f.count(Kind::Miner) > 0 || has(block::MINER.into()) {
            self.progress = Progress::Over;
        } else if self.progress == Progress::Start {
            if f.placed_as(self.gap).is_some_and(|(dir, _)| dir == self.run()) {
                self.progress = Progress::Fixed;
            }
        } else if has(IRON_INGOT) || has(COPPER_INGOT) {
            self.progress = Progress::Over;
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
            w.u8(o.progress as u8);
        }
    }

    /// Before version 57 there was no progress byte: the scene starts at `Start` (see `migrate`).
    pub fn read_state(r: &mut ByteReader) -> Option<Option<Opening>> {
        if !r.bool()? {
            return Some(None);
        }
        let (gap, loose, source, out) = (r.ivec3()?, r.ivec3()?, r.ivec3()?, r.ivec3()?);
        let progress = if r.version >= 57 {
            [Progress::Start, Progress::Fixed, Progress::Over].get(r.u8()? as usize).copied()?
        } else {
            Progress::Start
        };
        Some(Some(Opening { gap, loose, source, out, progress }))
    }
}

impl Sim {
    /// Builds the opening scene in front of the spawn column (0, 0), facing whichever way the ground is
    /// flattest, and returns the yaw that looks at it. Call once, on a new world before its first tick.
    pub fn build_opening(&mut self) -> f64 {
        let gen = self.world.generator();
        let ground = gen.height_at(0, 0);
        let k = (0..4).min_by_key(|&k| roughness(gen, k, ground)).unwrap_or(0);
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
        if let Some(slots) = self.factory.box_slots_mut(line(0)) {
            for (ore, n) in STOCK {
                add_to_slots(slots, ore.into(), n);
            }
        }
        let furnace = LINE.iter().position(|&b| b == SMELTER).unwrap_or(0) as i32;
        self.factory.insert(line(furnace), ItemId::from(COAL_ORE), COAL);
        // Building it is not news: no sounds or drops on the first tick.
        self.events.clear();
        self.opening = Some(Opening {
            gap: line(GAP),
            loose,
            source: line(0),
            out: line(LINE.len() as i32 - 1),
            progress: Progress::Start,
        });
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

#[cfg(test)]
mod tests;
