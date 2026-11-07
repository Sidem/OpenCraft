//! How a forester looks (presentation only). The machine: a yellow carrier on outriggers with a glazed cab, a log bunk
//! with stakes, and a slewing column carrying the crane. The crane and what it holds are drawn each frame from the job
//! and its progress ([`pose`]): `crane.rs` the knuckle boom and harvester head, `scene.rs` the fence round the patch,
//! the logs on the ground and on the bunk, and a red marker where uneven ground stops it.

use crate::block::tex;
use crate::math::Vec3;
use crate::TICK;

use super::super::super::power::FULL_SPEED;
use super::super::super::{ticks, DIRS};
use super::super::model::{local, part, round, Look, Part};
use super::super::{Extra, Processor, Status};
use super::{Forest, Job, Patch, Spot, HOME, SIZES};
use super::{
    BUCK_SECONDS, CUT_SECONDS, FELL_SECONDS, GATHER_SECONDS, GRIP_SECONDS, HAUL_SECONDS, PLANT_SECONDS, REACH_SECONDS,
    REST_SECONDS, UNLOAD_SECONDS,
};

/// The carrier's paint (the grid accents' yellow, a flat solid layer).
pub(super) const YELLOW: u16 = tex::GRID_FIRST + 4;
/// The crane's pivot on the slewing column, and the log bunk's middle, in the model's frame (front towards +z, the
/// patch towards -z, floor at -0.5).
const PIVOT: [f32; 3] = [0.0, 0.16, -0.2];
const BUNK: [f64; 3] = [-0.24, -0.12, 0.2];
/// Seconds the crane takes to swing home when it rests.
const PARK_SECONDS: f64 = 1.5;
/// The stem's axis grips this high above the ground; laid down it lies this much higher.
const GRIP_HEIGHT: f64 = 0.55;
const LAID: f64 = 0.45;
/// The head hovers this high over a plot it is about to plant, and over the bunk.
const HOVER: f64 = 1.1;

pub(super) const PARTS: [Part; 17] = [
    part([0.0, -0.44, 0.0], [0.96, 0.12, 0.96], Look::Band(tex::FRAME)),
    part([0.4, -0.4, 0.4], [0.14, 0.18, 0.14], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([-0.4, -0.4, 0.4], [0.14, 0.18, 0.14], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([0.4, -0.4, -0.4], [0.14, 0.18, 0.14], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([-0.4, -0.4, -0.4], [0.14, 0.18, 0.14], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([0.0, -0.27, 0.02], [0.84, 0.22, 0.82], Look::Tex([YELLOW, YELLOW, tex::MAT_GUNMETAL])),
    round([0.0, -0.13, -0.2], [0.44, 0.08, 0.44], Look::Tex([tex::MAT_GUNMETAL; 3])),
    round([0.0, 0.0, -0.2], [0.26, 0.24, 0.26], Look::Tex([YELLOW; 3])),
    part(PIVOT, [0.1, 0.1, 0.1], Look::Crane),
    part([0.25, 0.0, 0.2], [0.36, 0.32, 0.34], Look::Tex([YELLOW; 3])),
    part([0.25, 0.04, 0.375], [0.28, 0.17, 0.02], Look::Tex([tex::CAB_GLASS; 3])),
    part([0.435, 0.04, 0.2], [0.02, 0.17, 0.26], Look::Tex([tex::CAB_GLASS; 3])),
    part([0.25, 0.175, 0.2], [0.4, 0.03, 0.38], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([-0.24, -0.15, 0.2], [0.42, 0.03, 0.42], Look::Tex([tex::FRAME; 3])),
    part([-0.42, 0.0, 0.2], [0.04, 0.28, 0.4], Look::Tex([tex::STEEL; 3])),
    part([-0.06, 0.0, 0.2], [0.04, 0.28, 0.4], Look::Tex([tex::STEEL; 3])),
    part([0.42, -0.14, -0.3], [0.06, 0.08, 0.1], Look::Lamp),
];

/// What the head holds.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Load {
    Empty,
    /// A felled stem, this many logs long still.
    Stem(f64),
    /// A bundle of bucked logs.
    Bundle(u8),
    Sapling,
}

/// Where the head is and how it hangs at render time, camera-relative.
pub(super) struct Pose {
    /// A point on the axis of what the head grips.
    pub head: Vec3,
    /// Level, from the crane out towards the head: the head's arms face along it.
    pub toward: Vec3,
    /// 0 upright (a standing trunk), 1 lying (a stem being fed through).
    pub tilt: f64,
    /// 0 open, 1 closed.
    pub arms: f64,
    /// 0 parked, 1 through the cut.
    pub saw: f64,
    pub load: Load,
}

/// Draws a forester's crane, head and surroundings; `pivot` is the crane part's camera-relative centre.
pub(in crate::factory::process) fn draw_crane(p: &Processor, out: &mut Vec<f32>, rel: Vec3, pivot: Vec3, time: f64) {
    let Extra::Forest(f) = &p.extra else { return };
    let yaw = p.dir as f32 * std::f32::consts::FRAC_PI_2;
    let at = Place { p, f, rel, pivot, bunk: local(rel, yaw, BUNK) };
    let pose = pose(&at, share(p, f, time));
    let (rows, half) = SIZES[usize::from(p.tier).min(SIZES.len() - 1)];
    super::crane::draw(out, pivot, &pose, f64::from(rows).hypot(f64::from(half)));
    super::scene::draw(out, p, f, rel, &pose, at.bunk);
}

/// Where things are for one forester's drawing.
struct Place<'a> {
    p: &'a Processor,
    f: &'a Forest,
    rel: Vec3,
    pivot: Vec3,
    bunk: Vec3,
}

impl Place<'_> {
    fn patch(&self) -> Patch {
        self.f.patch(self.p.pos, self.p.dir)
    }

    /// The middle of the foot cell at `spot` (the bunk at home).
    fn ground(&self, (ahead, side): Spot) -> Vec3 {
        if (ahead, side) == HOME {
            return self.bunk;
        }
        let cell = self.patch().cell(i32::from(ahead), i32::from(side), self.p.pos.y);
        self.rel + (cell - self.p.pos).as_vec3() - Vec3::new(0.0, 0.5, 0.0)
    }

    /// Where the head waits over `spot`.
    fn hover(&self, spot: Spot) -> Vec3 {
        self.ground(spot) + up(if spot == HOME { 0.75 } else { HOVER })
    }

    fn grip(&self, spot: Spot) -> Vec3 {
        self.ground(spot) + up(GRIP_HEIGHT)
    }

    /// Where the logs of the tree at `spot` lie bucked: beside its stump, along the way the stem lay.
    fn pile(&self, spot: Spot) -> Vec3 {
        let toward = toward(self.pivot, self.grip(spot), self.p.dir);
        self.ground(spot) + Vec3::new(-toward.z, 0.0, toward.x) * 1.1 + toward * 0.5 + up(0.6)
    }
}

/// How far through its current step `p` is at render time (0 to 1), blended into the tick as a batch's is.
fn share(p: &Processor, f: &Forest, time: f64) -> f64 {
    let seconds = match f.job {
        Job::Reach | Job::Fetch => REACH_SECONDS,
        Job::Grip => GRIP_SECONDS,
        Job::Cut => CUT_SECONDS,
        Job::Fell => FELL_SECONDS,
        Job::Buck => BUCK_SECONDS,
        Job::Gather => GATHER_SECONDS,
        Job::Haul => HAUL_SECONDS,
        Job::Unload => UNLOAD_SECONDS,
        Job::Plant => PLANT_SECONDS,
        Job::Rest | Job::Blocked => {
            let gone = f64::from(ticks(REST_SECONDS).saturating_sub(u32::from(f.step)));
            return (gone / f64::from(ticks(PARK_SECONDS))).min(1.0);
        }
        Job::Look => return 1.0,
    };
    let rate = f64::from(p.stats().speed) * f64::from(p.speed) / 1e3;
    let ahead = if p.status == Status::Working { (time / TICK).fract() * rate } else { 0.0 };
    ((f64::from(p.progress) + ahead) / f64::from(ticks(seconds) * FULL_SPEED)).min(1.0)
}

/// The head's pose in `f`'s job, `s` of the way through it.
fn pose(at: &Place, s: f64) -> Pose {
    let f = at.f;
    let mut pose =
        Pose { head: at.hover(f.rest), toward: Vec3::ZERO, tilt: 0.0, arms: 0.0, saw: 0.0, load: Load::Empty };
    let laid = at.grip(f.tree) + up(LAID);
    let stem = f64::from(f.stem) - f64::from(f.cut);
    match f.job {
        Job::Look => {}
        Job::Rest | Job::Blocked => pose.head = swing(at.hover(f.from), at.hover(HOME), s),
        Job::Reach => pose.head = swing(at.hover(f.from), at.grip(f.tree), s),
        Job::Grip | Job::Cut => {
            pose.head = at.grip(f.tree);
            pose.arms = if f.job == Job::Grip { s } else { 1.0 };
            pose.saw = if f.job == Job::Cut { s } else { 0.0 };
        }
        Job::Fell => {
            (pose.head, pose.tilt, pose.arms) = (at.grip(f.tree) + up(LAID * s), s, 1.0);
            pose.load = Load::Stem(stem);
        }
        Job::Buck => {
            (pose.head, pose.tilt, pose.arms) = (laid, 1.0, 1.0);
            pose.load = Load::Stem(stem - s);
            pose.saw = ((s - 0.8) / 0.2).clamp(0.0, 1.0);
        }
        Job::Gather => {
            (pose.head, pose.tilt, pose.arms) = (swing(laid, at.pile(f.tree), s), 1.0, s);
            pose.load = if s > 0.7 { Load::Bundle(f.cut) } else { Load::Empty };
        }
        Job::Haul => {
            (pose.head, pose.tilt, pose.arms) = (swing(at.pile(f.tree), at.hover(HOME), s), 1.0, 1.0);
            pose.load = Load::Bundle(f.cut);
        }
        Job::Unload => {
            (pose.head, pose.tilt) = (at.hover(HOME) - up(0.3 * s), 1.0);
            (pose.arms, pose.load) = if s < 0.6 { (1.0, Load::Bundle(f.cut)) } else { (0.0, Load::Empty) };
        }
        Job::Fetch if s < 0.5 => pose.head = swing(at.hover(f.from), at.hover(HOME), 2.0 * s),
        Job::Fetch => {
            (pose.head, pose.arms) = (swing(at.hover(HOME), at.hover(f.tree), 2.0 * s - 1.0), 1.0);
            pose.load = Load::Sapling;
        }
        Job::Plant => {
            (pose.head, pose.arms) = (at.hover(f.tree) - up((HOVER - 0.45) * s), 1.0 - s);
            pose.load = if s < 0.9 { Load::Sapling } else { Load::Empty };
        }
    }
    pose.toward = toward(at.pivot, pose.head, at.p.dir);
    pose
}

/// From `a` to `b`, `s` of the way, eased and lifted in an arc over what lies between.
fn swing(a: Vec3, b: Vec3, s: f64) -> Vec3 {
    let e = s * s * (3.0 - 2.0 * s);
    let lift = (0.6 + 0.08 * (b - a).length()) * (s * std::f64::consts::PI).sin();
    a + (b - a) * e + up(lift)
}

/// Level and unit, from the crane's pivot towards `head`; straight out over the patch when the head is over the pivot.
pub(super) fn toward(pivot: Vec3, head: Vec3, dir: u8) -> Vec3 {
    let flat = Vec3::new(head.x - pivot.x, 0.0, head.z - pivot.z);
    if flat.length() > 0.3 {
        flat * (1.0 / flat.length())
    } else {
        DIRS[dir as usize % 4].as_vec3()
    }
}

pub(super) fn up(h: f64) -> Vec3 {
    Vec3::new(0.0, h, 0.0)
}
