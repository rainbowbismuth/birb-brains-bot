use std::cell::RefCell;

use crate::sim::{ALL_CONDITIONS, Combatant, CombatantId};

#[derive(Clone)]
struct LogData {
    pub clock_tick: usize,
    pub log: Option<Vec<String>>,
    pub prepend: String,
}

#[derive(Clone)]
pub struct Log {
    interior: RefCell<LogData>
}

impl Log {
    pub fn new() -> Log {
        Log { interior: RefCell::new(LogData::new()) }
    }

    pub fn new_no_log() -> Log {
        Log { interior: RefCell::new(LogData::new_no_log()) }
    }

    pub fn set_clock_tick(&self, clock_tick: usize) {
        self.interior.borrow_mut().set_clock_tick(clock_tick);
    }

    pub fn phase(&self, phase_name: &'static str) {
        self.interior.borrow_mut().phase(phase_name);
    }

    pub fn report<F>(&self, f: F)
        where F: Fn() -> String {
        self.interior.borrow_mut().report(f);
    }

    pub fn unit_report<F>(&self, combatant: &Combatant, f: F)
        where F: Fn() -> String {
        self.interior.borrow_mut().unit_report(combatant, f)
    }

    pub fn active_turn_bar(&self, combatant: &Combatant) {
        self.interior.borrow_mut().active_turn_bar(combatant)
    }
}

impl LogData {
    fn new() -> LogData {
        LogData {
            clock_tick: 0,
            log: Some(vec![]),
            prepend: String::from(""),
        }
    }

    fn new_no_log() -> LogData {
        LogData {
            clock_tick: 0,
            log: None,
            prepend: String::from(""),
        }
    }

    fn set_clock_tick(&mut self, clock_tick: usize) {
        self.clock_tick = clock_tick;
    }

    fn prepend_info(&self) -> String {
        format!("CT {}: {}", self.clock_tick, self.prepend)
    }

    fn phase(&mut self, phase_name: &'static str) {
        if self.log.is_some() {
            self.prepend = String::from(phase_name)
        }
    }

    fn add(&mut self, s: String) {
        match self.log.as_mut() {
            Some(log) => log.push(s),
            None => {}
        }
    }

    fn report<F>(&mut self, f: F)
        where F: Fn() -> String {
        if self.log.is_some() {
            let prepend = self.prepend_info();
            self.add(format!("{}: {}", prepend, f()));
        }
    }

    fn unit_report<F>(&mut self, combatant: &Combatant, f: F)
        where F: Fn() -> String {
        if self.log.is_some() {
            let prepend = self.prepend_info();
            self.add(format!("{}: {} ({} HP) {}", prepend, combatant.name, combatant.hp(), f()));
        }
    }

    fn active_turn_bar(&mut self, combatant: &Combatant) {
        if self.log.is_some() {
            let mut all_conditions = vec![];
            for condition in ALL_CONDITIONS.iter() {
                if combatant.has_condition(*condition) {
                    all_conditions.push(condition.name());
                }
            }
            // TODO: Add location
            if !all_conditions.is_empty() {
                self.prepend = format!("{} ({} HP, {})", combatant.name, combatant.hp(), all_conditions.join(", "));
            } else {
                self.prepend = format!("{} ({} HP)", combatant.name, combatant.hp());
            }
        }
    }
}
