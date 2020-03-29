use crate::sim::actions::{Action, ActionKind};
use crate::sim::{can_move_into_range, Combatant, CombatantId, Simulation, Source};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Potion,
    HiPotion,
    XPotion,
    Elixir,
    PhoenixDown,
}

fn item_range(user: &Combatant) -> i8 {
    if user.throw_item() {
        4
    } else {
        1
    }
}

pub fn consider_item(
    actions: &mut Vec<Action>,
    sim: &Simulation,
    user: &Combatant,
    target: &Combatant,
) {
    if !user.skill_set_item() {
        return;
    }

    if user.berserk() {
        return;
    }

    let range = item_range(user);
    if !can_move_into_range(user, range, target) {
        return;
    }

    consider_phoenix_down(actions, sim, user, target);
    consider_item_heal(actions, sim, user, target);
}

pub fn should_item_heal_foe(target: &Combatant) -> bool {
    target.undead()
}

pub fn should_item_heal_ally(target: &Combatant) -> bool {
    if target.undead() {
        false
    } else {
        target.hp_percent() <= 0.50
    }
}

pub fn consider_item_heal(
    actions: &mut Vec<Action>,
    _sim: &Simulation,
    user: &Combatant,
    target: &Combatant,
) {
    if target.petrify() || target.crystal() || target.dead() {
        return;
    }

    if user.different_team(target) && !should_item_heal_foe(target) {
        return;
    }

    if user.same_team(target) && !should_item_heal_ally(target) {
        return;
    }

    // TODO: Determine if you even have potion
    let all_potions = [
        ("Elixir", Item::Elixir),
        ("X-Potion", Item::XPotion),
        ("Hi-Potion", Item::HiPotion),
        ("Potion", Item::Potion),
    ];
    let mut best_potion = None;
    for potion in all_potions.iter() {
        if user.knows_ability(potion.0) {
            best_potion = Some(potion.1);
            break;
        }
    }

    if let Some(potion) = best_potion {
        actions.push(Action {
            kind: ActionKind::Item(potion),
            range: item_range(user),
            ctr: None,
            target_id: target.id(),
        });
    }
}

pub fn perform_item(
    sim: &mut Simulation,
    user_id: CombatantId,
    target_id: CombatantId,
    item: Item,
) {
    match item {
        Item::Potion | Item::HiPotion | Item::XPotion | Item::Elixir => {
            perform_item_heal(sim, user_id, target_id, item)
        }

        Item::PhoenixDown => perform_phoenix_down(sim, user_id, target_id),
    }
}

pub fn perform_item_heal(
    sim: &mut Simulation,
    _user_id: CombatantId,
    target_id: CombatantId,
    item: Item,
) {
    let mut heal_amount = match item {
        Item::Elixir => sim.combatant(target_id).max_hp(),
        Item::XPotion => 150,
        Item::HiPotion => 120,
        Item::Potion => 100,
        _ => panic!("tried to heal with a non-healing item"),
    };
    if sim.combatant(target_id).undead() {
        heal_amount = -heal_amount;
    }

    // TODO: On this whole source thing, should just add item/ability...
    sim.change_target_hp(target_id, -heal_amount, Source::Constant("Item"));
}

pub fn consider_phoenix_down(
    actions: &mut Vec<Action>,
    _sim: &Simulation,
    user: &Combatant,
    target: &Combatant,
) {
    if !user.knows_ability("Phoenix Down") {
        return;
    }

    if target.petrify() || target.crystal() {
        return;
    }

    if user.different_team(target) && !target.dead() && target.undead() {
        actions.push(Action {
            kind: ActionKind::Item(Item::PhoenixDown),
            range: item_range(user),
            ctr: None,
            target_id: target.id(),
        });
    } else if user.same_team(target) && !target.undead() && target.dead() && !target.reraise() {
        actions.push(Action {
            kind: ActionKind::Item(Item::PhoenixDown),
            range: item_range(user),
            ctr: None,
            target_id: target.id(),
        });
    }
}

pub fn perform_phoenix_down(sim: &mut Simulation, _user_id: CombatantId, target_id: CombatantId) {
    let target = sim.combatant(target_id);
    if target.undead() && !target.dead() {
        sim.change_target_hp(target_id, target.max_hp(), Source::Constant("Phoenix Down"));
    } else if !target.undead() && target.dead() {
        let heal_amount = sim.roll_inclusive(1, 20);
        sim.change_target_hp(target_id, -heal_amount, Source::Constant("Phoenix Down"));
    }
}
