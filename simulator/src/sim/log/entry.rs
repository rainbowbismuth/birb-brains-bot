use crate::dto::patch::Equipment;
use crate::sim::{Action, ActionKind, Combatant, CombatantId, Condition, Location, MAX_COMBATANTS, Phase};

#[derive(Clone)]
pub struct Entry<'a> {
    pub clock_tick: usize,
    pub phase: Phase,
    pub combatants: Vec<Combatant<'a>>,
    pub event: Event<'a>
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
    Moved(CombatantId, Location, Location),
}

#[derive(Copy, Clone)]
pub enum Source<'a> {
    Phase,
    Condition(Condition),
    Weapon(CombatantId, Option<&'a Equipment>),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EvasionType {
    Guarded,
    Blocked,
    Parried,
    Evaded,
}