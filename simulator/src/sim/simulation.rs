use std::cell::RefCell;

use rand;
use rand::{random, Rng};
use rand::prelude::SmallRng;

use crate::sim::{ALL_CONDITIONS, Combatant, COMBATANT_IDS, COMBATANT_IDS_LEN, CombatantId, Condition, Location, Log, Team, TIMED_CONDITIONS};

const MAX_COMBATANTS: usize = COMBATANT_IDS_LEN;
const TIME_OUT_CT: usize = 1_000;

#[derive(Clone)]
pub struct Simulation<'a> {
    pub rng: RefCell<SmallRng>,
    pub combatants: [Combatant<'a>; MAX_COMBATANTS],
    pub arena_length: i16,
    pub clock_tick: usize,
    pub trigger_reactions: bool,
    pub log: Log,
    pub slow_actions: bool,
    pub active_turns: bool,
    pub left_wins: Option<bool>,
    pub time_out_win: Option<bool>,
}

impl<'a> Simulation<'a> {
    pub fn combatant(&self, cid: CombatantId) -> &Combatant<'a> {
        &self.combatants[cid.index()]
    }

    pub fn combatant_mut(&mut self, cid: CombatantId) -> &mut Combatant<'a> {
        &mut self.combatants[cid.index()]
    }

    pub fn run(&mut self) {
        while let None = self.left_wins {
            self.tick();

            if !self.team_healthy(Team::Left) {
                self.left_wins = Some(false);
                self.time_out_win = Some(false);
            }

            if !self.team_healthy(Team::Right) {
                self.left_wins = Some(true);
                self.time_out_win = Some(false);
            }

            if self.clock_tick > TIME_OUT_CT {
                self.left_wins = Some(true);
                self.time_out_win = Some(true);
            }
        }
    }

    pub fn team_healthy(&self, team: Team) -> bool {
        self.combatants.iter()
            .filter(|combatant| combatant.team == team)
            .all(|combatant|
                !combatant.dead() && !combatant.petrify() && !combatant.blood_suck())
    }

    pub fn tick(&mut self) {
        self.phase_status_check();
        self.phase_slow_action_charging();
        if self.slow_actions {
            self.phase_slow_action_resolve();
        }
        self.phase_ct_charging();
        if self.active_turns {
            self.phase_active_turn_resolve();
        }
    }


    pub fn phase_status_check(&mut self) {
        self.clock_tick += 1;
        self.log.set_clock_tick(self.clock_tick);
        self.log.phase("Status Check");
        for combatant in &mut self.combatants {
            for condition in &TIMED_CONDITIONS {
                let removed = combatant.tick_condition(*condition).unwrap();
                if removed {
                    self.log.unit_report(
                        combatant,
                        || format!("no longer has {}", condition.name()));
                }
            }
        }
    }

    //     def phase_slow_action_charging(self):
    //         self.set_phase('Slow Action Charge')
    //         for combatant in self.combatants:
    //             if not combatant.ctr_action:
    //                 continue
    //
    //             if combatant.stop:
    //                 continue  # FIXME: Does stop just remove the slow action?
    //
    //             combatant.ctr -= 1
    //             if combatant.ctr <= 0:
    //                 self.slow_actions.append(combatant)
    //
    pub fn phase_slow_action_charging(&mut self) {
        self.log.phase("Slow Action Charge");
        for combatant in &mut self.combatants {
            // TODO: Implement slow actions
        }
    }

    //     def phase_slow_action_resolve(self):
    //         self.set_phase('Slow Action Resolve')
    //         while self.slow_actions:
    //             combatant = self.slow_actions.pop(0)
    //             if not combatant.healthy:
    //                 continue
    //             self.prepend = f'{self.colored_name(combatant)}\'s C'
    //             action = combatant.ctr_action
    //             combatant.ctr_action = None
    //             action()
    pub fn phase_slow_action_resolve(&mut self) {
        self.log.phase("Slow Action Resolve");
        for combatant in &self.combatants {
            // TODO: Implement slow action resolve
        }
        self.slow_actions = false;
    }

    pub fn phase_ct_charging(&mut self) {
        self.log.phase("CT Charging");
        for combatant in &mut self.combatants {
            if combatant.stop() || combatant.sleep() || combatant.petrify() {
                continue;
            }

            let mut speed = combatant.speed();
            if combatant.haste() {
                speed = (speed * 3) / 2;
            }
            if combatant.slow() {
                speed = (speed * 2) / 3;
            }

            combatant.ct += speed;
            if combatant.ct >= 100 {
                self.active_turns = true;
            }
        }
    }

    fn clear_active_turn_flags(&mut self) {
        for combatant in &mut self.combatants {
            combatant.on_active_turn = false;
            combatant.moved_during_active_turn = false;
            combatant.acted_during_active_turn = false;
            combatant.took_damage_during_active_turn = false;
        }
    }

    fn end_of_active_turn_checks(&mut self) {
        for cid in &COMBATANT_IDS {
            let combatant = self.combatant(*cid);
            if combatant.acted_during_active_turn || combatant.took_damage_during_active_turn {
                self.cancel_condition(*cid, Condition::Transparent, None);
            }

            let combatant = self.combatant(*cid);
            if combatant.on_active_turn {
                let mut minus_ct = 60;
                if combatant.moved_during_active_turn {
                    minus_ct += 20;
                }
                if combatant.acted_during_active_turn {
                    minus_ct += 20;
                }

                let new_ct = 60.min(combatant.ct - minus_ct);
                self.combatant_mut(*cid).ct = new_ct;
            }

            let combatant = self.combatant(*cid);
            if !(combatant.acted_during_active_turn || combatant.acted_during_active_turn) {
                self.log.unit_report(combatant, || String::from("did nothing"));
            }

            self.combatant_mut(*cid).on_active_turn = false;
        }
    }

    pub fn cancel_condition(&mut self, target_id: CombatantId, condition: Condition, src: Option<String>) {
        let target = self.combatant_mut(target_id);
        if !target.has_condition(condition) {
            return;
        }
        target.cancel_condition(condition);
        let target = self.combatant(target_id);
        match src {
            Some(src_str) => self.log.unit_report(
                target, || format!("had {} cancelled by {}", condition.name(), src_str)),
            None => self.log.unit_report(
                target, || format!("had {} cancelled", condition.name()))
        }
    }

    pub fn add_condition(&mut self, target_id: CombatantId, condition: Condition, src: Option<String>) {
        // TODO: Immunity
        //         if target.immune_to(status):
        //             return

        // TODO: Death
        //         if status == DEATH:
        //             self.target_died(target)
        //             self.unit_report(target, f'was killed by {status} from {src}')
        //             return

        let target = self.combatant_mut(target_id);
        let had_status = target.has_condition(condition);
        target.add_condition(condition);
        let target = self.combatant(target_id);
        if !had_status {
            match src {
                Some(src_str) => self.log.unit_report(
                    target, || format!("now has {} from {}", condition.name(), src_str)),
                None => self.log.unit_report(
                    target, || format!("now has {}", condition.name()))
            };
        }

        // TODO: Cancelling statuses
        //         for cancelled in CANCELLING_STATUS.get(status, []):
        //             self.cancel_status(target, cancelled, status)
    }

    pub fn phase_active_turn_resolve(&mut self) {
        self.log.phase("Active Turn Resolve");
        loop {
            let combatant = self.combatants.iter().find(|c| c.ct >= 100);
            if combatant.is_none() {
                break;
            }
            let combatant = combatant.unwrap();
            let cid = combatant.id;
            if combatant.petrify() || combatant.crystal() || combatant.stop() || combatant.sleep() {
                continue;
            }

            //             if combatant.dead and combatant.reraise and not combatant.undead:
            //                 self.change_target_hp(combatant, combatant.max_hp // 10, RERAISE)
            //                 self.cancel_status(combatant, RERAISE)
            if combatant.dead() && combatant.reraise() && !combatant.undead() {
                // TODO: Do the reraise
            }


            if combatant.dead() && !combatant.crystal() {
                let now_crystal = self.combatant_mut(cid).tick_crystal_counter();
                let combatant = self.combatant(cid);
                // TODO: undead reraise chance
                if now_crystal && combatant.undead() && false {
                    self.combatant_mut(cid).reset_crystal_counter();
                }

                let combatant = self.combatant(cid);
                if combatant.crystal() {
                    self.log.unit_report(combatant, || String::from("has become a crystal"));
                    continue;
                }
            }

            let combatant = self.combatant(cid);
            if combatant.dead() {
                continue;
            }

            self.clear_active_turn_flags();
            self.combatant_mut(cid).on_active_turn = true;

            let combatant = self.combatant(cid);
            self.log.active_turn_bar(combatant);

            if combatant.regen() {
                // TODO: Do the heal
                //                 self.change_target_hp(combatant, -(combatant.max_hp // 8), 'regen')
            }

            self.ai_do_active_turn(cid);

            let combatant = self.combatant(cid);
            if combatant.poison() {
                // TODO: Do the poison
                //                 self.change_target_hp(combatant, combatant.max_hp // 8, 'poison')
            }

            self.end_of_active_turn_checks()
        }
    }

    fn roll_brave_reaction(&self, combatant: &Combatant) -> bool {
        if combatant.berserk() || !self.trigger_reactions {
            false
        } else {
            let roll: f32 = self.rng.borrow_mut().gen();
            roll <= combatant.brave_percent()
        }
    }

    fn ai_thirteen_rule(&self) -> bool {
        let roll: f32 = self.rng.borrow_mut().gen();
        roll <= 0.137
    }

    fn ai_can_be_cowardly(&self, user: &Combatant) -> bool {
        let any_healthy = self.combatants.iter()
            .filter(|c| user.team == c.team && user.id != c.id)
            .any(|c| c.healthy());
        let all_critical = self.combatants.iter()
            .filter(|c| user.team == c.team && user.id != c.id)
            .all(|c| c.critical());
        any_healthy && !all_critical
    }

    fn do_move_with_bounds(&mut self, user_id: CombatantId, new_location: Location) {
        let arena_length = self.arena_length;
        let user = self.combatant_mut(user_id);
        let old_location = user.location;
        user.location = Location::new(
            -arena_length.max(new_location.x.min(arena_length)));
        if old_location == new_location {
            return;
        }
        user.moved_during_active_turn = true;
        self.log.report(|| format!("moved from {} to {}", old_location.x, new_location.x));
    }

    fn do_move_to_range(&mut self, user_id: CombatantId, range: i16, target_id: CombatantId) {
        let target_location = self.combatant(target_id).location;
        let user = self.combatant(user_id);
        if user.moved_during_active_turn || user.dont_move() {
            return;
        }
        // TODO: Charm?
        let desired = match user.team {
            Team::Left => target_location.x - range,
            Team::Right => target_location.x + range
        };
        let v = desired - user.location.x;
        let diff = user.movement().min(v.abs());
        let sign = if v > 0 { 1 } else { -1 };
        let new_location = Location::new(user.location.x + diff * sign);
        self.do_move_with_bounds(user_id, new_location);
    }

    fn do_move_towards_unit(&mut self, user_id: CombatantId, target_id: CombatantId) {
        let target_location = self.combatant(target_id).location;
        let user = self.combatant(user_id);
        if user.moved_during_active_turn || user.dont_move() {
            return;
        }
        if user.location.x - target_location.x > 0 {
            let new_location = Location::new(target_location.x.max(user.location.x - user.movement()));
            self.do_move_with_bounds(user_id, new_location);
        } else {
            let new_location = Location::new(target_location.x.min(user.location.x + user.movement()));
            self.do_move_with_bounds(user_id, new_location);
        }
    }

    fn do_move_out_of_combat(&mut self, user_id: CombatantId) {
        let user = self.combatant(user_id);
        if user.moved_during_active_turn || user.dont_move() {
            return;
        }
        match user.team {
            Team::Left => {
                let new_location = Location::new(user.location.x - user.movement());
                self.do_move_with_bounds(user_id, new_location);
            }
            Team::Right => {
                let new_location = Location::new(user.location.x + user.movement());
                self.do_move_with_bounds(user_id, new_location);
            }
        }
    }

    fn ai_do_active_turn(&mut self, cid: CombatantId) {}
}

pub fn can_move_into_range(user: &Combatant, range: i16, target: &Combatant) -> bool {
    user.distance(target) <= range + user.movement()
}

fn ai_calculate_target_value(user: &Combatant, target: &Combatant) -> f32 {
    let mut priority = target.hp_percent();
    priority += 0.51 * target.broken_items as f32;
    priority += ai_calculate_status_target_value_mod(target);
    priority += ai_calculate_caster_hate_mod(target);
    // TODO: Golem fear
    // TODO: Charm

    if user.different_team(target) {
        -priority
    } else {
        priority
    }
}

fn ai_calculate_caster_hate_mod(target: &Combatant) -> f32 {
    if !target.can_cast_mp_ability() {
        0.0
    } else {
        (target.mp_percent() / 16.0) * (target.number_of_mp_using_abilities as f32)
    }
}

fn ai_calculate_status_target_value_mod(target: &Combatant) -> f32 {
    let mut total = 0.0;

    // # 0x0058: Current Statuses 1
    // # 		0x80 - 							0% (0000)
    // # 		0x40 - Crystal					-150% -c0(ff40)
    // # 		0x20 - Dead						-150% -c0(ff40)
    // # 		0x10 - Undead					-30.5% -27(ffd9)
    // # 		0x08 - Charging					0% (0000)
    // # 		0x04 - Jump						0% (0000)
    // # 		0x02 - Defending				0% (0000)
    // # 		0x01 - Performing				0% (0000)
    if target.dead() {
        total -= 1.5;
    }

    if target.undead() {
        total -= 0.305;
    }

    // # 	0x0059: Current Statuses 2
    // # 		0x80 - Petrify					-90.6% -74(ff8c)
    if target.petrify() {
        total -= 0.906;
    }

    // # 		0x40 - Invite					-180.4% -e7(ff19)
    // # NOTE: Skipping Invite because it doesn't exist in FFTBG

    // # 		0x20 - Darkness					-50% [-40(ffc0) * Evadable abilities] + 3 / 4
    // # TODO: Add darkness

    // # 		0x10 - Confusion				-50% -40(ffc0) (+1 / 4 if slow/stop/sleep/don't move/act/)
    if target.confusion() {
        if target.slow() || target.stop() || target.sleep() || target.dont_move() || target.dont_act() {
            total += 0.25;
        } else {
            total -= 0.5;
        }
    }

    // # 		0x08 - Silence					-70.3% [-5a(ffa6) * Silence abilities] + 3 / 4
    if target.silence() {
        total -= 0.703;
        // # TODO: Calculate number of silenced abilities
    }

    // # 		0x04 - Blood Suck				-90.6% -74(ff8c) (+1 / 4 if slow/stop/sleep/don't move/act/)
    if target.blood_suck() {
        if target.slow() || target.stop() || target.sleep() || target.dont_move() || target.dont_act() {
            total += 0.25;
        } else {
            total -= 0.906;
        }
    }

    // # 		0x02 - Cursed					0%(0000)
    // # 		0x01 - Treasure					-150% -c0(ff40)
    // # 	0x005a: Current Statuses 3
    // # 		0x80 - Oil						-5.5% -7(fff9)
    if target.oil() {
        total -= 0.055;
    }

    // # 		0x40 - Float					9.4% c(000c)
    if target.float() {
        total += 0.094;
    }

    // # 		0x20 - Reraise					39.8% 33(0033)
    if target.reraise() {
        total += 0.398;
    }

    // # 		0x10 - Transparent				29.7% 26(0026)
    if target.transparent() {
        total += 0.297;
    }

    // # 		0x08 - Berserk					-30.5% -27(ffd9)
    if target.berserk() {
        total -= 0.305;
    }

    // # 		0x04 - Chicken					-20.3% -1a(ffe6)
    if target.chicken() {
        total -= 0.203;
    }

    // # 		0x02 - Frog						-40.6% -34(ffcc)
    if target.frog() {
        total -= 0.406;
    }
    // # 		0x01 - Critical					-25% -20(ffe0)
    if target.critical() {
        total -= 0.25;
    }

    // # 	0x005b: Current Statuses 4
    // # 		0x80 - Poison					-20.3% -1a(ffe6)
    if target.poison() {
        total -= 0.203;
    }

    // # 		0x40 - Regen					19.5% 19(0019)
    if target.regen() {
        total += 0.195;
    }

    // # 		0x20 - Protect					19.5% 19(0019)
    if target.protect() {
        total += 0.195;
    }

    // # 		0x10 - Shell					19.5% 19(0019)
    if target.shell() {
        total += 0.195;
    }

    // # 		0x08 - Haste					14.8% 13(0013)
    if target.haste() {
        total += 0.148;
    }

    // # 		0x04 - Slow						-30.5% -27(ffd9) 0 if Confusion/Charm/Blood Suck
    if target.slow() && !(target.confusion() || target.charm() || target.blood_suck()) {
        total -= 0.305;
    }

    // # 		0x02 - Stop						-70.3% -5a(ffa6) 0 if Confusion/Charm/Blood Suck
    if target.stop() && !(target.confusion() || target.charm() || target.blood_suck()) {
        total -= 0.703;
    }

    // # 		0x01 - Wall						50% 40(0040)
    if target.wall() {
        total += 0.50;
    }


    // # 	0x005c: Current Statuses 5
    // # 		0x80 - Faith					4.7% 6(0006)
    if target.faith() {
        total += 0.047;
    }

    // # 		0x40 - Innocent					-5.5% -7(fff9)
    if target.innocent() {
        total -= 0.055;
    }

    // # 		0x20 - Charm					-50% -40(ffc0) (+1 / 4 if slow/stop/sleep/don't move/act/)
    if target.charm() {
        if target.slow() || target.stop() || target.sleep() || target.dont_move() || target.dont_act() {
            total += 0.25;
        } else {
            total -= 0.50;
        }
    }

    // # 		0x10 - Sleep					-30.5% -27(ffd9) 0 if Confusion/Charm/Blood Suck
    if target.sleep() && !(target.confusion() || target.charm() || target.blood_suck()) {
        total -= 0.305;
    }

    // # 		0x08 - Don't Move				-30.5% -27(ffd9) 0 if Confusion/Charm/Blood Suck
    if target.dont_move() && !(target.confusion() || target.charm() || target.blood_suck()) {
        total -= 0.305;
    }

    // # 		0x04 - Don't Act				-50% -40(ffc0) 0 if Confusion/Charm/Blood Suck
    if target.dont_act() && !(target.confusion() || target.charm() || target.blood_suck()) {
        total -= 0.50;
    }

    // # 		0x02 - Reflect					19.5% 19(0019)
    if target.reflect() {
        total += 0.195;
    }

    // # 		0x01 - Death Sentence			-80.5% -67(ff99)
    if target.death_sentence() {
        total -= 0.805;
    }

    total
}

fn ai_target_value_sum(user: &Combatant, combatants: &[Combatant]) -> f32 {
    combatants.iter()
        .map(|target| ai_calculate_target_value(user, target))
        .sum()
}

//
//     def ai_do_basic_turn(self, user: Combatant):
//         if user.dont_act:
//             self.move_out_of_combat(user)
//             return
//
//         targets = self.combatants
//         acting_cowardly = user.critical and self.ai_can_be_cowardly(user)
//         if acting_cowardly:
//             targets = [user]
//
//         actions = []
//         for target in targets:
//             actions.extend(cmd_item.consider_item(self, user, target))
//             actions.extend(cmd_attack.consider_attack(self, user, target))
//
//         self.ai_calculate_all_target_values(user)
//         basis = self.ai_target_value_sum()
//         considered_actions = []
//         for action in actions:
//             if not self.can_move_into_range(user, action.range, action.target):
//                 continue
//
//             simulated_world = copy.copy(self)
//             simulated_world.combatants = [copy.copy(combatant) for combatant in simulated_world.combatants]
//
//             simulated_world.log_report = False
//             simulated_world.trigger_reactions = False
//             simulated_user = simulated_world.combatants[action.user.index]
//             simulated_target = simulated_world.combatants[action.target.index]
//             action.perform(simulated_world, simulated_user, simulated_target)
//             simulated_world.ai_calculate_all_target_values(simulated_user)
//             new_value = simulated_world.ai_target_value_sum()
//             if new_value < basis:
//                 continue
//             considered_actions.append((new_value, action))
//
//         considered_actions.sort(key=lambda x: x[0], reverse=True)
//         for _, action in considered_actions:
//             if not self.in_range(user, action.range, action.target):
//                 self.move_to_range(user, action.range, action.target)
//
//             # TODO: This handles don't move, is there a better way?
//             if not self.in_range(user, action.range, action.target):
//                 continue
//
//             user.acted_during_active_turn = True
//             action.perform(self, action.user, action.target)
//             break
//
//         if user.moved_during_active_turn:
//             return
//
//         first_foe_in_action = None
//         for action in actions:
//             if user.is_foe(action.target):
//                 first_foe_in_action = action.target
//                 break
//         if first_foe_in_action:
//             self.move_towards_unit(user, first_foe_in_action)
//             return
//
//         self.move_out_of_combat(user)
//
//     def in_range(self, user: Combatant, range: int, target: Combatant):
//         dist = user.distance(target)
//         return dist <= range
//
//     def do_physical_evade(self, user: Combatant, weapon: Equipment, target: Combatant) -> bool:
//         if target.blade_grasp and not target.berserk and self.roll_brave_reaction(target):
//             self.unit_report(target, f'blade grasped {user.name}\'s attack')
//             return True
//
//         if target.arrow_guard and not target.berserk and weapon.weapon_type in (
//                 'Longbow', 'Bow', 'Gun', 'Crossbow') and self.roll_brave_reaction(target):
//             self.unit_report(target, f'arror guarded {user.name}\'s attack')
//             return True
//
//         if user.transparent or user.concentrate:
//             return False
//         # TODO: Arrow Guard, etc?
//         if random.random() < target.physical_accessory_evasion:
//             self.unit_report(target, f'guarded {user.name}\'s attack')
//             return True
//         if random.random() < target.physical_shield_evasion / 2.0:
//             self.unit_report(target, f'blocked {user.name}\'s attack')
//             return True
//         if random.random() < target.weapon_evasion / 2.0:
//             self.unit_report(target, f'parried {user.name}\'s attack')
//             return True
//         if random.random() < target.class_evasion / 2.0:
//             self.unit_report(target, f'evaded {user.name}\'s attack')
//             return True
//         return False
//
//     def add_status(self, target: Combatant, status: str, src: str):
//         if target.immune_to(status):
//             return
//
//         if status == DEATH:
//             self.target_died(target)
//             self.unit_report(target, f'was killed by {status} from {src}')
//             return
//
//         had_status = target.has_status(status)
//         target.add_status_flag(status)
//         if not had_status:
//             self.unit_report(target, f'now has {status} from {src}')
//
//         for cancelled in CANCELLING_STATUS.get(status, []):
//             self.cancel_status(target, cancelled, status)
//
//     def cancel_status(self, target: Combatant, status: str, src: Optional[str] = None):
//         if not target.has_status(status):
//             return
//         target.cancel_status(status)
//         if src:
//             self.unit_report(target, f'had {status} cancelled by {src}')
//         else:
//             self.unit_report(target, f'had {status} cancelled')
//
//     def weapon_chance_to_add_or_cancel_status(self, user: Combatant, weapon: Equipment, target: Combatant):
//         if not target.healthy:
//             return  # FIXME: this doesn't strictly make sense I don't think...
//
//         if not (weapon.chance_to_add or weapon.chance_to_cancel):
//             return
//
//         for status in weapon.chance_to_add:
//             if random.random() >= 0.19:
//                 continue
//             self.add_status(target, status, f'{user.name}\'s {weapon.weapon_name}')
//
//         for status in weapon.chance_to_cancel:
//             if random.random() >= 0.19:
//                 continue
//             self.cancel_status(target, status, f'{user.name}\'s {weapon.weapon_name}')
//
//     def change_target_hp(self, target: Combatant, amount, source: str):
//         if amount > 0:
//             if not target.healthy:
//                 return
//             if target.mana_shield and target.mp > 0 and self.roll_brave_reaction(target):
//                 self.change_target_mp(target, amount, source + ' (mana shield)')
//
//         target.hp = min(target.max_hp, max(0, target.hp - amount))
//         if amount >= 0:
//             self.unit_report(target, f'took {amount} damage from {source}')
//             target.took_damage_during_active_turn = True
//             for status in DAMAGE_CANCELS:
//                 self.cancel_status(target, status, source)
//         else:
//             self.unit_report(target, f'was healed for {abs(amount)} from {source}')
//         if target.hp == 0:
//             self.target_died(target)
//
//     def change_target_mp(self, target: Combatant, amount, source: str):
//         if not target.healthy:
//             return
//         target.mp = min(target.max_mp, max(0, target.mp - amount))
//         if amount >= 0 and source:
//             self.unit_report(target, f'took {amount} MP damage from {source}')
//         elif amount < 0 and source:
//             self.unit_report(target, f'recovered {abs(amount)} MP from {source}')
//
//     def after_damage_reaction(self, target: Combatant, inflicter: Combatant, amount: int):
//         if not self.trigger_reactions:
//             return
//
//         if amount == 0 or target.dead:
//             return
//
//         if target.auto_potion and self.roll_brave_reaction(target):
//             # FIXME: Need to consider UNDEAD
//             self.change_target_hp(target, -100, 'auto potion')
//             return
//
//         if target.damage_split and self.roll_brave_reaction(target):
//             self.change_target_hp(target, -(amount // 2), 'damage split')
//             self.change_target_hp(inflicter, amount // 2, 'damage split')
//             return
//
//     def target_died(self, target: Combatant):
//         target.hp = 0
//         self.unit_report(target, 'died')
//         for status in DEATH_CANCELS:
//             self.cancel_status(target, status, 'death')
//         target.crystal_counter = 4
//
//
// # IDEAS:
// #
// #  - Need to account for picking up crystals. I think this will go with expanding the
// #      where do I move to selection function? Because I will want to get out of AoEs I guess?
// #  - Pick up crystal Y/N could just happen after movement.
// #      will need a state for 'no longer exists at all?' can I just remove from combatants? do I want to?
// #  - I still don't entirely understand this target value thing, I should continue to read the docs
// #      if it is used to pick what skills to use, can I separate what is used in that calculation
// #      into a separate AI data block? That will require rewriting these skills and how they work, bluh.
// #  - Add 13% rule skip in the action consideration loop.
// #  - Can I keep statistics on how much different actions happen? Could be a useful part of testing.
// #  - Would be interesting to see if these true positives align with bird's true positives
// #  - If I run many simulations per match I could start calculating log loss as well.
// #  - At that point I should bring in multi-processing :)
// #
// #  - Need a hard fail & hard succeed rolling function for the sim-within-a-sim!
// #  - I'm guessing generally things like weapon adding status are assumed to hard fail
// #  - While your own abilities are a hard succeed
//
// def show_one():
//     tourny = fftbg.tournament.parse_tournament(Path('data/tournaments/1584818551017.json'))
//     patch = fftbg.patch.get_patch(tourny.modified)
//
//     for match_up in tourny.match_ups:
//         LOG.info(f'Starting match, {match_up.left.color} vs {match_up.right.color}')
//         combatants = []
//         for i, d in enumerate(match_up.left.combatants):
//             combatants.append(Combatant(d, patch, 0, i))
//         for i, d in enumerate(match_up.right.combatants):
//             combatants.append(Combatant(d, patch, 1, i + 4))
//         arena = fftbg.arena.get_arena(match_up.game_map)
//         sim = Simulation(combatants, arena, log_report=True)
//         sim.run()
//         if sim.left_wins:
//             LOG.info('Left team wins!')
//         else:
//             LOG.info('Right team wins!')
//
//
// def main():
//     import tqdm
//     fftbg.server.configure_logging('SIMULATION_LOG_LEVEL')
//
//     num_sims = 1
//     time_out_wins = 0
//     correct = 0
//     total = 0
//
//     for path in tqdm.tqdm(list(Path('data/tournaments').glob('*.json'))):
//         tourny = fftbg.tournament.parse_tournament(path)
//         patch = fftbg.patch.get_patch(tourny.modified)
//
//         for match_up in tourny.match_ups:
//             for _ in range(num_sims):
//                 combatants = []
//                 for i, d in enumerate(match_up.left.combatants):
//                     combatants.append(Combatant(d, patch, 0, i))
//                 for i, d in enumerate(match_up.right.combatants):
//                     combatants.append(Combatant(d, patch, 1, i + 4))
//                 arena = fftbg.arena.get_arena(match_up.game_map)
//                 sim = Simulation(combatants, arena, log_report=False)
//                 sim.run()
//
//                 if sim.left_wins and match_up.left_wins:
//                     correct += 1
//                     total += 1
//                 else:
//                     total += 1
//                 if sim.time_out_win:
//                     time_out_wins += 1
//
//     LOG.info(f'Total correct: {correct}/{total}')
//     LOG.info(f'Percent correct: {correct / total:.1%}')
//     LOG.info(f'Time outs: {time_out_wins}')
//
//
// if __name__ == '__main__':
//     main()
