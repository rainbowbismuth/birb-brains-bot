use colored::Colorize;

use crate::dto::rust::Equipment;
use crate::sim::{
    combatant_height, tile_height, Action, ActionTarget, Arena, CalcAlgorithm, CalcAttribute,
    Combatant, CombatantId, Condition, Facing, Panel, Phase, RelativeFacing, Team, MAX_COMBATANTS,
};

#[derive(Clone)]
pub struct Entry<'a> {
    pub clock_tick: usize,
    pub phase: Phase,
    pub combatants: [Combatant<'a>; MAX_COMBATANTS],
    pub event: Event<'a>,
}

#[derive(Clone)]
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
    Moved(CombatantId, Panel, Panel),
    UsingAbility(CombatantId, Action<'a>),
    AbilityMissed(CombatantId, CombatantId),
    StartedCharging(CombatantId, Action<'a>),
    Silenced(CombatantId, Action<'a>),
    NoMP(CombatantId, Action<'a>),
    Broke(CombatantId, &'a Equipment),
    PhysicalAttackBuff(CombatantId, i8, Source<'a>),
    MagicalAttackBuff(CombatantId, i8, Source<'a>),
    SpeedBuff(CombatantId, i8, Source<'a>),
    Knockback(CombatantId, Panel),
    CriticalQuick(CombatantId),
    SpellReflected(CombatantId, Panel),
    BraveBuff(CombatantId, i8, Source<'a>),
}

#[derive(Copy, Clone)]
pub enum Source<'a> {
    Phase,
    Ability,
    Constant(&'static str),
    Condition(Condition),
    Weapon(CombatantId, Option<&'a Equipment>),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EvasionType {
    Guarded,
    Blocked,
    Parried,
    Evaded,
    BladeGrasp,
    ArrowGuard,
}

pub fn describe_entry(entry: &Entry, arena: &Arena) -> String {
    format!(
        "CT {}: {}: {}",
        entry.clock_tick,
        describe_phase(&entry.phase, &entry.combatants),
        describe_event(&entry.event, &entry.combatants, arena),
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

pub fn describe_location(panel: Panel, arena: &Arena) -> String {
    let location = panel.location();
    if location.x < 0
        || location.y < 0
        || (location.x >= arena.width as i16)
        || (location.y >= arena.height as i16)
    {
        return format!("({},{})", location.x, location.y);
    }
    let tile = arena.tile(panel);
    format!("({},{},{}h)", location.x, location.y, tile_height(&tile))
}

pub fn describe_location_combatant(combatant: &Combatant, arena: &Arena) -> String {
    let panel = combatant.panel;
    let location = panel.location();
    if location.x < 0
        || location.y < 0
        || (location.x >= arena.width as i16)
        || (location.y >= arena.height as i16)
    {
        return format!("({},{})", location.x, location.y);
    }
    let tile = arena.tile(panel);
    let upper_lower = if panel.layer() { "U" } else { "L" };
    format!(
        "({},{},{}h,{})",
        panel.x(),
        panel.y(),
        combatant_height(&tile, combatant),
        upper_lower
    )
}

pub fn describe_event(event: &Event, combatants: &[Combatant], arena: &Arena) -> String {
    match event {
        Event::DidNothing(target_id) => format!(
            "{} did nothing!",
            describe_combatant(*target_id, combatants, arena)
        ),

        Event::HpDamage(target_id, amount, src) => format!(
            "{} took {} damage from {}",
            describe_combatant(*target_id, combatants, arena),
            amount,
            describe_source(*src, combatants)
        ),

        Event::HpHeal(target_id, amount, src) => format!(
            "{} was healed for {} HP from {}",
            describe_combatant(*target_id, combatants, arena),
            amount.abs(),
            describe_source(*src, combatants)
        ),

        Event::MpDamage(target_id, amount, src) => format!(
            "{} lost {} MP from {}",
            describe_combatant(*target_id, combatants, arena),
            amount,
            describe_source(*src, combatants)
        ),

        Event::MpHeal(target_id, amount, src) => format!(
            "{} gained {} MP from {}",
            describe_combatant(*target_id, combatants, arena),
            amount.abs(),
            describe_source(*src, combatants)
        ),

        Event::AddedCondition(target_id, cond, src) => format!(
            "{} now has {} because of {}",
            describe_combatant(*target_id, combatants, arena),
            cond.name(),
            describe_source(*src, combatants)
        ),

        Event::LostCondition(target_id, cond, src) => format!(
            "{} no longer has {} because of {}",
            describe_combatant(*target_id, combatants, arena),
            cond.name(),
            describe_source(*src, combatants)
        ),

        Event::Died(target_id, src) => format!(
            "{} died from {}",
            describe_combatant(*target_id, combatants, arena),
            describe_source(*src, combatants)
        ),

        Event::BecameCrystal(target_id) => format!(
            "{} is now a crystal",
            describe_combatant(*target_id, combatants, arena)
        ),

        Event::Evaded(target_id, EvasionType::Guarded, src) => format!(
            "{} guarded {}",
            describe_combatant(*target_id, combatants, arena),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::Blocked, src) => format!(
            "{} blocked {}",
            describe_combatant(*target_id, combatants, arena),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::Parried, src) => format!(
            "{} parried {}",
            describe_combatant(*target_id, combatants, arena),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::Evaded, src) => format!(
            "{} evaded {}",
            describe_combatant(*target_id, combatants, arena),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::BladeGrasp, src) => format!(
            "{} blade grasped {}",
            describe_combatant(*target_id, combatants, arena),
            describe_source(*src, combatants)
        ),

        Event::Evaded(target_id, EvasionType::ArrowGuard, src) => format!(
            "{} arrow guarded {}",
            describe_combatant(*target_id, combatants, arena),
            describe_source(*src, combatants)
        ),

        Event::Moved(target_id, old_location, new_location) => format!(
            "{} moved from {} to {}",
            describe_combatant(*target_id, combatants, arena),
            describe_location(*old_location, arena),
            describe_location(*new_location, arena),
        ),

        Event::UsingAbility(target_id, action) => {
            let combatant_desc = describe_combatant(*target_id, combatants, arena);
            let ability_name = action.ability.name;
            let target_desc = describe_target_short(action.target, combatants, arena);
            match describe_relative_facing(*target_id, action.target, combatants) {
                Some(relative_facing) => format!(
                    "{} is using {} on {} from the {}",
                    combatant_desc, ability_name, target_desc, relative_facing
                ),
                None => format!(
                    "{} is using {} on {}",
                    combatant_desc, ability_name, target_desc
                ),
            }
        }

        Event::AbilityMissed(user_id, target_id) => format!(
            "{}'s ability missed {}!",
            describe_combatant_short(*user_id, combatants),
            describe_combatant_short(*target_id, combatants),
        ),

        Event::StartedCharging(target_id, action) => format!(
            "{} started charging {} on {}",
            describe_combatant(*target_id, combatants, arena),
            action.ability.name,
            describe_target_short(action.target, combatants, arena),
        ),

        Event::Silenced(target_id, action) => format!(
            "{} couldn't finish charging {} because they were silenced",
            describe_combatant(*target_id, combatants, arena),
            action.ability.name
        ),

        Event::NoMP(target_id, action) => format!(
            "{} couldn't finish {} due to lack of MP",
            describe_combatant(*target_id, combatants, arena),
            action.ability.name
        ),

        Event::Broke(target_id, equip) => format!(
            "{}\'s {} was broken",
            describe_combatant(*target_id, combatants, arena),
            equip.name
        ),

        Event::PhysicalAttackBuff(target_id, amount, src) => format!(
            "{}\'s physical attack increased by {} because of {}",
            describe_combatant(*target_id, combatants, arena),
            amount,
            describe_source(*src, combatants)
        ),

        Event::MagicalAttackBuff(target_id, amount, src) => format!(
            "{}\'s magical attack increased by {} because of {}",
            describe_combatant(*target_id, combatants, arena),
            amount,
            describe_source(*src, combatants)
        ),

        Event::SpeedBuff(target_id, amount, src) => format!(
            "{}\'s speed increased by {} because of {}",
            describe_combatant(*target_id, combatants, arena),
            amount,
            describe_source(*src, combatants)
        ),

        Event::Knockback(target_id, new_location) => format!(
            "{} was knocked back into {}",
            describe_combatant_short(*target_id, combatants),
            describe_location(*new_location, arena),
        ),

        Event::CriticalQuick(target_id) => format!(
            "{} had critical quick triggered!",
            describe_combatant_short(*target_id, combatants)
        ),

        Event::SpellReflected(target_id, new_location) => format!(
            "A spell was reflected off of {} onto {}",
            describe_combatant_short(*target_id, combatants),
            describe_location(*new_location, arena),
        ),

        Event::BraveBuff(target_id, amount, src) => format!(
            "{}\'s brave increased by {} because of {}",
            describe_combatant(*target_id, combatants, arena),
            amount,
            describe_source(*src, combatants)
        ),
    }
}

pub fn describe_target_short(
    target: ActionTarget,
    combatants: &[Combatant],
    arena: &Arena,
) -> String {
    match target {
        ActionTarget::Id(target_id) => describe_combatant_short(target_id, combatants),
        ActionTarget::Panel(location) => describe_location(location, arena),
        ActionTarget::Math(attr, algo) => describe_math(attr, algo),
    }
}

pub fn describe_math(attr: CalcAttribute, algo: CalcAlgorithm) -> String {
    let attr_name = match attr {
        CalcAttribute::CT => "CT",
        CalcAttribute::Height => "Height",
    };
    match algo {
        CalcAlgorithm::Prime => format!("everyone with a prime {}", attr_name),
        CalcAlgorithm::M5 => format!("everyone with a {} divisible by 5", attr_name),
        CalcAlgorithm::M4 => format!("everyone with a {} divisible by 4", attr_name),
        CalcAlgorithm::M3 => format!("everyone with a {} divisible by 3", attr_name),
    }
}

pub fn describe_combatant_short(c_id: CombatantId, combatants: &[Combatant]) -> String {
    let combatant = &combatants[c_id.index()];
    match combatant.team() {
        Team::Left => combatant.name().red().to_string(),
        Team::Right => combatant.name().blue().to_string(),
    }
}

pub fn describe_combatant(c_id: CombatantId, combatants: &[Combatant], arena: &Arena) -> String {
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
            "{} [{} HP, {} MP, {}{}{}]",
            combatant.name().red(),
            combatant.hp(),
            combatant.mp(),
            describe_location_combatant(combatant, arena),
            describe_facing(combatant.facing),
            cond_str
        ),

        Team::Right => format!(
            "{} [{} HP, {} MP, {}{}{}]",
            combatant.name().blue(),
            combatant.hp(),
            combatant.mp(),
            describe_location_combatant(combatant, arena),
            describe_facing(combatant.facing),
            cond_str
        ),
    }
}

pub fn describe_facing(facing: Facing) -> &'static str {
    match facing {
        Facing::North => "N",
        Facing::East => "E",
        Facing::South => "S",
        Facing::West => "W",
    }
}

pub fn describe_relative_facing(
    user_id: CombatantId,
    target: ActionTarget,
    combatants: &[Combatant],
) -> Option<&'static str> {
    if let Some(target_id) = target.to_target_id_only() {
        let user = combatants[user_id.index()];
        let target = combatants[target_id.index()];
        Some(match user.relative_facing(&target) {
            RelativeFacing::Front => "front",
            RelativeFacing::Side => "side",
            RelativeFacing::Back => "back",
        })
    } else {
        None
    }
}

pub fn describe_source(src: Source, combatants: &[Combatant]) -> String {
    match src {
        Source::Phase => String::from("the current phase"),
        Source::Ability => String::from("the used ability"),
        Source::Constant(str) => str.to_owned(),
        Source::Condition(cond) => String::from(cond.name()),
        Source::Weapon(c_id, Some(weapon)) => format!(
            "{}\'s {}",
            describe_combatant_short(c_id, combatants),
            weapon.name
        ),
        Source::Weapon(c_id, None) => format!(
            "{}\'s bare hands",
            describe_combatant_short(c_id, combatants)
        ),
    }
}
