use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, TARGET_NOT_SELF,
    TARGET_SELF_ONLY,
};

pub const THROW_ABILITIES: &[Ability] = &[
    // Knife: movement range, 0 AoE. Effect: Throw Knife Damage (Speed * ThrownWP).
    // Sword: movement range, 0 AoE. Effect: Throw Sword Damage (Speed * ThrownWP).
    // Hammer: movement range, 0 AoE. Effect: Throw Flail Damage (Speed * ThrownWP).
    // Staff: movement range, 0 AoE. Effect: Throw Staff Damage (Speed * ThrownWP).
    // Ninja Sword: movement range, 0 AoE. Effect: Throw Ninja Blade Damage (Speed * ThrownWP).
    // Axe: movement range, 0 AoE. Effect: Throw Axe Damage (Speed * ThrownWP).
    // Spear: movement range, 0 AoE. Effect: Throw Spear Damage (Speed * ThrownWP).
    // Stick: movement range, 0 AoE. Effect: Throw Pole Damage (Speed * ThrownWP).
    // Wand: movement range, 0 AoE. Effect: Throw Rod Damage (Speed * ThrownWP).
    // Dictionary: movement range, 0 AoE. Effect: Throw Book Damage (Speed * ThrownWP).

    // Shuriken: movement range, 0 AoE. Effect: Throw Shuriken Damage (Speed * ThrownWP).
    // // Shuriken: 5 WP, Shuriken.
    // // Magic Shuriken: 7 WP, Shuriken. Element: Ice.
    // // Yagyu Shuriken: 9 WP, Shuriken. Element: Dark.
    Ability {
        name: "Shuriken",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: None,
        implementation: &ThrowImpl {
            items: &[
                ThrowableItem {
                    wp: 5,
                    element: Element::None,
                    name: "Shuriken",
                },
                ThrowableItem {
                    wp: 7,
                    element: Element::Ice,
                    name: "Magic Shuriken",
                },
                ThrowableItem {
                    wp: 9,
                    element: Element::Dark,
                    name: "Yagyu Shuriken",
                },
            ],
        },
    },
    // Bomb: movement range, 0 AoE. Effect: Throw Bomb Damage (Speed * ThrownWP).
    // // Burst Bomb: 8 WP, Bomb. Element: Fire.
    // // Torrent Bomb: 8 WP, Bomb. Element: Water.
    // // Spark Bomb: 8 WP, Bomb. Element: Lightning.
    Ability {
        name: "Bomb",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: None,
        implementation: &ThrowImpl {
            items: &[
                ThrowableItem {
                    wp: 8,
                    element: Element::Fire,
                    name: "Burst Bomb",
                },
                ThrowableItem {
                    wp: 8,
                    element: Element::Water,
                    name: "Torrent Bomb",
                },
                ThrowableItem {
                    wp: 8,
                    element: Element::Lightning,
                    name: "Spark Bomb",
                },
            ],
        },
    },
];

struct ThrowableItem {
    wp: i16,
    element: Element,
    name: &'static str,
}

struct ThrowImpl {
    items: &'static [ThrowableItem],
}

impl AbilityImpl for ThrowImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, user.movement(), None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        if sim.do_physical_evade(user, target, Source::Ability) {
            sim.try_countergrasp(user_id, target_id);
            return;
        }

        if user.ally(target) {
            let least_damage = self
                .items
                .iter()
                .map(|item| (throw_formula(user, target, item.element, item.wp), item))
                .min_by_key(|p| p.0);

            if let Some(least_damage) = least_damage {
                sim.change_target_hp(
                    target_id,
                    least_damage.0,
                    Source::Constant(least_damage.1.name),
                );
                sim.try_countergrasp(user_id, target_id);
            }
        } else {
            let most_damage = self
                .items
                .iter()
                .map(|item| (throw_formula(user, target, item.element, item.wp), item))
                .max_by_key(|p| p.0);

            if let Some(most_damage) = most_damage {
                sim.change_target_hp(
                    target_id,
                    most_damage.0,
                    Source::Constant(most_damage.1.name),
                );
                sim.try_countergrasp(user_id, target_id);
            }
        }
    }
}

fn throw_formula(user: &Combatant, target: &Combatant, element: Element, wp: i16) -> i16 {
    let mut speed = user.speed() as i16;

    //   1. If target has Defense UP, then (Sp1 = [Sp0 * 2/3]), else Sp1 = Sp0
    if target.defense_up() {
        speed = (speed * 2) / 3;
    }

    //   2. If target has Protect, then (Sp2 = [Sp1 * 2/3]), else Sp2 = Sp1
    if target.protect() {
        speed = (speed * 2) / 3;
    }
    //   3. If target is Charging, then (Sp3 = [Sp2 * 3/2]), else Sp3 = Sp2
    if target.charging() {
        speed = (speed * 3) / 2;
    }

    //   4. If target is Sleeping, then (Sp4 = [Sp3 * 3/2]), else Sp4 = Sp3
    if target.sleep() {
        speed = (speed * 3) / 2;
    }

    //   5. If target is a Frog and/or Chicken, then (Sp5 = [Sp4 * 3/2]), else
    //      Sp5 = Sp4
    if target.frog() || target.chicken() {
        speed = (speed * 3) / 2;
    }

    //   6. Apply zodiac multipliers:
    //            If compatibility is 'Good', then (Sp6 = Sp5 + [(Sp5)/4]))
    //            ElseIf compatibility is 'Bad', then (Sp6 = Sp5 - [(Sp5)/4])
    //            ElseIf compatibility is 'Best', then (Sp6 = Sp5 + [(Sp5)/2])
    //            ElseIf compatibility is 'Worst', then (Sp6 = Sp5 - [(Sp5)/2])
    //            Else Sp6 = Sp5
    speed = (speed as f32 * user.zodiac_compatibility(target)) as i16;

    //   7. damage0 = Sp6 * (ThrownWpnPwr)
    let mut damage = speed * wp;

    //   8. If target is 'Weak' against the weapon's element, then
    //          damage1 = damage0 * 2
    //        Else, damage1 = damage0
    if target.weak(element) {
        damage *= 2;
    }

    //   9. If target has 'Half' against the weapon's element, then
    //          damage2 = [damage1 / 2]
    //        Else, damage2 = damage1
    if target.halves(element) {
        damage /= 2;
    }

    //  10. If target has 'Absorb' against the weapon's element, then
    //          damage3 = -(damage2)
    //        Else, damage3 = damage2
    if target.absorbs(element) {
        damage = -damage;
    }

    //  11. The damage done by the THROW attack will be equal to damage3.
    damage
}
