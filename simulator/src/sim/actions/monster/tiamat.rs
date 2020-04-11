use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::attack::AttackImpl;
use crate::sim::common::{
    mod_5_formula, mod_5_formula_xa, mod_6_formula, DemiImpl, ElementalDamageSpellImpl,
};
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Event, Simulation, Source, TARGET_NOT_SELF,
    TARGET_SELF_ONLY,
};

pub const TIAMAT_ABILITIES: &[Ability] = &[
    // Triple Attack: 1 range, 1 AoE (x3 line). Effect: Normal Attack.
    Ability {
        name: "Triple Attack",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::TriLine,
        implementation: &AttackImpl {},
    },
    // Triple Breath: 2 range, 2 AoE (x3 line). Hit: (MA + 90)%. Effect: Damage (45)%.
    Ability {
        name: "Triple Breath",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::TriLine,
        implementation: &DemiImpl {
            base_chance: 90,
            hp_percent: 0.45,
            range: 2,
            ctr: None,
        },
    },
    // Triple Thunder: 4 range, 1 AoE, 3 CT, 10 MP. Element: Lightning. Hit: 3 times. Effect: Damage ((MA + 12) / 2 * MA).
    Ability {
        name: "Triple Thunder",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 10,
        aoe: AoE::None, // handled in the ability itself.
        implementation: &TripleElementalImpl {
            ma_plus: 12,
            element: Element::Lightning,
            range: 4,
            ctr: Some(3),
        },
    },
    // Triple Flame: 4 range, 1 AoE, 3 CT, 10 MP. Element: Fire. Hit: 3 times. Effect: Damage ((MA + 20) / 2 * MA).
    Ability {
        name: "Triple Flame",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 10,
        aoe: AoE::None, // handled in the ability itself.
        implementation: &TripleElementalImpl {
            ma_plus: 20,
            element: Element::Fire,
            range: 4,
            ctr: Some(3),
        },
    },
    // TODO: Noxious Gas: 3 range, 1 AoE. Hit: Faith(MA + 223)%. Effect: Add Defending, Darkness, Oil, Poison, Slow, Don't Move, Death Sentence (Separate).
];

struct TripleElementalImpl {
    ma_plus: i16,
    ctr: Option<u8>,
    element: Element,
    range: i8,
}

impl AbilityImpl for TripleElementalImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        // This will only very roughly simulate the 'random hits' AI characteristic
        if sim.roll_inclusive(1, 2) == 1 {
            return;
        }
        if user.ally(target) && !target.absorbs(self.element) {
            return;
        }
        if user.foe(target) && target.absorbs(self.element) {
            return;
        }
        actions.push(Action::new(ability, self.range, self.ctr, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let target_location = sim.combatant(target_id).location;

        for _i in 1..=3 {
            let panel_roll = sim.roll_inclusive(1, 5) as usize;
            let panel = target_location
                .diamond(1)
                .skip(panel_roll - 1)
                .next()
                .unwrap();
            let option_target_id = sim.combatant_on_panel(panel);
            if let Some(target) = option_target_id.map(|id| sim.combatant(id)) {
                let user = sim.combatant(user_id);
                if target.cancels(self.element) {
                    continue;
                }
                let xa = mod_5_formula_xa(user.ma() as i16, user, target, self.element, false);
                let damage = ((xa + self.ma_plus) / 2) * user.ma() as i16;
                sim.change_target_hp(target_id, damage, Source::Ability);
            }
        }
    }
}
