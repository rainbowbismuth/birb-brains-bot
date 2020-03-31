use std::cell::RefCell;

use rand;
use rand::prelude::SmallRng;
use rand::Rng;

use crate::dto::rust::Equipment;
use crate::sim::{
    Action, ai_consider_actions, ai_target_value_sum, ALL_CONDITIONS, Combatant, COMBATANT_IDS,
    COMBATANT_IDS_LEN, CombatantId, Condition, DAMAGE_CANCELS, DEATH_CANCELS, EvasionType, Event, Location, Log, NO_SHORT_CHARGE,
    perform_action, Phase, SlowAction, Source, Team,
    TIMED_CONDITIONS, WeaponType,
};

pub const MAX_COMBATANTS: usize = COMBATANT_IDS_LEN;
const TIME_OUT_CT: usize = 1_000;

#[derive(Clone)]
pub struct Simulation<'a> {
    pub rng: RefCell<SmallRng>,
    pub combatants: [Combatant<'a>; MAX_COMBATANTS],
    pub actions: RefCell<Vec<Action<'a>>>,
    pub arena_length: i8,
    pub clock_tick: usize,
    pub prediction_mode: bool,
    pub log: Log<'a>,
    pub slow_actions: bool,
    pub active_turns: bool,
    pub left_wins: Option<bool>,
    pub time_out_win: Option<bool>,
}

impl<'a> Simulation<'a> {
    pub fn new(
        combatants: [Combatant<'a>; MAX_COMBATANTS],
        arena_length: i8,
        rng: SmallRng,
        event_log: bool,
    ) -> Simulation<'a> {
        let mut sim = Simulation {
            rng: RefCell::new(rng),
            combatants,
            actions: RefCell::new(vec![]),
            arena_length: arena_length / 2,
            clock_tick: 0,
            prediction_mode: false,
            log: if event_log {
                Log::new()
            } else {
                Log::new_no_log()
            },
            slow_actions: false,
            active_turns: false,
            left_wins: None,
            time_out_win: None,
        };
        for i in 0..4 {
            let combatant = &mut sim.combatants[i];
            combatant.location = Location::new(-arena_length as i16, combatant.location.y + (i*2) as i16 - 3);
        }
        for i in 0..4 {
            let combatant = &mut sim.combatants[i + 4];
            combatant.location = Location::new(arena_length as i16, combatant.location.y + (i*2) as i16 - 3);
        }
        sim
    }

    fn prediction_clone(&self) -> Simulation<'a> {
        Simulation {
            rng: self.rng.clone(),
            combatants: self.combatants.clone(),
            actions: RefCell::new(vec![]),
            arena_length: self.arena_length,
            clock_tick: self.clock_tick,
            prediction_mode: true,
            log: Log::new_no_log(),
            slow_actions: self.slow_actions,
            active_turns: self.active_turns,
            left_wins: self.left_wins,
            time_out_win: self.time_out_win,
        }
    }

    pub fn log_event(&self, event: Event<'a>) {
        self.log.add(&self.combatants, event);
    }

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
        self.combatants
            .iter()
            .filter(|combatant| combatant.team() == team)
            .any(|combatant| !combatant.dead() && !combatant.petrify() && !combatant.blood_suck())
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
        self.log.set_phase(Phase::StatusCheck);
        for cid in &COMBATANT_IDS {
            for condition in &TIMED_CONDITIONS {
                let removed = self.combatant_mut(*cid).tick_condition(*condition).unwrap();
                if removed {
                    self.log_event(Event::LostCondition(*cid, *condition, Source::Phase));
                }
            }
        }
    }

    pub fn phase_slow_action_charging(&mut self) {
        self.log.set_phase(Phase::SlowActionCharging);
        let mut slow_action_ready = false;
        for c_id in &COMBATANT_IDS {
            let combatant = self.combatant_mut(*c_id);
            if combatant.stop() {
                // FIXME: Does stop just remove the slow action? Sleep, etc...
                continue;
            }
            if let Some(slow_action) = combatant.ctr_action.as_mut() {
                if slow_action.ctr > 0 {
                    slow_action.ctr -= 1;
                }
                slow_action_ready = slow_action_ready || slow_action.ctr == 0;
            }
        }
        self.slow_actions = slow_action_ready;
    }

    pub fn phase_slow_action_resolve(&mut self) {
        for c_id in &COMBATANT_IDS {
            let combatant = self.combatant_mut(*c_id);
            if combatant.stop() {
                // FIXME: Does stop just remove the slow action? Sleep, etc...
                continue;
            }
            if let Some(slow_action) = combatant.ctr_action.clone() {
                if slow_action.ctr != 0 {
                    continue;
                }
                self.log.set_phase(Phase::SlowAction(*c_id));
                self.log_event(Event::UsingAbility(*c_id, slow_action.action));
                perform_action(self, *c_id, slow_action.action);
            }
            let combatant = self.combatant_mut(*c_id);
            combatant.ctr_action = None;
        }
    }

    pub fn phase_ct_charging(&mut self) {
        self.log.set_phase(Phase::CtCharging);
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
            combatant.ct += speed as u8;
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
                self.cancel_condition(*cid, Condition::Transparent, Source::Phase);
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
            if combatant.on_active_turn
                && !(combatant.moved_during_active_turn || combatant.acted_during_active_turn)
            {
                self.log_event(Event::DidNothing(*cid));
            }

            self.combatant_mut(*cid).on_active_turn = false;
        }
    }

    pub fn cancel_condition(
        &mut self,
        target_id: CombatantId,
        condition: Condition,
        src: Source<'a>,
    ) {
        let target = self.combatant_mut(target_id);
        if !target.has_condition(condition) {
            return;
        }
        target.cancel_condition(condition);
        self.log_event(Event::LostCondition(target_id, condition, src));
    }

    pub fn add_condition(&mut self, target_id: CombatantId, condition: Condition, src: Source<'a>) {
        if self.combatant(target_id).immune_to(condition) {
            return;
        }

        if condition == Condition::Death {
            self.target_died(target_id, src);
            return;
        }

        let target = self.combatant_mut(target_id);
        let had_status = target.has_condition(condition);
        target.add_condition(condition);
        if !had_status {
            self.log_event(Event::AddedCondition(target_id, condition, src));
        }

        for cancelled_condition in condition.cancels() {
            self.cancel_condition(
                target_id,
                *cancelled_condition,
                Source::Condition(condition),
            );
        }
    }

    pub fn phase_active_turn_resolve(&mut self) {
        for c_id in &COMBATANT_IDS {
            let combatant = self.combatant(*c_id);
            if combatant.ct < 100 {
                continue;
            }

            self.log.set_phase(Phase::ActiveTurn(*c_id));

            if combatant.petrify() || combatant.crystal() || combatant.stop() || combatant.sleep() {
                // TODO: What should really go here?
                self.combatant_mut(*c_id).ct = 0;
                continue;
            }

            if combatant.dead() && combatant.reraise() && !combatant.undead() {
                self.change_target_hp(
                    *c_id,
                    -(combatant.max_hp() / 10),
                    Source::Condition(Condition::Reraise),
                );
                self.cancel_condition(
                    *c_id,
                    Condition::Reraise,
                    Source::Condition(Condition::Reraise),
                );
            }

            let combatant = self.combatant(*c_id);
            if combatant.dead() && !combatant.crystal() {
                let now_crystal = self.combatant_mut(*c_id).tick_crystal_counter();
                let combatant = self.combatant(*c_id);

                if now_crystal && combatant.undead() && self.roll_auto_fail() < 0.5 {
                    let max_hp = combatant.max_hp();
                    self.combatant_mut(*c_id).reset_crystal_counter();
                    let heal_amount = self.roll_inclusive(1, max_hp);
                    self.change_target_hp(
                        *c_id,
                        -heal_amount,
                        Source::Condition(Condition::Undead),
                    );
                }

                let combatant = self.combatant(*c_id);
                if combatant.crystal() {
                    self.log_event(Event::BecameCrystal(*c_id));
                    continue;
                }
            }

            let combatant = self.combatant(*c_id);
            if combatant.death_sentence() {
                let is_undead = combatant.undead();
                let now_dead = self.combatant_mut(*c_id).tick_death_sentence_counter();
                if now_dead && is_undead {
                    self.cancel_condition(
                        *c_id,
                        Condition::DeathSentence,
                        Source::Condition(Condition::Undead),
                    );
                } else if now_dead {
                    self.target_died(*c_id, Source::Condition(Condition::DeathSentence));
                }
            }

            let combatant = self.combatant(*c_id);
            if combatant.dead() {
                continue;
            }

            self.clear_active_turn_flags();
            self.combatant_mut(*c_id).on_active_turn = true;

            let combatant = self.combatant(*c_id);
            if combatant.regen() {
                self.change_target_hp(
                    *c_id,
                    -(combatant.max_hp() / 8),
                    Source::Condition(Condition::Regen),
                );
            }

            self.ai_do_active_turn(*c_id);

            let combatant = self.combatant(*c_id);
            if combatant.poison() {
                // TODO: Can poison damage be mana shielded? *think*
                self.change_target_hp(
                    *c_id,
                    combatant.max_hp() / 8,
                    Source::Condition(Condition::Poison),
                );
            }

            self.end_of_active_turn_checks()
        }
    }

    pub fn roll_inclusive(&self, min: i16, max: i16) -> i16 {
        self.rng.borrow_mut().gen_range(min, max + 1)
    }

    pub fn roll_auto_succeed(&self) -> f32 {
        if self.prediction_mode {
            0.0
        } else {
            self.rng.borrow_mut().gen()
        }
    }

    pub fn roll_auto_fail(&self) -> f32 {
        if self.prediction_mode {
            1.0
        } else {
            self.rng.borrow_mut().gen()
        }
    }

    // NOTE: I flipped this bool to be true when in prediction mode
    fn roll_brave_reaction(&self, combatant: &Combatant) -> bool {
        if combatant.berserk() {
            false
        } else {
            self.roll_auto_fail() <= combatant.brave_percent()
        }
    }

    fn ai_thirteen_rule(&self) -> bool {
        let roll: f32 = self.rng.borrow_mut().gen();
        roll <= 0.137
    }

    fn ai_can_be_cowardly(&self, user: &Combatant) -> bool {
        let any_healthy = self
            .combatants
            .iter()
            .filter(|c| user.team() == c.team() && user.id() != c.id())
            .any(|c| c.healthy());
        let all_critical = self
            .combatants
            .iter()
            .filter(|c| user.team() == c.team() && user.id() != c.id())
            .all(|c| c.critical());
        any_healthy && !all_critical
    }

    fn do_move_with_bounds(&mut self, user_id: CombatantId, desired_location: Location) {
        let arena_length = self.arena_length;
        let user = self.combatant_mut(user_id);
        let old_location = user.location;
        let new_location = Location::new(
            (-arena_length as i16).max(desired_location.x.min(arena_length as i16)),
            (-arena_length as i16).max(desired_location.y.min(arena_length as i16)),
        );
        if old_location == new_location {
            return;
        }
        user.location = new_location;
        user.moved_during_active_turn = true;
        self.log_event(Event::Moved(user_id, old_location, new_location));

        let combatant = self.combatant(user_id);
        if combatant.moved_during_active_turn && combatant.move_hp_up() {
            self.change_target_hp(
                user_id,
                -(combatant.max_hp() / 10),
                Source::Constant("Move-HP Up"),
            );
        }
        let combatant = self.combatant(user_id);
        if combatant.moved_during_active_turn && combatant.move_mp_up() {
            self.change_target_mp(
                user_id,
                -(combatant.max_mp() / 10),
                Source::Constant("Move-MP Up"),
            );
        }
    }

    fn ai_do_active_turn(&mut self, user_id: CombatantId) {
        let user = self.combatant(user_id);
        if user.dont_act() {
            self.post_action_move(user_id);
            return;
        }

        if user.ctr_action.is_some() {
            // TODO: The AI in reality reconsiders here, will have to learn more.
            self.post_action_move(user_id);
            return;
        }

        let acting_cowardly = user.critical() && self.ai_can_be_cowardly(user);
        let targets = if acting_cowardly {
            &self.combatants[user_id.index()..user_id.index() + 1]
        } else {
            &self.combatants
        };

        let basis = {
            let mut actions = self.actions.borrow_mut();
            actions.clear();
            ai_consider_actions(&mut actions, self, user, targets);
            ai_target_value_sum(user, &self.combatants)
        };

        let best_action = {
            self.actions
                .borrow()
                .iter()
                .flat_map(|action| {
                    if self.ai_thirteen_rule() {
                        return None;
                    }
                    // TODO: This isn't strictly correct..
                    if !can_move_into_range(user, action.range, self.combatant(action.target_id)) {
                        return None;
                    }
                    let mut simulated_world = self.prediction_clone();
                    perform_action(&mut simulated_world, user_id, *action);
                    let new_value = ai_target_value_sum(
                        simulated_world.combatant(user_id),
                        &simulated_world.combatants,
                    );
                    if new_value < basis {
                        return None;
                    }

                    // FIXME: A hack to get around the whole partial ord thing
                    let ordered_val = (new_value * 1_000_000.0) as i64;
                    Some((ordered_val, *action))
                })
                .min_by_key(|pair| pair.0)
        };

        if let Some((_, action)) = best_action {
            let user = self.combatant(user_id);
            let target = self.combatant(action.target_id);
            if !in_range(user, action.range, target) {
                self.pre_action_move(user_id, action.range as u8, action.target_id);
            }
            if let Some(mut ctr) = action.ctr {
                let user = self.combatant(user_id);
                if user.short_charge() && action.ability.flags & NO_SHORT_CHARGE == 0 {
                    ctr /= 2;
                }
                self.combatant_mut(user_id).ctr_action = Some(SlowAction { ctr, action });
                self.log_event(Event::StartedCharging(user_id, action));
            } else {
                self.log_event(Event::UsingAbility(user_id, action));
                perform_action(self, user_id, action);
            }
            self.combatant_mut(user_id).acted_during_active_turn = true;
        }

        let user = self.combatant(user_id);
        if user.moved_during_active_turn {
            return;
        }

        if !user.acted_during_active_turn {
            self.engage_enemy_blindly(user_id);
            return;
        }

        self.post_action_move(user_id);
    }

    pub fn do_physical_evade(&self, user: &Combatant, target: &Combatant, src: Source<'a>) -> bool {
        if target.blade_grasp() && self.roll_brave_reaction(target) {
            self.log_event(Event::Evaded(target.id(), EvasionType::BladeGrasp, src));
            return true;
        }

        //         if target.arrow_guard and not target.berserk and weapon.weapon_type in (
        //                 'Longbow', 'Bow', 'Gun', 'Crossbow') and self.roll_brave_reaction(target):
        //             self.unit_report(target, f'arror guarded {user.name}\'s attack')
        //             return True

        if user.transparent() || user.concentrate() {
            return false;
        }

        if self.roll_auto_fail() < target.physical_accessory_evasion() {
            self.log_event(Event::Evaded(target.id(), EvasionType::Guarded, src));
            true
        } else if self.roll_auto_fail() < target.physical_shield_evasion() / 2.0 {
            self.log_event(Event::Evaded(target.id(), EvasionType::Blocked, src));
            true
        } else if self.roll_auto_fail() < target.weapon_evasion() / 2.0 {
            self.log_event(Event::Evaded(target.id(), EvasionType::Parried, src));
            true
        } else if self.roll_auto_fail() < target.class_evasion() / 2.0 {
            self.log_event(Event::Evaded(target.id(), EvasionType::Evaded, src));
            true
        } else {
            false
        }
    }

    pub fn do_magical_evade(&self, _user: &Combatant, target: &Combatant, src: Source<'a>) -> bool {
        if self.roll_auto_fail() < target.magical_accessory_evasion() {
            self.log_event(Event::Evaded(target.id(), EvasionType::Guarded, src));
            true
        } else if self.roll_auto_fail() < target.magical_shield_evasion() / 2.0 {
            self.log_event(Event::Evaded(target.id(), EvasionType::Blocked, src));
            true
        } else {
            false
        }
    }

    pub fn weapon_chance_to_add_or_cancel_status(
        &mut self,
        user_id: CombatantId,
        weapon: Option<&'a Equipment>,
        target_id: CombatantId,
    ) {
        let target = self.combatant(target_id);
        if !target.healthy() {
            return; // TODO: this doesn't strictly make sense I don't think...
        }

        let user = self.combatant(user_id);
        if user.sicken() {
            self.add_condition(target_id, Condition::Poison, Source::Constant("Sicken"));
        }

        if let Some(equip) = weapon {
            // FIXME: Out of all my flag refactoring, this is the part that sucks
            if equip.chance_to_cancel == 0 && equip.chance_to_cancel == 0 {
                return;
            }

            for condition in ALL_CONDITIONS.iter() {
                if equip.chance_to_add & condition.flag() != 0 {
                    if self.roll_auto_fail() < (1.0 - 0.19) {
                        continue;
                    }
                    self.add_condition(target_id, *condition, Source::Weapon(user_id, weapon));
                }
                if equip.chance_to_cancel & condition.flag() != 0 {
                    if self.roll_auto_fail() < (1.0 - 0.19) {
                        continue;
                    }
                    self.cancel_condition(target_id, *condition, Source::Weapon(user_id, weapon));
                }
            }
        }
    }

    pub fn change_target_hp(&mut self, target_id: CombatantId, amount: i16, src: Source<'a>) {
        let target = self.combatant(target_id);
        if amount > 0 {
            if !target.healthy() {
                return;
            }
            if target.mana_shield() && target.mp() > 0 && self.roll_brave_reaction(target) {
                self.change_target_mp(target_id, amount, src);
                // TODO: Is this considered damage of DAMAGE_CANCELS?
                return;
            }
        }
        let target = self.combatant_mut(target_id);
        target.set_hp_within_bounds(target.hp() - amount);
        let now_dead = target.dead();
        if amount > 0 {
            target.took_damage_during_active_turn = true;

            self.log_event(Event::HpDamage(target_id, amount, src));
            for condition in &DAMAGE_CANCELS {
                self.cancel_condition(target_id, *condition, src)
            }
        } else {
            self.log_event(Event::HpHeal(target_id, amount.abs(), src));
        }

        if now_dead {
            self.target_died(target_id, src);
        }
    }

    pub fn change_target_mp(&mut self, target_id: CombatantId, amount: i16, src: Source<'a>) {
        let target = self.combatant_mut(target_id);
        if target.dead() || target.petrify() || target.crystal() {
            return;
        }
        target.set_mp_within_bounds(target.mp() - amount);
        if amount >= 0 {
            self.log_event(Event::MpDamage(target_id, amount, src));
        } else {
            self.log_event(Event::MpHeal(target_id, amount, src));
        }
    }

    pub fn target_died(&mut self, target_id: CombatantId, src: Source<'a>) {
        let target = self.combatant_mut(target_id);
        target.set_hp_within_bounds(0);

        target.reset_crystal_counter();
        self.log_event(Event::Died(target_id, src));
        for condition in &DEATH_CANCELS {
            self.cancel_condition(target_id, *condition, Source::Condition(Condition::Death));
        }
    }

    // TODO: Make this private, rework the flow, etc etc
    pub fn after_damage_reaction(
        &mut self,
        user_id: CombatantId,
        target_id: CombatantId,
        amount: i16,
    ) {
        let target = self.combatant(target_id);
        if amount == 0 || target.dead() {
            return;
        }

        if target.auto_potion() && self.roll_brave_reaction(target) {
            let auto_potion_amount = if target.undead() { 100 } else { -100 };
            self.change_target_hp(
                target_id,
                auto_potion_amount,
                Source::Constant("Auto Potion"),
            );
            return;
        }

        if target.damage_split() && self.roll_brave_reaction(target) {
            self.change_target_hp(target_id, -(amount / 2), Source::Constant("Damage Split"));
            self.change_target_hp(user_id, amount / 2, Source::Constant("Damage Split"));
        }
    }

    pub fn calculate_weapon_xa(
        &self,
        user: &Combatant,
        weapon: Option<&'a Equipment>,
        k: i16,
    ) -> i16 {
        let weapon_type = weapon.and_then(|e| e.weapon_type);
        match weapon_type {
            None => ((user.pa() as i16 + k as i16) * user.raw_brave as i16) / 100,

            Some(WeaponType::Knife) | Some(WeaponType::NinjaSword) | Some(WeaponType::Bow) => {
                (user.pa() as i16 + k + user.speed() as i16 + k) / 2
            }

            Some(WeaponType::KnightSword) | Some(WeaponType::Katana) => {
                ((user.pa() as i16 + k) * user.raw_brave as i16) / 100
            }

            Some(WeaponType::Sword)
            | Some(WeaponType::Pole)
            | Some(WeaponType::Spear)
            | Some(WeaponType::Crossbow) => user.pa() as i16 + k,

            Some(WeaponType::Staff) => user.ma() as i16 + k,

            Some(WeaponType::Flail) | Some(WeaponType::Bag) => {
                self.roll_inclusive(1, (user.pa() as i16 + k).max(1))
            }

            Some(WeaponType::Cloth) | Some(WeaponType::Harp) | Some(WeaponType::Book) => {
                (user.pa() as i16 + k + user.ma() as i16 + k) / 2
            }

            // TODO: Magical guns
            Some(WeaponType::Gun) => weapon.unwrap().wp as i16 + k,
        }
    }

    fn pre_action_move(&mut self, user_id: CombatantId, range: u8, target_id: CombatantId) {
        let user = self.combatant(user_id);
        let target = self.combatant(target_id);
        let movement = if user.dont_move() { 0 } else { user.movement() };
        if in_range(user, range as i8, target) {
            return;
        }
        let best_panel = target
            .location
            .diamond(range)
            .flat_map(|location| {
                if user.location.distance(location) > movement as i16 {
                    return None;
                }
                if self.occupied_panel(location) {
                    return None;
                }
                let enemy_distance = self.enemy_distance_metric(user, location);
                Some((enemy_distance, location))
            })
            .max_by_key(|p| p.0)
            .map(|p| p.1);

        if let Some(panel) = best_panel {
            self.do_move_with_bounds(user_id, panel);
        }
    }

    fn post_action_move(&mut self, user_id: CombatantId) {
        let user = self.combatant(user_id);
        if user.dont_move() {
            return;
        }
        let best_panel = user
            .movement_diamond()
            .flat_map(|location| {
                if self.occupied_panel(location) {
                    return None;
                }
                let enemy_distance = self.enemy_distance_metric(user, location);
                // TODO: Add metric based on currently charging slow actions.
                Some((enemy_distance, location))
            })
            .max_by_key(|p| p.0)
            .map(|p| p.1);

        if let Some(panel) = best_panel {
            self.do_move_with_bounds(user_id, panel);
        }
    }

    fn engage_enemy_blindly(&mut self, user_id: CombatantId) {
        let user = self.combatant(user_id);
        if user.dont_move() {
            return;
        }
        let best_panel = user
            .movement_diamond()
            .flat_map(|location| {
                if self.occupied_panel(location) {
                    return None;
                }
                let enemy_distance = self.enemy_distance_metric(user, location);
                // TODO: Add metric based on currently charging slow actions.
                Some((enemy_distance, location))
            })
            .min_by_key(|p| p.0);

        if let Some((_, panel)) = best_panel {
            self.do_move_with_bounds(user_id, panel);
        }
    }

    fn occupied_panel(&self, location: Location) -> bool {
        // TODO: This doesn't belong in this function, but I'm not handling not being able to walk
        //  through enemy units, I.E. path finding.
        for combatant in &self.combatants {
            if combatant.location == location {
                return true;
            }
        }
        false
    }

    fn enemy_distance_metric(&self, user: &Combatant, location: Location) -> i16 {
        let mut metric = 0;
        for combatant in &self.combatants {
            if !user.foe(combatant) {
                continue;
            }

            metric += combatant.location.distance(location);
        }
        metric
    }

    pub fn combatant_on_panel(&self, location: Location) -> Option<CombatantId> {
        for combatant in &self.combatants {
            if combatant.location == location {
                return Some(combatant.id());
            }
        }
        None
    }
}

pub fn in_range(user: &Combatant, range: i8, target: &Combatant) -> bool {
    let dist = user.distance(target);
    dist <= range as i16
}

pub fn can_move_into_range(user: &Combatant, range: i8, target: &Combatant) -> bool {
    let movement = if user.dont_move() { 0 } else { user.movement() };
    user.distance(target) <= range as i16 + movement as i16
}

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
