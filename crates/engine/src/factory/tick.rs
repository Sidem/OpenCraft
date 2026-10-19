//! One factory tick (`Factory::update`), in a fixed order: sensors and trains, beams and links when stale, then power
//! (`power.rs`: supply and demand) and data, miners, quarries, boxes, processing machines, pods, routers, labs, pipework,
//! and last the belts (downstream first, see `links.rs`). Core state (DEV_PLAN section 3.4): it reports to the view only
//! through `SimEvent`s. To run a new kind each tick: its step goes here, in the place its inputs and outputs need.

use super::*;
use crate::research::Bonus;
use crate::satellites::COMMS;
use crate::TICK;
use belt::belt_step;
use lab::step_labs;
use links::Sinks;

impl Factory {
    /// Runs every machine for one tick (`TICK` seconds). `tick` must differ between calls: the
    /// deposits' shared draw budgets refill once per tick.
    pub fn update(&mut self, world: &mut World, tick: u64, events: &mut Vec<SimEvent>) {
        self.step_sensors();
        self.step_trains();
        if self.dirty || self.beams.stale {
            self.aim_beams(world);
        }
        if self.dirty {
            self.relink();
        }
        let Factory {
            belts,
            miners,
            storages,
            processors,
            routers,
            generators,
            labs,
            pipework,
            quarries,
            nodes,
            power,
            data,
            deposits,
            research,
            orbit,
            pods,
            order,
            changed,
            ..
        } = self;
        orbit.now = tick;
        let unlocked = research.machine_recipes_unlocked();
        process::sense_sources(processors, world, orbit, tick);
        power.balance(generators, miners, processors, labs, pipework, quarries, research, &unlocked, tick);
        data.balance((nodes, processors), power, &unlocked, research, (orbit, tick));
        let mut sinks = Sinks { storages, processors, routers, generators, labs, unlocked: &unlocked };
        let (mining, machines) = (research.rate_permille(Bonus::Mining), research.rate_permille(Bonus::Machines));
        let tuned = process::optimizer_bonuses(sinks.processors, power, data);
        let swarms = process::hub_bonuses(sinks.processors, power, data);
        for (m, &p) in miners.iter_mut().zip(&power.miner_pole) {
            (m.speed, m.boost) =
                (power.speed(p), mining * process::optimizer_bonus_at(&tuned, m.pos) / power::FULL_SPEED);
            m.step(deposits, world, tick, belts, &mut sinks, events);
        }
        for (q, &p) in quarries.iter_mut().zip(&power.quarry_pole) {
            q.speed = power.speed(p);
            q.step(world, belts, &mut sinks, changed, events);
        }
        for s in sinks.storages.iter_mut() {
            s.step(belts);
        }
        for (i, (m, &p)) in sinks.processors.iter_mut().zip(&power.process_pole).enumerate() {
            let mut share = if m.draws_power() { power.speed(p) } else { power::FULL_SPEED };
            if m.spec.compute < 0 {
                share = share * data.satisfaction(data.process_node[i]) / power::FULL_SPEED;
                // a data consumer
            }
            m.boost = machines * process::optimizer_bonus_at(&tuned, m.pos) / power::FULL_SPEED;
            let pos = m.pos;
            if let Some(port) = m.port_mut() {
                port.bonus = process::hub_bonus_at(&swarms, pos);
                port.linked = orbit.covering(COMMS, pos.x, pos.z, tick) > 0;
            }
            m.pump(deposits, world, tick, share);
            m.step(belts, share, &unlocked);
            m.report_made(events);
            if let Some(item) = m.take_lifted() {
                orbit.launch(item, m.pos, tick);
                events.push(SimEvent::Launched { pos: m.pos, item });
            }
        }
        process::run_pods(sinks.processors, pods, orbit, tick, events);
        for r in sinks.routers.iter_mut() {
            r.step(belts);
        }
        step_labs(sinks.labs, sinks.processors, (&power.lab_pole, &power.process_pole), power, data, research);
        let used = sinks.labs.iter_mut().flat_map(|l| l.used.drain(..));
        let burned = sinks.generators.iter_mut().flat_map(|g| g.used.drain(..));
        events.extend(used.chain(burned).map(|s| SimEvent::Consumed { item: s.item, count: s.count }));
        pumping::step_pipework(pipework, &power.pipe_pole, power, world, changed);
        process::draw_water(sinks.processors, pipework);
        belt_step(belts, &mut sinks, order, TICK);
    }
}
