use std::cell::RefCell;

pub use entry::*;

use crate::sim::{Combatant, CombatantId, Phase, Simulation, ALL_CONDITIONS, MAX_COMBATANTS};

pub mod entry;

#[derive(Clone)]
struct LogData<'a> {
    clock_tick: usize,
    phase: Phase,
    log: Option<Vec<Entry<'a>>>,
}

#[derive(Clone)]
pub struct Log<'a> {
    interior: RefCell<LogData<'a>>,
}

impl<'a> Log<'a> {
    pub fn new() -> Log<'a> {
        Log {
            interior: RefCell::new(LogData::new()),
        }
    }

    pub fn new_no_log() -> Log<'a> {
        Log {
            interior: RefCell::new(LogData::new_no_log()),
        }
    }

    pub fn set_clock_tick(&self, clock_tick: usize) {
        self.interior.borrow_mut().set_clock_tick(clock_tick);
    }

    pub fn set_phase(&self, phase: Phase) {
        self.interior.borrow_mut().set_phase(phase);
    }

    pub fn add(&self, combatants: &[Combatant<'a>; MAX_COMBATANTS], event: Event<'a>) {
        self.interior.borrow_mut().add(combatants, event);
    }

    pub fn entries(&self) -> Vec<Entry<'a>> {
        Vec::from(self.interior.borrow().entries())
    }
}

impl<'a> LogData<'a> {
    fn new() -> LogData<'a> {
        LogData {
            clock_tick: 0,
            phase: Phase::StatusCheck,
            log: Some(vec![]),
        }
    }

    fn new_no_log() -> LogData<'a> {
        LogData {
            clock_tick: 0,
            phase: Phase::StatusCheck,
            log: None,
        }
    }

    fn set_clock_tick(&mut self, clock_tick: usize) {
        self.clock_tick = clock_tick;
    }

    fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
    }

    fn add(&mut self, combatants: &[Combatant<'a>; 8], event: Event<'a>) {
        if let Some(log) = &mut self.log {
            let entry = Entry {
                clock_tick: self.clock_tick,
                phase: self.phase,
                combatants: combatants.clone(),
                event,
            };
            // println!("{}", describe_entry(&entry)); // For intermixing with debugging, should flag.
            log.push(entry);
        }
    }

    fn entries(&self) -> &[Entry<'a>] {
        self.log.as_ref().unwrap().as_slice()
    }
}
