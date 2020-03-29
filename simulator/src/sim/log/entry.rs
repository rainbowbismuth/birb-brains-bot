use colored::Colorize;

use crate::dto::rust::Equipment;
use crate::sim::{Combatant, CombatantId, Condition, Location, Phase, Team, MAX_COMBATANTS};

#[derive(Clone, Debug)]
pub struct Entry<'a> {
    pub clock_tick: usize,
    pub phase: Phase,
    pub combatants: [Combatant<'a>; MAX_COMBATANTS],
    pub event: Event<'a>,
}

#[derive(Clone, Debug)]
pub enum Event<'a> {
    DidNothing(CombatantId),
    HpDamage(CombatantId, i16, Source<'a>),
    HpHeal(CombatantId, i16, Source<'a>),
    MpDamage(CombatantId, i16, Source<'a>),
    MpHeal(CombatantId, i16, Source<'a>),
    AddedCondition(CombatantId, Condition, Source<'a>),
    LostCondition(CombatantId, Condition, Source<'a>),
    Died(CombatantId, Source<'a>),
    BecameCrystal(CombatantId),
    Evaded(CombatantId, EvasionType, Source<'a>),
    Moved(CombatantId, Location, Location),
}

#[derive(Copy, Clone, Debug)]
pub enum Source<'a> {
    Phase,
    Constant(&'static str),
    Condition(Condition),
    Weapon(CombatantId, Option<&'a Equipment>),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EvasionType {
    Guarded,
    Blocked,
    Parried,
    Evaded,
    BladeGrasp,
}

pub fn describe_entry(entry: &Entry) -> String {
    format!(
        "CT {}: {}: {}",
        entry.clock_tick,
        describe_phase(&entry.phase, &entry.combatants),
        describe_event(&entry.event, &entry.combatants)
    )
}

pub fn describe_phase(phase: &Phase, combatants: &[Combatant]) -> String {
    match phase {
        Phase::StatusCheck => "Status Check".cyan().to_string(),

        Phase::SlowActionCharging => "Slow Action Charging".cyan().to_string(),

        Phase::SlowAction(cid) => format!("{}'s SA", combatants[cid.index()].name())
            .cyan()
            .to_string(),

        Phase::CtCharging => "CT Charging".cyan().to_string(),

        Phase::ActiveTurn(cid) => format!("{}'s AT", combatants[cid.index()].name())
            .cyan()
            .to_string(),
    }
}

pub fn describe_event(event: &Event, combatants: &[Combatant]) -> String {
    match event {
        Event::DidNothing(target_id) => format!(
            "{} did nothing!",
            describe_combatant(*target_id, combatants)
        ),

        Event::HpDamage(target_id, amount, src) => format!(
            "{} took {} damage from {}",
            describe_combatant(*target_id, combatants),
            amount,
            describe_source(*src, combatants)
        ),

        Event::HpHeal(target_id, amount, src) => format!(
            "{} was healed for {} hp from {}",
            describe_combatant(*target_id, combatants),
            amount,
            describe_source(*src, combatants)
        ),

        Event::MpDamage(target_id, amount, src) => format!(
            "{} lost {} MP from {}",
            describe_combatant(*target_id, combatants),
            amount,
            describe_source(*src, combatants)
        ),

        Event::MpHeal(target_id, amount, src) => format!(
            "{} gained {} MP from {}",
            describe_combatant(*target_id, combatants),
            amount,
            describe_source(*src, combatants)
        ),

        Event::AddedCondition(target_id, cond, src) => format!(
            "{} now has {} because of {}",
            describe_combatant(*target_id, combatants),
            cond.name(),
            describe_source(*src, combatants)
        ),

        Event::LostCondition(target_id, cond, src) => format!(
            "{} no longer has {} because of {}",
            describe_combatant(*target_id, combatants),
            cond.name(),
            describe_source(*src, combatants)
        ),

        Event::Died(target_id, src) => format!(
            "{} died from {}",
            describe_combatant(*target_id, combatants),
            describe_source(*src, combatants)
        ),

        Event::BecameCrystal(target_id) => format!(
            "{} is now a crystal",
            describe_combatant(*target_id, combatants)
        ),

        Event::Evaded(target_id, EvasionType::Guarded, src) => format!(
            "{} guarded {}",
            describe_combatant(*target_id, combatants),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::Blocked, src) => format!(
            "{} blocked {}",
            describe_combatant(*target_id, combatants),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::Parried, src) => format!(
            "{} parried {}",
            describe_combatant(*target_id, combatants),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::Evaded, src) => format!(
            "{} evaded {}",
            describe_combatant(*target_id, combatants),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::BladeGrasp, src) => format!(
            "{} blade grasped {}",
            describe_combatant(*target_id, combatants),
            describe_source(*src, combatants)
        ),

        Event::Moved(target_id, old_location, new_location) => format!(
            "{} moved from {} to {}",
            describe_combatant(*target_id, combatants),
            old_location.x,
            new_location.x
        ),
    }
}

pub fn describe_combatant(c_id: CombatantId, combatants: &[Combatant]) -> String {
    let combatant = &combatants[c_id.index()];
    let conditions = combatant.all_conditions();
    let cond_str = if conditions.is_empty() {
        "".to_owned()
    } else {
        format!(
            ", {}",
            conditions
                .iter()
                .map(|c| c.name())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    match combatant.team() {
        Team::Left => format!(
            "{} ({} HP, loc: {}{})",
            combatant.name().red(),
            combatant.hp(),
            combatant.location.x,
            cond_str
        ),

        Team::Right => format!(
            "{} ({} HP, loc: {}{})",
            combatant.name().blue(),
            combatant.hp(),
            combatant.location.x,
            cond_str
        ),
    }
}

pub fn describe_source(src: Source, combatants: &[Combatant]) -> String {
    match src {
        Source::Phase => String::from("the current phase"),
        Source::Constant(str) => str.to_owned(),
        Source::Condition(cond) => String::from(cond.name()),
        Source::Weapon(c_id, Some(weapon)) => format!(
            "{}\'s {}",
            describe_combatant(c_id, combatants),
            weapon.name
        ),
        Source::Weapon(c_id, None) => {
            format!("{}\'s bare hands", describe_combatant(c_id, combatants))
        }
    }
}
