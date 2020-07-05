use crate::sim::actions::{Ability, AbilityImpl, Action, AoE, ALLY_OK, FOE_OK};
use crate::sim::common::{mod_6_formula, AddConditionSpellImpl, ConditionClearSpellImpl};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Event, Simulation, Source, CAN_BE_CALCULATED,
    CAN_BE_REFLECTED, SILENCEABLE,
};

pub const YIN_YANG_MAGIC_ABILITIES: &[Ability] = &[
    // Blind: 5 range, 1 AoE, 2 CT, 4 MP. Hit: Faith(MA + 200)%. Effect: Add Blind.
    Ability {
        name: "Blind",
        flags: FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 4,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Darkness],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 200,
            ctr: 2,
            range: 5,
        },
    },
    // Spell Absorb: 5 range, 0 AoE, 2 CT, 2 MP. Hit: Faith(MA + 175)%. Effect: AbsorbMP (33)%
    Ability {
        name: "Spell Absorb",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 2,
        aoe: AoE::None,
        implementation: &AbsorbSpellImpl {
            hp_not_mp: false,
            amount: 0.33,
            base_chance: 175,
            ctr: 2,
            range: 5,
        },
    },
    // Life Drain: 5 range, 0 AoE, 2 CT, 16 MP. Hit: Faith(MA + 185)%. Effect: AbsorbHP (25)%
    Ability {
        name: "Life Drain",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 16,
        aoe: AoE::None,
        implementation: &AbsorbSpellImpl {
            hp_not_mp: true,
            amount: 0.25,
            base_chance: 185,
            ctr: 2,
            range: 5,
        },
    },
    // Pray Faith: 5 range, 0 AoE, 4 CT, 6 MP. Hit: Faith(MA + 150)%. Effect: Add Faith.
    Ability {
        name: "Pray Faith",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 6,
        aoe: AoE::None,
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Faith],
            can_be_evaded: false,
            ignore_magic_def: false,
            base_chance: 150,
            ctr: 4,
            range: 5,
        },
    },
    // Doubt Faith: 5 range, 0 AoE, 4 CT, 6 MP. Hit: Faith(MA + 150)%. Effect: Add Innocent.
    Ability {
        name: "Doubt Faith",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 6,
        aoe: AoE::None,
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Innocent],
            can_be_evaded: false,
            ignore_magic_def: false,
            base_chance: 150,
            ctr: 4,
            range: 5,
        },
    },
    // Zombie: 5 range, 0 AoE, 5 CT, 20 MP. Hit: Faith(MA + 115)%. Effect: Add Undead.
    Ability {
        name: "Zombie",
        flags: FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 20,
        aoe: AoE::None,
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Undead],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 115,
            ctr: 5,
            range: 5,
        },
    },
    // Silence Song: 5 range, 1 AoE, 3 CT, 16 MP. Hit: Faith(MA + 180)%. Effect: Add Silence.
    Ability {
        name: "Silence Song",
        flags: FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 16,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Silence],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 180,
            ctr: 3,
            range: 5,
        },
    },
    // Blind Rage: 5 range, 0 AoE, 5 CT, 16 MP. Hit: Faith(MA + 130)%. Effect: Add Berserk.
    Ability {
        name: "Blind Rage",
        flags: FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 16,
        aoe: AoE::None,
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Berserk],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 130,
            ctr: 5,
            range: 5,
        },
    },
    // TODO Foxbird: 5 range, 0 AoE, 4 CT, 20 MP. Hit: Faith(MA + 145)%. Effect: -30 Brave.
    // Confusion Song: 5 range, 0 AoE, 5 CT, 20 MP. Hit: Faith(MA + 135)%. Effect: Add Confusion.
    Ability {
        name: "Confusion Song",
        flags: FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 20,
        aoe: AoE::None,
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Confusion],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 135,
            ctr: 5,
            range: 5,
        },
    },
    // Dispel Magic: 5 range, 0 AoE, 3 CT, 34 MP. Hit: Faith(MA + 200)%. Effect: Cancel Float, Reraise, Transparent, Regen, Protect, Shell, Haste, Faith, Reflect
    Ability {
        name: "Dispel Magic",
        flags: FOE_OK | SILENCEABLE | CAN_BE_CALCULATED,
        mp_cost: 34,
        aoe: AoE::None,
        implementation: &ConditionClearSpellImpl {
            conditions: &[
                Condition::Float,
                Condition::Reraise,
                Condition::Transparent,
                Condition::Regen,
                Condition::Protect,
                Condition::Shell,
                Condition::Haste,
                Condition::Faith,
                Condition::Reflect,
            ],
            base_chance: 200,
            ignore_magic_def: false,
            ctr: 3,
            range: 5,
        },
    },
    // Paralyze: 5 range, 1 AoE, 5 CT, 10 MP. Hit: Faith(MA + 185)%. Effect: Add Don't Act.
    Ability {
        name: "Paralyze",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 10,
        aoe: AoE::Diamond(1, Some(0)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::DontAct],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 185,
            ctr: 5,
            range: 5,
        },
    },
    // Sleep: 5 range, 1 AoE, 6 CT, 24 MP. Hit: Faith(MA + 175)%. Effect: Add Sleep.
    Ability {
        name: "Sleep",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 24,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Sleep],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 175,
            ctr: 6,
            range: 5,
        },
    },
    // Petrify: 5 range, 0 AoE, 9 CT, 16 MP. Hit: Faith(MA + 125)%. Effect: Add Petrify.
    Ability {
        name: "Petrify",
        flags: FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 16,
        aoe: AoE::None,
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Petrify],
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 125,
            ctr: 9,
            range: 5,
        },
    },
];

struct AbsorbSpellImpl {
    hp_not_mp: bool,
    amount: f32,
    base_chance: i16,
    range: u8,
    ctr: u8,
}

impl AbilityImpl for AbsorbSpellImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(
            ability,
            self.range,
            Some(self.ctr),
            target.id(),
        ));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        if sim.do_magical_evade(user, target, Source::Ability) {
            return;
        }
        let success_chance = mod_6_formula(user, target, Element::None, self.base_chance, false);
        if !(sim.roll_auto_succeed() < success_chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }

        if self.hp_not_mp {
            let absorbed_amount = (target.max_hp() as f32 * self.amount) as i16;
            sim.change_target_hp(target_id, absorbed_amount, Source::Ability);
            sim.change_target_hp(user_id, -absorbed_amount, Source::Ability);
        } else {
            let absorbed_amount = (target.max_mp() as f32 * self.amount) as i16;
            sim.change_target_mp(target_id, absorbed_amount, Source::Ability);
            sim.change_target_mp(user_id, -absorbed_amount, Source::Ability);
        }
    }
}
