use crate::sim::{can_move_into_range, Combatant, CombatantId, Event, Simulation, Source};
use crate::sim::actions::{Action, ActionKind};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WhiteMagic {
    Cure,
    Cure2,
    Cure3,
    Cure4,
    Raise,
    Raise2,
    Reraise,
    Regen,
    Protect,
    Protect2,
    Esuna,
    Holy,
}

pub fn consider_white_magic(
    actions: &mut Vec<Action>,
    sim: &Simulation,
    user: &Combatant,
    target: &Combatant,
) {
    if !user.skill_set_white_magic() {
        return;
    }
    if user.silence() || target.innocent() {
        return;
    }
    consider_cure(actions, sim, user, target);
    consider_raise(actions, sim, user, target);
}

pub fn consider_cure(
    actions: &mut Vec<Action>,
    _sim: &Simulation,
    user: &Combatant,
    target: &Combatant,
) {
    if !user.knows_ability("Cure") {
        return;
    }
    if target.dead() || target.petrify() || target.crystal() {
        return;
    }
    if user.same_team(target) && (target.undead() || target.hp_percent() > 0.50) {
        return;
    }
    if user.different_team(target) && !target.undead() {
        return;
    }
    if !can_move_into_range(user, 4, target) {
        return;
    }
    if user.mp() < 6 {
        return;
    }
    actions.push(Action {
        kind: ActionKind::WhiteMagic(WhiteMagic::Cure),
        range: 4,
        ctr: Some(4),
        target_id: target.id(),
    });
}

pub fn consider_raise(
    actions: &mut Vec<Action>,
    _sim: &Simulation,
    user: &Combatant,
    target: &Combatant,
) {
    if !user.knows_ability("Raise") {
        return;
    }
    if target.petrify() || target.crystal() {
        return;
    }
    if user.same_team(target) && (target.undead() || !target.dead()) {
        return;
    }
    if user.different_team(target) && (!target.undead() || target.dead()) {
        return;
    }
    if !can_move_into_range(user, 4, target) {
        return;
    }
    if user.mp() < 10 {
        return;
    }
    actions.push(Action {
        kind: ActionKind::WhiteMagic(WhiteMagic::Raise),
        range: 4,
        ctr: Some(4),
        target_id: target.id(),
    });
}

pub fn perform_white_magic(
    sim: &mut Simulation,
    user_id: CombatantId,
    target_id: CombatantId,
    spell: WhiteMagic,
) {
    if sim.combatant(user_id).silence() {
        sim.log_event(Event::Silenced(user_id));
    }
    match spell {
        // | [Cure]                         [ 001 ]                          WHITE MAGIC |
        // |=============================================================================|
        // | magical  | CBG: - |  MP:   6   | Restore [CFa/100 * TFa/100 * MA * 14] HP   |
        // | REFL: +  |  CM: - | CTR:   4   | If target is Undead, HP is subtracted      |
        // | CALC: +  |  CF: - |  JP:  50   | instead of added.                          |
        // | ELEM: -  | EVD: - | MOD:   5   | Ignores Shell and Magic DefendUP.          |
        // |--------------------------------|                                            |
        // | Range: 4 / Effect: 2v1         |                                            |
        //  -----------------------------------------------------------------------------
        WhiteMagic::Cure => {
            let user = sim.combatant(user_id);
            if user.mp() < 6 {
                sim.log_event(Event::NoMP(user_id));
                return;
            }
            let target = sim.combatant(target_id);
            if target.dead() || target.crystal() || target.petrify() {
                sim.log_event(Event::SlowActionTargetDied(target_id));
                return;
            }

            let mut heal_amount = 1.0;
            heal_amount *= user.faith_percent();
            heal_amount *= target.faith_percent();
            heal_amount *= user.ma() as f32;
            heal_amount *= 14.0;
            heal_amount *= user.zodiac_compatibility(target);

            if target.undead() {
                heal_amount = -heal_amount;
            }
            sim.change_target_hp(target_id, -heal_amount as i16, Source::Constant("Cure"));
        }
        //  _____________________________________________________________________________
        // | [Raise]                        [ 005 ]                          WHITE MAGIC |
        // |=============================================================================|
        // | magical  | CBG: - |  MP:  10   | Cancel: Dead & Restore RU{T_MaxHP * 50/100}|
        // | REFL: +  |  CM: - | CTR:   4   | Spell will miss unless target is Dead.     |
        // | CALC: +  |  CF: - |  JP: 180   | If target is Undead, RU{T_MaxHP * 50/100}  |
        // | ELEM: -  | EVD: - | MOD:   6   |  will be substracted from its HP total.    |
        // |--------------------------------| If target is Dead and Undead, spell will   |
        // | Range: 4 / Effect: 1           |  miss. Ignores Shell and Magic DefendUP.   |
        // |                                | Success% = [CFa/100 * TFa/100 * (MA + 180)]|
        //  -----------------------------------------------------------------------------
        WhiteMagic::Raise => {
            // TODO: SUCCESS CHANCE lol

            let user = sim.combatant(user_id);
            if user.mp() < 10 {
                sim.log_event(Event::NoMP(user_id));
                return;
            }
            let target = sim.combatant(target_id);
            if target.crystal() || target.petrify() || (!target.dead() && !target.undead()) {
                return;
            }

            let mut success_chance = 1.0;
            success_chance *= user.faith_percent();
            success_chance *= target.faith_percent();
            success_chance *= (user.ma() as f32 + 180.0) / 100.0;
            success_chance *= user.zodiac_compatibility(target);

            if !(sim.roll_auto_succeed() < success_chance) {
                // TODO: Log spell failed.
                return;
            }

            let mut heal_amount = ((target.max_hp() * 50) / 100).max(1);
            if target.undead() {
                heal_amount = -heal_amount;
            }
            sim.change_target_hp(target_id, -heal_amount, Source::Constant("Raise"));
        }
        _ => {
            panic!("other white magic spells have not been implemented");
        }
    }
}

//  _____________________________________________________________________________
// | [Cure 2]                       [ 002 ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  10   | Restore [CFa/100 * TFa/100 * MA * 20] HP   |
// | REFL: +  |  CM: - | CTR:   5   | If target is Undead, HP is subtracted      |
// | CALC: +  |  CF: - |  JP: 180   | instead of added.                          |
// | ELEM: -  | EVD: - | MOD:   5   | Ignores Shell and Magic DefendUP.          |
// |--------------------------------|                                            |
// | Range: 4 / Effect: 2v1         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Cure 3]                       [ 003 ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  16   | Restore [CFa/100 * TFa/100 * MA * 30] HP   |
// | REFL: +  |  CM: - | CTR:   7   | If target is Undead, HP is subtracted      |
// | CALC: +  |  CF: - |  JP: 400   | instead of added.                          |
// | ELEM: -  | EVD: - | MOD:   5   | Ignores Shell and Magic DefendUP.          |
// |--------------------------------|                                            |
// | Range: 4 / Effect: 2v2         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Cure 4]                       [ 004 ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  20   | Restore [CFa/100 * TFa/100 * MA * 40] HP   |
// | REFL: -  |  CM: - | CTR:  10   | If target is Undead, HP is subtracted      |
// | CALC: -  |  CF: - |  JP: 700   | instead of added.                          |
// | ELEM: -  | EVD: - | MOD:   5   | Ignores Shell and Magic DefendUP.          |
// |--------------------------------|                                            |
// | Range: 4 / Effect: 2v3         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Raise 2]                      [ 006 ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  20   | Cancel: Dead & Restore (Target's Max HP    |
// | REFL: +  |  CM: - | CTR:  10   | Spell will miss unless target is Dead.     |
// | CALC: +  |  CF: - |  JP: 500   | If target is Undead, (T_MaxHP) will be     |
// | ELEM: -  | EVD: - | MOD:   6   |  subtracted from its HP total.             |
// |--------------------------------| If target is Dead and Undead, spell will   |
// | Range: 4 / Effect: 1           |  miss. Ignores Shell and Magic DefendUP.   |
// |                                | Success% = [CFa/100 * TFa/100 * (MA + 160)]|
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Reraise]                      [ 007 ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  16   | Add: Reraise                               |
// | REFL: +  |  CM: - | CTR:   7   | If target is Undead, spell will miss.      |
// | CALC: +  |  CF: - |  JP: 800   | Success% = [CFa/100 * TFa/100 * (MA + 140)]|
// | ELEM: -  | EVD: - | MOD:   6   | Ignores Shell and Magic DefendUP.          |
// |--------------------------------|                                            |
// | Range: 4 / Effect: 1           |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Regen]                        [ 008 ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:   8   | Add: Regen                                 |
// | REFL: +  |  CM: - | CTR:   4   | Success% = [CFa/100 * TFa/100 * (MA + 170)]|
// | CALC: +  |  CF: - |  JP: 300   | Ignores Shell and Magic DefendUP.          |
// | ELEM: -  | EVD: - | MOD:   6   |                                            |
// |--------------------------------|                                            |
// | Range: 3 / Effect: 2v0         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Protect]                      [ 009 ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:   6   | Add: Protect                               |
// | REFL: +  |  CM: - | CTR:   4   | Success% = [CFa/100 * TFa/100 * (MA + 200)]|
// | CALC: +  |  CF: - |  JP:  70   | Ignores Shell and Magic DefendUP.          |
// | ELEM: -  | EVD: - | MOD:   6   |                                            |
// |--------------------------------|                                            |
// | Range: 3 / Effect: 2v0         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Protect 2]                    [ 00A ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  24   | Add: Protect                               |
// | REFL: -  |  CM: - | CTR:   7   | Success% = [CFa/100 * TFa/100 * (MA + 120)]|
// | CALC: -  |  CF: - |  JP: 500   | Ignores Shell and Magic DefendUP.          |
// | ELEM: -  | EVD: - | MOD:   6   |                                            |
// |--------------------------------|                                            |
// | Range: 3 / Effect: 2v3         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Shell]                        [ 00B ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:   6   | Add: Shell                                 |
// | REFL: +  |  CM: - | CTR:   4   | Success% = [CFa/100 * TFa/100 * (MA + 200)]|
// | CALC: +  |  CF: - |  JP:  70   | Ignores Shell and Magic DefendUP.          |
// | ELEM: -  | EVD: - | MOD:   6   |                                            |
// |--------------------------------|                                            |
// | Range: 3 / Effect: 2v0         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Shell 2]                      [ 00C ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  20   | Add: Shell                                 |
// | REFL: -  |  CM: - | CTR:   7   | Success% = [CFa/100 * TFa/100 * (MA + 120)]|
// | CALC: -  |  CF: - |  JP: 500   | Ignores Shell and Magic DefendUP.          |
// | ELEM: -  | EVD: - | MOD:   6   |                                            |
// |--------------------------------|                                            |
// | Range: 3 / Effect: 2v3         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Wall]                         [ 00D ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  24   | Add: Protect, Shell                        |
// | REFL: +  |  CM: - | CTR:   4   | Success% = [CFa/100 * TFa/100 * (MA + 140)]|
// | CALC: +  |  CF: - |  JP: 380   | Ignores Shell and Magic DefendUP.          |
// | ELEM: -  | EVD: - | MOD:   6   |                                            |
// |--------------------------------|                                            |
// | Range: 3 / Effect: 1           |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Esuna]                        [ 00E ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  18   | Cancel: Petrify, Darkness, Confusion,      |
// | REFL: +  |  CM: - | CTR:   3   |         Silence, Berserk, Frog, Poison,    |
// | CALC: +  |  CF: - |  JP: 280   |         Sleep, Don't Move, Don't Act       |
// | ELEM: -  | EVD: - | MOD:   6   | Success% = [CFa/100 * TFa/100 * (MA + 190)]|
// |--------------------------------| Ignores Shell and Magic DefendUP.          |
// | Range: 3 / Effect: 2v2         |                                            |
//  -----------------------------------------------------------------------------
//  _____________________________________________________________________________
// | [Holy]                         [ 00F ]                          WHITE MAGIC |
// |=============================================================================|
// | magical  | CBG: - |  MP:  56   | Damage = [CFa/100 * TFa/100 * MA * 50]     |
// | REFL: +  |  CM: + | CTR:   6   |                                            |
// | CALC: +  |  CF: - |  JP: 600   |                                            |
// | ELEM: H  | EVD: - | MOD:   5   |                                            |
// |--------------------------------|                                            |
// | Range: 5 / Effect: 1           |                                            |
//  -----------------------------------------------------------------------------
