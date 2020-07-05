use crate::sim::actions::common::{mod_5_formula, AddConditionSpellImpl};
use crate::sim::actions::monster::{BadBreathImpl, UlmaguestImpl};
use crate::sim::actions::punch_art::Pummel;
use crate::sim::actions::talk_skill::ConditionTalkSkillImpl;
use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::attack::AttackImpl;
use crate::sim::common::{do_hp_heal, mod_2_formula_xa, mod_5_formula_xa};
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, CAN_BE_REFLECTED,
    CASTER_IMMUNE, SILENCEABLE, TARGET_NOT_SELF, TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const BYBLOS_ABILITIES: &[Ability] = &[
    // Bio Tenebris: 4 range, 1 AoE, 3 CT, 8 MP. Element: Wind. Effect: Damage Faith(MA * 12); Chance to Add Darkness.
    Ability {
        name: "Bio Tenebris",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 8,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &ByblosElemental {
            element: Element::Wind,
            q: 12,
            range: 4,
            ctr: Some(3),
            condition: Some(Condition::Darkness),
        },
    },
    // Bio Venenum: 4 range, 1 AoE, 3 CT, 8 MP. Element: Water. Effect: Damage Faith(MA * 12); Chance to Add Poison.
    Ability {
        name: "Bio Venenum",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 8,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &ByblosElemental {
            element: Element::Water,
            q: 12,
            range: 4,
            ctr: Some(3),
            condition: Some(Condition::Poison),
        },
    },
    // Bio Oleum: 4 range, 1 AoE, 3 CT, 8 MP. Element: Earth. Effect: Damage Faith(MA * 12); Chance to Add Oil.
    Ability {
        name: "Bio Oleum",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 8,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &ByblosElemental {
            element: Element::Earth,
            q: 12,
            range: 4,
            ctr: Some(3),
            condition: Some(Condition::Oil),
        },
    },
    // Bio Ranae: 4 range, 1 AoE, 5 CT, 16 MP. Hit: Faith(MA + 110)% Effect: Add Frog.
    Ability {
        name: "Bio Ranae",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 16,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Frog],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 110,
            ctr: 5,
            range: 4,
        },
    },
    // Bio Sanctus: 4 range, 1 AoE, 5 CT, 16 MP. Hit: Faith(MA + 110)% Effect: Add Slow.
    Ability {
        name: "Bio Sanctus",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 16,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Slow],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 110,
            ctr: 5,
            range: 4,
        },
    },
    // Bio Silentium: 4 range, 1 AoE, 5 CT, 16 MP. Hit: Faith(MA + 120)% Effect: Add Silence.
    Ability {
        name: "Bio Silentium",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 12,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Silence],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 120,
            ctr: 5,
            range: 4,
        },
    },
    // Bio Lapis: 4 range, 1 AoE, 5 CT, 16 MP. Hit: Faith(MA + 110)% Effect: Add Petrify.
    Ability {
        name: "Bio Lapis",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 12,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Petrify],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 110,
            ctr: 5,
            range: 4,
        },
    },
    // Bio Immortuos: 4 range, 2 AoE, 6 CT, 24 MP. Effect: Damage Faith(MA * 24); Chance to Add Undead.
    Ability {
        name: "Bio Immortuos",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 24,
        aoe: AoE::Diamond(2, Some(1)),
        implementation: &ByblosElemental {
            element: Element::None,
            q: 24,
            range: 4,
            ctr: Some(6),
            condition: Some(Condition::Undead),
        },
    },
    // Bio Mortem: 4 range, 2 AoE, 6 CT, 24 MP. Effect: Damage Faith(MA * 24); Chance to Add Death.
    Ability {
        name: "Bio Mortem",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 24,
        aoe: AoE::Diamond(2, Some(1)),
        implementation: &ByblosElemental {
            element: Element::None,
            q: 24,
            range: 4,
            ctr: Some(6),
            condition: Some(Condition::Death),
        },
    },
    // Bio Insanis: 4 range, 2 AoE, 6 CT, 24 MP. Effect: Damage Faith(MA * 24); Chance to Add Confusion.
    Ability {
        name: "Bio Insanis",
        flags: CASTER_IMMUNE | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED,
        mp_cost: 24,
        aoe: AoE::Diamond(2, Some(1)),
        implementation: &ByblosElemental {
            element: Element::None,
            q: 24,
            range: 4,
            ctr: Some(6),
            condition: Some(Condition::Confusion),
        },
    },
    // Vengeance: 5 range, 0 AoE. Effect: Damage (CasterMaxHP - CasterCurrentHP).
    Ability {
        name: "Vengeance",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &UlmaguestImpl { range: 5 },
    },
    // TODO: Manaburn: 5 range, 0 AoE. Effect: Damage (TargetCurrentMP).
    // TODO: Energize: 4 range, 0 AoE. Effect: Heal (CasterMaxHP * 2 / 5); DamageCaster (CasterMaxHP / 5).
    // Parasite: 4 range, 0 AoE. Effect: Add Petrify, Darkness, Confusion, Silence, Oil, Frog, Poison, Sleep (Separate).
    Ability {
        name: "Parasite",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &BadBreathImpl {
            conditions: &[
                Condition::Petrify,
                Condition::Darkness,
                Condition::Confusion,
                Condition::Silence,
                Condition::Oil,
                Condition::Frog,
                Condition::Poison,
                Condition::Sleep,
            ],
            range: 4,
        },
    },
];

struct ByblosElemental {
    element: Element,
    q: i16,
    range: u8,
    ctr: Option<u8>,
    condition: Option<Condition>,
}

impl AbilityImpl for ByblosElemental {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.ally(target) && !target.absorbs(self.element) {
            return;
        }
        if user.foe(target) && target.absorbs(self.element) {
            return;
        }
        actions.push(Action::new(ability, self.range, self.ctr, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        if target.cancels(self.element) {
            return;
        }
        if sim.do_magical_evade(user, target, Source::Ability) {
            return;
        }
        let damage_amount = mod_5_formula(user, target, self.element, self.q);
        sim.change_target_hp(target_id, damage_amount, Source::Ability);
        if let Some(cond) = self.condition {
            if sim.roll_auto_succeed() < 0.25 {
                sim.add_condition(target_id, cond, Source::Ability);
            }
        }
    }
}
