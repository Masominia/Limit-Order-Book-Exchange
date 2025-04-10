//! One book, a population of agents, a fundamental value and a tape of trades.

use crate::agents::maker::Maker;
use crate::agents::momentum::Momentum;
use crate::agents::noise::Noise;
use crate::agents::value::Value;
use crate::agents::{Action, Agent, View};
use crate::book::Book;
use crate::math::rng::Rng;
use crate::sim::params::Params;
use crate::types::{OrderId, Qty, Trade};

/// Owner number of orders sent from outside the population (execution schedules, tests).
pub const EXTERNAL: u32 = u32::MAX;

/// State of the market at the end of one step.
#[derive(Clone, Copy, Debug)]
pub struct Record {
    pub mid: f64,
    /// Ask minus bid in ticks, NaN when one side is empty.
    pub spread: f64,
    /// Volume within five ticks of the best price, both sides added.
    pub depth: f64,
    pub volume: Qty,
    pub signed_volume: i64,
}

pub struct Market {
    pub book: Book,
    pub rng: Rng,
    pub time: u64,
    pub fundamental: f64,
    pub reference: f64,
    pub mids: Vec<f64>,
    pub tape: Vec<Trade>,
    pub records: Vec<Record>,
    fundamental_vol: f64,
    agents: Vec<Box<dyn Agent>>,
    scratch: Vec<Action>,
    recording: bool,
}

impl Market {
    pub fn empty(start: f64, fundamental_vol: f64, seed: u64) -> Self {
        Market {
            book: Book::new(),
            rng: Rng::new(seed),
            time: 0,
            fundamental: start,
            reference: start,
            mids: Vec::new(),
            tape: Vec::new(),
            records: Vec::new(),
            fundamental_vol,
            agents: Vec::new(),
            scratch: Vec::new(),
            recording: true,
        }
    }

    /// The standard population described by `params`, already warmed up.
    pub fn standard(params: &Params, seed: u64) -> Self {
        let mut m = Market::empty(params.start, params.fundamental_vol, seed);
        m.add(Box::new(Noise::new(params.limit_rate, params.market_rate, params.cancel_rate, params.depth_scale, params.max_qty)));
        for _ in 0..params.makers {
            m.add(Box::new(Maker::new(params.maker_half_spread, params.maker_levels, params.maker_qty, params.maker_skew, params.maker_refresh)));
        }
        for _ in 0..params.momentum {
            m.add(Box::new(Momentum::new(params.momentum_window, params.momentum_threshold, params.momentum_rate, params.max_qty)));
        }
        for _ in 0..params.value {
            m.add(Box::new(Value::new(params.value_band, params.value_rate, params.max_qty)));
        }
        m.recording = false;
        m.run(params.warmup);
        m.recording = true;
        m.tape.clear();
        m
    }

    /// Adds an agent and returns its owner number.
    pub fn add(&mut self, agent: Box<dyn Agent>) -> u32 {
        self.agents.push(agent);
        (self.agents.len() - 1) as u32
    }

    pub fn agents(&self) -> &[Box<dyn Agent>] {
        &self.agents
    }

    pub fn run(&mut self, steps: u64) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// One step: the fundamental value moves, then every agent acts in turn on the current book.
    pub fn step(&mut self) {
        self.time += 1;
        self.fundamental += self.fundamental_vol * self.rng.normal();
        let tape_start = self.tape.len();
        let mut agents = std::mem::take(&mut self.agents);
        let mut actions = std::mem::take(&mut self.scratch);
        for i in 0..agents.len() {
            actions.clear();
            let view = View { book: &self.book, time: self.time, reference: self.reference, fundamental: self.fundamental, mids: &self.mids };
            agents[i].act(&view, &mut self.rng, &mut actions);
            for &action in &actions {
                self.apply(i as u32, action, &mut agents);
            }
        }
        self.agents = agents;
        self.scratch = actions;
        if let Some(mid) = self.book.mid() {
            self.reference = mid;
        }
        self.mids.push(self.reference);
        if self.recording {
            let fresh = &self.tape[tape_start..];
            self.records.push(Record {
                mid: self.reference,
                spread: self.book.spread().map_or(f64::NAN, |s| s as f64),
                depth: (self.book.volume_near(crate::Side::Buy, 5) + self.book.volume_near(crate::Side::Sell, 5)) as f64,
                volume: fresh.iter().map(|t| t.qty).sum(),
                signed_volume: fresh.iter().map(|t| t.taker_side.sign() * t.qty as i64).sum(),
            });
        } else {
            self.tape.clear();
        }
    }

    fn apply(&mut self, owner: u32, action: Action, agents: &mut [Box<dyn Agent>]) -> Option<OrderId> {
        let start = self.tape.len();
        let placed = match action {
            Action::Limit { side, price, qty } => {
                let (id, rest) = self.book.limit(owner, side, price, qty, &mut self.tape);
                (rest > 0).then_some(id)
            }
            Action::Market { side, qty } => {
                self.book.market(owner, side, qty, &mut self.tape);
                None
            }
            Action::Cancel(id) => {
                self.book.cancel(id);
                None
            }
        };
        if let (Some(id), Some(agent)) = (placed, agents.get_mut(owner as usize)) {
            agent.placed(id);
        }
        for t in &self.tape[start..] {
            if let Some(agent) = agents.get_mut(t.maker_owner as usize) {
                agent.filled(t.taker_side.opposite(), t.price, t.qty);
            }
            if let Some(agent) = agents.get_mut(t.taker_owner as usize) {
                agent.filled(t.taker_side, t.price, t.qty);
            }
        }
        placed
    }

    /// Sends an order from outside the population. Returns the resting order id, if any, and the fills.
    pub fn submit(&mut self, action: Action) -> (Option<OrderId>, Vec<Trade>) {
        let start = self.tape.len();
        let mut agents = std::mem::take(&mut self.agents);
        let placed = self.apply(EXTERNAL, action, &mut agents);
        self.agents = agents;
        (placed, self.tape[start..].to_vec())
    }
}
