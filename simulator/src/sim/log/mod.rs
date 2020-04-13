use std::cell::RefCell;

pub use entry::*;

use crate::sim::{Combatant, Phase, MAX_COMBATANTS};

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

pub fn unit_card(user: &Combatant) -> String {
    let l1 = format!(
        "         HP {:>10}           |  {:>1}. {:<10}",
        user.max_hp(),
        user.id().id,
        user.info.name
    );
    let l2 = format!(
        "         MP {:>10}           |     {:<10}",
        user.max_mp(),
        user.info.job
    );
    let l4 = format!(
        "          {:>10}   {:>1}         | {:<2}  Brave {:<02}  Faith {:<02}",
        "",
        "",
        user.info.sign.to_emoji(),
        user.raw_brave,
        user.raw_faith
    );
    let l5 = format!(
        " Move ... {:<3}     Weap.Power          AT  C-EV  S-EV  A-EV",
        user.movement()
    );
    let line_break = "-".repeat(l5.len());
    let l6 = format!(
        " Jump ... {:<3}    R ... {:>03} / {:>02}%   🔪 {:>02} / {:>02}% / {:>02}% / {:>02}%",
        user.jump(),
        user.main_hand().map_or(0, |w| w.wp),
        user.main_hand().map_or(0, |w| w.w_ev),
        user.pa(),
        (user.class_evasion(false) * 100.0) as i16,
        (user.physical_shield_evasion(false) * 100.0) as i16,
        (user.physical_accessory_evasion(false) * 100.0) as i16
    );
    let l7 = format!(
        "Speed ... {:<02}     L ... {:>03} / {:>02}%   🔮 {:>02} / {:>02}% / {:>02}% / {:>02}%",
        user.speed(),
        user.off_hand().map_or(0, |w| w.wp),
        user.off_hand().map_or(0, |w| w.w_ev),
        user.ma(),
        0,
        (user.magical_shield_evasion() * 100.0) as i16,
        (user.magical_accessory_evasion() * 100.0) as i16
    );
    let l8 = format!("    EQP. {:<23} |    Actions", "");
    let l9 = format!(
        "🤚       {:<23} |     {:<14}",
        user.main_hand().map_or("", |eq| &eq.name),
        user.info.abilities.get(0).map_or("", |a| a.name)
    );
    let l10 = format!(
        "✋       {:<23} |     {:<14}",
        user.off_hand().map_or("", |eq| &eq.name),
        user.info.abilities.get(1).map_or("", |a| a.name)
    );
    let l11 = format!(
        "🧢       {:<23} |     {:<14}",
        user.headgear().map_or("", |eq| &eq.name),
        user.info.abilities.get(2).map_or("", |a| a.name)
    );
    let l12 = format!(
        "👚       {:<23} |     {:<14}",
        user.armor().map_or("", |eq| &eq.name),
        user.info.abilities.get(3).map_or("", |a| a.name)
    );
    let l13 = format!(
        "📿       {:<23} |     {:<14}",
        user.accessory().map_or("", |eq| &eq.name),
        user.info.abilities.get(4).map_or("", |a| a.name)
    );

    let mut extra = vec![];
    if user.info.abilities.len() > 5 {
        for ability in &user.info.abilities[5..] {
            extra.push(format!("         {:<23} |     {:<14}", "", ability.name));
        }
    }
    let mut out = vec![
        line_break.clone(),
        l1,
        l2,
        l4,
        line_break.clone(),
        l5,
        l6,
        l7,
        line_break.clone(),
        l8,
        l9,
        l10,
        l11,
        l12,
        l13,
    ];
    out.append(&mut extra);
    out.push(line_break.clone());
    out.join("\n")
}
