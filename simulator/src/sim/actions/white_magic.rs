use crate::sim::actions::{
    Ability, AbilityImpl, Action, AoE, ALLY_OK, CAN_BE_CALCULATED, CAN_BE_REFLECTED, FOE_OK,
    NOT_ALIVE_OK, PETRIFY_OK, SILENCEABLE, UNDER_50_PERCENT_HP_ONLY,
};
use crate::sim::common::{
    do_hp_heal, should_heal_ally, should_heal_foe, AddConditionSpellImpl, ConditionClearSpellImpl,
    CureSpellImpl, ElementalDamageSpellImpl,
};
use crate::sim::{Combatant, CombatantId, Condition, Element, Event, Simulation};

pub const WHITE_MAGIC_ABILITIES: &[Ability] = &[
    // Cure: 5 range, 1 AoE, 3 CT, 6 MP. Effect: Heal Faith(MA * 15).
    Ability {
        name: "Cure",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 6,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &CureSpellImpl {
            q: 15,
            ctr: Some(3),
            range: 5,
        },
    },
    // Cure 2: 5 range, 1 AoE, 4 CT, 10 MP. Effect: Heal Faith(MA * 20).
    Ability {
        name: "Cure 2",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 10,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &CureSpellImpl {
            q: 20,
            ctr: Some(4),
            range: 5,
        },
    },
    // Cure 3: 5 range, 1 AoE, 6 CT, 16 MP. Effect: Heal Faith(MA * 30).
    Ability {
        name: "Cure 3",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 16,
        aoe: AoE::Diamond(1, Some(2)),
        implementation: &CureSpellImpl {
            q: 30,
            ctr: Some(6),
            range: 5,
        },
    },
    // Cure 4: 5 range, 1 AoE, 8 CT, 24 MP. Effect: Heal Faith(MA * 40).
    Ability {
        name: "Cure 4",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 24,
        aoe: AoE::Diamond(1, Some(2)),
        implementation: &CureSpellImpl {
            q: 40,
            ctr: Some(8),
            range: 5,
        },
    },
    // Raise: 5 range, 0 AoE, 4 CT, 10 MP. Hit: Faith(MA + 190)%. Effect: Cancel Death; If successful Heal (50)%.
    Ability {
        name: "Raise",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | NOT_ALIVE_OK | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 10,
        aoe: AoE::None,
        implementation: &RaiseSpellImpl {
            hp_percent: 0.5,
            base_chance: 190,
            ctr: 4,
            range: 5,
        },
    },
    // Raise 2: 5 range, 0 AoE, 10 CT, 20 MP. Hit: Faith(MA + 160)%. Effect: Cancel Death; If successful Heal (100)%.
    Ability {
        name: "Raise 2",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | NOT_ALIVE_OK | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 20,
        aoe: AoE::None,
        implementation: &RaiseSpellImpl {
            hp_percent: 1.0,
            base_chance: 160,
            ctr: 10,
            range: 5,
        },
    },
    // Reraise: 4 range, 0 AoE, 7 CT, 16 MP. Hit: Faith(MA + 140)%. Effect: Add Reraise.
    Ability {
        name: "Reraise",
        flags: ALLY_OK
            | SILENCEABLE
            | UNDER_50_PERCENT_HP_ONLY
            | CAN_BE_REFLECTED
            | CAN_BE_CALCULATED,
        mp_cost: 16,
        aoe: AoE::None,
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Reraise],
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 140,
            ctr: 7,
            range: 4,
        },
    },
    // Regen: 4 range, 1 AoE, 4 CT, 8 MP. Hit: Faith(MA + 170)%. Effect: Add Regen.
    Ability {
        name: "Regen",
        flags: ALLY_OK
            | SILENCEABLE
            | UNDER_50_PERCENT_HP_ONLY
            | CAN_BE_REFLECTED
            | CAN_BE_CALCULATED,
        mp_cost: 8,
        aoe: AoE::Diamond(1, Some(0)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Regen],
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 170,
            ctr: 4,
            range: 4,
        },
    },
    // Protect: 4 range, 1 AoE, 3 CT, 6 MP. Hit: Faith(MA + 200)%. Effect: Add Protect.
    Ability {
        name: "Protect",
        flags: ALLY_OK
            | SILENCEABLE
            | UNDER_50_PERCENT_HP_ONLY
            | CAN_BE_REFLECTED
            | CAN_BE_CALCULATED,
        mp_cost: 6,
        aoe: AoE::Diamond(1, Some(0)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Protect],
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 200,
            ctr: 3,
            range: 4,
        },
    },
    // Protect 2: 4 range, 1 AoE, 6 CT, 18 MP. Hit: Faith(MA + 240)%. Effect: Add Protect.
    Ability {
        name: "Protect 2",
        flags: ALLY_OK | UNDER_50_PERCENT_HP_ONLY | SILENCEABLE,
        mp_cost: 18,
        aoe: AoE::Diamond(1, Some(3)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Protect],
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 240,
            ctr: 6,
            range: 4,
        },
    },
    // Shell: 4 range, 1 AoE, 3 CT, 6 MP. Hit: Faith(MA + 200)%. Effect: Add Shell.
    Ability {
        name: "Shell",
        flags: ALLY_OK
            | SILENCEABLE
            | UNDER_50_PERCENT_HP_ONLY
            | CAN_BE_REFLECTED
            | CAN_BE_CALCULATED,
        mp_cost: 6,
        aoe: AoE::Diamond(1, Some(0)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Shell],
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 200,
            ctr: 3,
            range: 4,
        },
    },
    // Shell 2: 4 range, 1 AoE, 6 CT, 18 MP. Hit: Faith(MA + 240)%. Effect: Add Shell.
    Ability {
        name: "Shell 2",
        flags: ALLY_OK | UNDER_50_PERCENT_HP_ONLY | SILENCEABLE,
        mp_cost: 18,
        aoe: AoE::Diamond(1, Some(3)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Shell],
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 240,
            ctr: 6,
            range: 4,
        },
    },
    // Wall: 4 range, 1 AoE, 4 CT, 24 MP. Hit: Faith(MA + 140)%. Effect: Add Protect, Shell (All).
    Ability {
        name: "Wall",
        flags: ALLY_OK | SILENCEABLE,
        mp_cost: 24,
        aoe: AoE::Diamond(1, Some(3)),
        implementation: &AddConditionSpellImpl {
            condition: &[Condition::Protect, Condition::Shell],
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 140,
            ctr: 4,
            range: 4,
        },
    },
    // Esuna: 5 range, 1 AoE, 3 CT, 16 MP. Hit: Faith(MA + 195)%. Effect: Cancel Petrify, Darkness,
    //  Confusion, Silence, Blood Suck, Berserk, Frog, Poison, Sleep, Don't Move, Don't Act.
    Ability {
        name: "Esuna",
        flags: ALLY_OK | PETRIFY_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 16,
        aoe: AoE::Diamond(1, Some(2)),
        implementation: &ConditionClearSpellImpl {
            conditions: &[
                Condition::Petrify,
                Condition::Darkness,
                Condition::Confusion,
                Condition::Silence,
                Condition::BloodSuck,
                Condition::Berserk,
                Condition::Frog,
                Condition::Poison,
                Condition::Sleep,
                Condition::DontMove,
                Condition::DontAct,
            ],
            ignore_magic_def: true,
            base_chance: 195,
            ctr: 3,
            range: 4,
        },
    },
    // Holy: 5 range, 0 AoE, 6 CT, 56 MP. Element: Holy. Effect: Damage Faith(MA * 47).
    Ability {
        name: "Holy",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | CAN_BE_REFLECTED | CAN_BE_CALCULATED,
        mp_cost: 56,
        aoe: AoE::None,
        implementation: &ElementalDamageSpellImpl {
            element: Element::Holy,
            q: 47,
            ctr: Some(6),
            range: 5,
            evadable: true,
        },
    },
    // Magic Barrier: 3 range, 11 CT, 35 MP. Hit: Faith(MA + 200)%. Effect: Add Reraise, Regen, Protect, Shell, Haste (All).
    Ability {
        name: "Magic Barrier",
        flags: ALLY_OK | SILENCEABLE,
        mp_cost: 35,
        aoe: AoE::Diamond(1, Some(3)),
        implementation: &AddConditionSpellImpl {
            condition: &[
                Condition::Reraise,
                Condition::Regen,
                Condition::Protect,
                Condition::Shell,
                Condition::Haste,
            ],
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 200,
            ctr: 11,
            range: 3,
        },
    },
];

struct RaiseSpellImpl {
    hp_percent: f32,
    base_chance: i16,
    ctr: u8,
    range: u8,
}

impl AbilityImpl for RaiseSpellImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.ally(target) && !should_heal_ally(target, true) {
            return;
        }
        if user.foe(target) && !should_heal_foe(target, true) {
            return;
        }
        if !target.dead() && !target.undead() {
            return;
        }
        actions.push(Action::new(
            ability,
            self.range,
            Some(self.ctr),
            target.id(),
        ));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let mut success_chance = 1.0;
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        if !target.dead() && !target.undead() {
            return;
        }
        success_chance *= user.faith_percent();
        success_chance *= target.faith_percent();
        success_chance *= (user.ma() as f32 + self.base_chance as f32) / 100.0;
        success_chance *= user.zodiac_compatibility(target);

        if !(sim.roll_auto_succeed() < success_chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }

        let heal_amount = ((target.max_hp() as f32 * self.hp_percent) as i16).max(1);
        do_hp_heal(sim, target_id, heal_amount, true);
    }
}
