use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK, SILENCEABLE};
use crate::sim::common::{mod_5_formula_xa, mod_6_formula, ElementalDamageSpellImpl, EmpowerImpl};
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Event, Simulation, Source, TARGET_NOT_SELF,
    TARGET_SELF_ONLY,
};

pub const ULTIMA_DEMON_ABILITIES: &[Ability] = &[
    // Nanoflare: 4 range, 2 AoE, 5 CT. Effect: Damage ((MA + 5) / 2 * MA).
    Ability {
        name: "Nanoflare",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 0,
        aoe: AoE::Diamond(2),
        implementation: &NanoflareImpl {
            ma_plus: 5,
            ctr: 5,
            range: 4,
        },
    },
    // Dark Holy: 5 range, 0 AoE, 7 CT, 40 MP. Element: Dark. Effect: Damage Faith(MA * 41).
    Ability {
        name: "Dark Holy",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 40,
        aoe: AoE::None,
        implementation: &ElementalDamageSpellImpl {
            element: Element::Dark,
            q: 41,
            ctr: Some(7),
            range: 5,
            evadable: true,
        },
    },
    // Ultima: 5 range, 1 AoE, 5 CT, 10 MP. Element: Holy. Effect: Damage Faith(MA * 25).
    Ability {
        name: "Ultima",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 10,
        aoe: AoE::Diamond(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Holy,
            q: 25,
            ctr: Some(5),
            range: 5,
            evadable: true,
        },
    },
    // Hurricane: 4 range, 2 AoE. Element: Wind. Hit: (MA + 50)%. Effect: Damage (35)%.
    Ability {
        name: "Hurricane",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(2),
        implementation: &HuricaneImpl {
            element: Element::Wind,
            base_chance: 50,
            range: 4,
        },
    },
    // Ulmaguest: 4 range, 2 AoE. Effect: Damage (CasterMaxHP - CasterCurrentHP).
    Ability {
        name: "Ulmaguest",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(2),
        implementation: &UlmaguestImpl { range: 4 },
    },
    // Empower: 4 range, 0 AoE, 8 CT, 13 MP. Effect: +2 PA, +2 MA, +2 Speed.
    Ability {
        name: "Empower",
        flags: ALLY_OK,
        mp_cost: 13,
        aoe: AoE::None,
        implementation: &EmpowerImpl {
            range: 4,
            ctr: Some(8),
            brave_mod: 0,
            pa_buff: 2,
            ma_buff: 2,
            speed_buff: 2,
        },
    },
];

struct NanoflareImpl {
    ma_plus: i16,
    range: u8,
    ctr: u8,
}

impl AbilityImpl for NanoflareImpl {
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
        let xa = mod_5_formula_xa(user.ma() as i16, user, target, Element::None, false);
        let damage = ((xa + self.ma_plus) / 2) * user.ma_bang() as i16;
        sim.change_target_hp(target_id, damage, Source::Ability);
    }
}

struct UlmaguestImpl {
    range: u8,
}

impl AbilityImpl for UlmaguestImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, self.range, None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let damage = user.max_hp() - user.hp();
        sim.change_target_hp(target_id, damage, Source::Ability);
    }
}

struct HuricaneImpl {
    element: Element,
    base_chance: i16,
    range: u8,
}

impl AbilityImpl for HuricaneImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, self.range, None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        let chance = mod_6_formula(user, target, self.element, self.base_chance, false);
        if !(sim.roll_auto_succeed() < chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }
        let damage = target.max_hp() / 3;
        sim.change_target_hp(target_id, damage, Source::Ability);
    }
}
