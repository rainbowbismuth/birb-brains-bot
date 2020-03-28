use crate::sim::{Combatant, CombatantId, Simulation};
use crate::sim::actions::Action;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Potion,
    HiPotion,
    XPotion,
    PhoenixDown,
}

pub fn consider_item(actions: &mut Vec<Action>, sim: &Simulation, user: &Combatant, target: &Combatant) {
    return;
}

pub fn perform_item(sim: &mut Simulation, user: CombatantId, target: CombatantId, item: Item) {}


// import random
//
// from fftbg.simulation.abc.simulation import AbstractSimulation
// from fftbg.simulation.action import Action
// from fftbg.simulation.combatant import Combatant
//
//
// def item_range(user: Combatant) -> int:
//     if user.throw_item:
//         return 4
//     else:
//         return 1
//
//
// def consider_item(sim: AbstractSimulation, user: Combatant, target: Combatant):
//     if user.berserk:
//         return
//
//     if not sim.in_range(user, item_range(user), target):
//         return
//
//     yield from consider_phoenix_down(sim, user, target)
//     yield from consider_item_heal(sim, user, target)
//
//
// def should_item_heal_foe(target: Combatant) -> bool:
//     if target.undead:
//         return True
//     return False
//
//
// def should_item_heal_ally(target: Combatant) -> bool:
//     if target.undead:
//         return False
//     if target.hp_percent > 0.50:
//         return False
//     return True
//
//
// def consider_item_heal(sim: AbstractSimulation, user: Combatant, target: Combatant):
//     if target.petrified or target.crystal or target.dead:
//         return
//
//     if user.is_foe(target) and not should_item_heal_foe(target):
//         return
//
//     if user.is_friend(target) and not should_item_heal_ally(target):
//         return
//
//     for item in ('Elixir', 'X-Potion', 'Hi-Potion', 'Potion'):
//         if not user.has_ability(item):
//             continue
//         action_range = item_range(user)
//         yield Action(
//             range=action_range,
//             user=user,
//             target=target,
//             perform=lambda sim, user, target: do_cmd_item_heal(sim, user, item, target))
//         break
//
//
// def do_cmd_item_heal(sim: AbstractSimulation, user: Combatant, item: str, target: Combatant):
//     if item == 'Elixir':
//         heal_amount = target.max_hp
//     elif item == 'X-Potion':
//         heal_amount = 150
//     elif item == 'Hi-Potion':
//         heal_amount = 120
//     elif item == 'Potion':
//         heal_amount = 100
//     else:
//         raise Exception(f'{item} isn\'t a known healing item')
//     sim.change_target_hp(target, -heal_amount, item)
//
//
// def consider_phoenix_down(sim: AbstractSimulation, user: Combatant, target: Combatant):
//     if not user.has_ability('Phoenix Down'):
//         return
//
//     if target.petrified or target.crystal:
//         return
//
//     action_range = item_range(user)
//     if user.is_foe(target) and target.undead and not target.dead:
//         yield Action(
//             range=action_range,
//             user=user,
//             target=target,
//             perform=do_cmd_item_phoenix_down)
//
//     if user.is_friend(target) and not target.undead and target.dead and not target.reraise:
//         yield Action(
//             range=action_range,
//             user=user,
//             target=target,
//             perform=do_cmd_item_phoenix_down)
//
//
// def do_cmd_item_phoenix_down(sim: AbstractSimulation, user: Combatant, target: Combatant):
//     if target.undead and not target.dead:
//         sim.change_target_hp(target, target.max_hp, 'Phoenix Down')
//     if not target.undead and target.dead:
//         heal_amount = random.randint(1, 20)
//         sim.change_target_hp(target, -heal_amount, 'Phoenix Down')