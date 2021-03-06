use std::cell::RefCell;

use rand;
use rand::prelude::SmallRng;
use rand::Rng;

use crate::dto::rust::{Equipment, Tile};
use crate::sim::actions::attack::{attack_range, ATTACK_ABILITY};
use crate::sim::actions::basic_skill::DASH_ABILITY;

use crate::sim::{
    ai_consider_actions, ai_target_value_sum, perform_action, perform_action_slow, AbilityFlags,
    Action, ActionTarget, Arena, Combatant, CombatantId, Condition, EvasionType, Event, Location,
    Log, MovementInfo, Panel, Pathfinder, Phase, SlowAction, Source, Team, WeaponType, ALLY_OK,
    ALL_CONDITIONS, COMBATANT_IDS, COMBATANT_IDS_LEN, COMBATANT_IDS_TURN_RESOLVE, DAMAGE_CANCELS,
    DEATH_CANCELS, FOE_OK, NO_SHORT_CHARGE, TIMED_CONDITIONS,
};
use std::borrow::Borrow;

pub const MAX_COMBATANTS: usize = COMBATANT_IDS_LEN;
const TIME_OUT_CT: usize = 1_000;

#[derive(Clone)]
pub struct Simulation<'a> {
    pub rng: RefCell<SmallRng>,
    pub combatants: [Combatant<'a>; MAX_COMBATANTS],
    pub actions: RefCell<Vec<Action<'a>>>,
    pub arena: &'a Arena,
    pub pathfinder: &'a RefCell<Pathfinder<'a>>,
    pub clock_tick: usize,
    pub prediction_mode: bool,
    pub trigger_countergrasps: bool,
    pub log: Log<'a>,
    pub slow_actions: bool,
    pub active_turns: bool,
    pub left_wins: Option<bool>,
    pub time_out_win: Option<bool>,
}

impl<'a> Simulation<'a> {
    pub fn new(
        combatants: [Combatant<'a>; MAX_COMBATANTS],
        arena: &'a Arena,
        pathfinder: &'a RefCell<Pathfinder<'a>>,
        rng: SmallRng,
        event_log: bool,
    ) -> Simulation<'a> {
        let mut sim = Simulation {
            rng: RefCell::new(rng),
            combatants,
            actions: RefCell::new(vec![]),
            arena,
            pathfinder,
            clock_tick: 0,
            prediction_mode: false,
            trigger_countergrasps: true,
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
        sim.set_starting_locations();
        sim
    }

    fn set_starting_locations(&mut self) {
        for starting_location in &self.arena.starting_locations {
            let idx = if starting_location.left_team {
                starting_location.unit
            } else {
                starting_location.unit + 4
            };
            let mut combatant = self.combatant_mut(CombatantId::new(idx));
            combatant.panel = Panel::new(
                Location::new(starting_location.x as i16, starting_location.y as i16),
                starting_location.layer,
            );
            combatant.facing = starting_location.facing;
        }
    }

    fn prediction_clone(&self) -> Simulation<'a> {
        Simulation {
            rng: self.rng.clone(),
            combatants: self.combatants.clone(),
            actions: RefCell::new(vec![]),
            arena: self.arena,
            pathfinder: self.pathfinder,
            clock_tick: self.clock_tick,
            prediction_mode: true,
            trigger_countergrasps: true,
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
            .any(|combatant| combatant.healthy() && !combatant.blood_suck())
    }

    pub fn enemy_team_only_confused(&self, user: &Combatant) -> bool {
        self.combatants
            .iter()
            .filter(|target| user.foe(target) && target.healthy())
            .all(|target| target.confusion())
    }

    pub fn tick(&mut self) {
        self.phase_status_check();
        self.phase_slow_action_charging();
        if self.slow_actions {
            self.phase_slow_action_resolve(false);
        }
        self.phase_ct_charging();
        if self.active_turns {
            self.phase_active_turn_resolve();
        }
    }

    pub fn slow_actions_remaining(&self) -> bool {
        for combatant in &self.combatants {
            if combatant.stop() {
                continue;
            }
            if combatant.ctr_action.is_some() {
                return true;
            }
        }
        false
    }

    pub fn run_slow_actions(&mut self) {
        while self.slow_actions_remaining() {
            self.phase_slow_action_charging();
            if self.slow_actions {
                self.phase_slow_action_resolve(true);
            }
        }
    }

    pub fn phase_status_check(&mut self) {
        self.clock_tick += 1;
        self.log.set_clock_tick(self.clock_tick);
        self.log.set_phase(Phase::StatusCheck);
        for cid in &COMBATANT_IDS {
            let combatant = self.combatant(*cid);
            if combatant.jumping() {
                continue;
            }

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

    pub fn phase_slow_action_resolve(&mut self, slow_action_only_mode: bool) {
        for c_id in &COMBATANT_IDS_TURN_RESOLVE {
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
                perform_action_slow(self, *c_id, slow_action.action);

                let combatant = self.combatant_mut(*c_id);

                if slow_action_only_mode {
                    combatant.ctr_action = None;
                    continue;
                }

                if combatant.performing() {
                    let mut sa = combatant.ctr_action.as_mut().unwrap();
                    sa.ctr = sa.starting_ctr;
                } else {
                    combatant.ctr_action = None;
                }

                if !self.combatant(*c_id).monster() && !self.combatant(*c_id).mimic() {
                    self.do_mime_cycle(*c_id, slow_action.action);
                }
            }
        }
    }

    pub fn phase_ct_charging(&mut self) {
        self.log.set_phase(Phase::CtCharging);
        for combatant in &mut self.combatants {
            if combatant.stop() || combatant.sleep() || combatant.petrify() {
                continue;
            }

            // TODO: I'm not sure what is making these overflow :/
            let mut speed: u8 = combatant.speed();
            if combatant.haste() {
                speed = speed.saturating_mul(3) / 2;
            }
            if combatant.slow() {
                speed = speed.saturating_mul(2) / 3;
            }
            combatant.ct = combatant.ct.saturating_add(speed);
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
            combatant.damage_took_during_active_turn = None;
        }
    }

    pub fn end_of_action_checks(&mut self, user_id: CombatantId) {
        if self.prediction_mode {
            return;
        }
        for cid in &COMBATANT_IDS {
            let combatant = self.combatant(*cid);
            if let Some(amount) = combatant.damage_took_during_active_turn {
                self.after_damage_reaction(user_id, *cid, amount);
            }
        }
    }

    fn end_of_active_turn_checks(&mut self) {
        for cid in &COMBATANT_IDS {
            let combatant = self.combatant(*cid);
            if combatant.acted_during_active_turn
                || combatant.damage_took_during_active_turn.is_some()
            {
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
        if !target.has_condition(condition) {
            self.log_event(Event::LostCondition(target_id, condition, src));
        }
    }

    pub fn add_condition(&mut self, target_id: CombatantId, condition: Condition, src: Source<'a>) {
        let target = self.combatant(target_id);
        if !target.healthy() || target.immune_to(condition) {
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
        for c_id in &COMBATANT_IDS_TURN_RESOLVE {
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
                } else {
                    self.combatant_mut(*c_id).ct = 0;
                }

                let combatant = self.combatant(*c_id);
                if combatant.crystal() {
                    self.log_event(Event::BecameCrystal(*c_id));
                    continue;
                }
            }

            let combatant = self.combatant(*c_id);
            if combatant.defending() {
                self.cancel_condition(*c_id, Condition::Defending, Source::Phase);
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
            self.face_closest_enemy(*c_id);

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

    pub fn in_map(&self, panel: Panel) -> bool {
        self.arena.panel_to_index(panel).is_some()
    }

    pub fn tile(&self, panel: Panel) -> Tile {
        self.arena.tile(panel)
    }

    pub fn height(&self, panel: Panel) -> f32 {
        let tile = self.tile(panel);
        tile_height(&tile)
    }

    pub fn combatant_height(&self, combatant_id: CombatantId) -> f32 {
        let combatant = self.combatant(combatant_id);
        let panel = combatant.panel;
        let tile = self.tile(panel);
        combatant_height(&tile, combatant)
    }

    pub fn height_diff(&self, user_id: CombatantId, target_id: CombatantId) -> f32 {
        self.combatant_height(user_id) - self.combatant_height(target_id)
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

    fn combatant_submerged(&self, combatant: &Combatant) -> bool {
        let tile = self.tile(combatant.panel);
        combatant_submerged(&tile, combatant)
    }

    // NOTE: I flipped this bool to be true when in prediction mode
    fn roll_brave_reaction(&self, combatant: &Combatant) -> bool {
        if combatant.berserk() || combatant.confusion() || self.combatant_submerged(combatant) {
            false
        } else {
            self.roll_auto_fail() <= combatant.brave_percent()
        }
    }

    fn ai_thirteen_rule(&self) -> bool {
        let roll: f32 = self.rng.borrow_mut().gen();
        roll <= (0.137 * 5.0)
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

    fn check_crystal_pickup(&mut self, user_id: CombatantId, new_panel: Panel) {
        let mut found_crystal = None;
        for combatant in &self.combatants {
            if combatant.panel == new_panel && combatant.crystal() {
                found_crystal = Some(combatant.id());
                break;
            }
        }
        if let Some(crystal_id) = found_crystal {
            let crystal = self.combatant_mut(crystal_id);
            crystal.take_crystal();
            self.change_target_hp(user_id, -999, Source::Constant("picked up crystal"));
            self.change_target_mp(user_id, -999, Source::Constant("picked up crystal"));
        }
    }

    fn do_move_with_bounds(&mut self, user_id: CombatantId, desired_panel: Panel) {
        {
            let pathfinder = self.pathfinder.borrow();
            if !pathfinder.inside_map(desired_panel) {
                return;
            }
        }
        let user = self.combatant_mut(user_id);
        let old_location = user.panel;
        if old_location == desired_panel {
            return;
        }
        user.panel = desired_panel;
        user.moved_during_active_turn = true;
        self.log_event(Event::Moved(user_id, old_location, desired_panel));
        self.check_crystal_pickup(user_id, desired_panel);
        let combatant = self.combatant(user_id);
        if combatant.moved_during_active_turn && combatant.move_hp_up() && !combatant.confusion() {
            self.change_target_hp(
                user_id,
                -(combatant.max_hp() / 10),
                Source::Constant("Move-HP Up"),
            );
        }
        let combatant = self.combatant(user_id);
        if combatant.moved_during_active_turn && combatant.move_mp_up() && !combatant.confusion() {
            self.change_target_mp(
                user_id,
                -(combatant.max_mp() / 10),
                Source::Constant("Move-MP Up"),
            );
        }

        let mut combatant = self.combatant_mut(user_id);
        if combatant.dont_move_while_charging() {
            combatant.ctr_action = None;
        }
    }

    pub fn ai_foes_have_non_disabled_units(&self, user: &Combatant) -> bool {
        self.combatants
            .iter()
            .filter(|target| user.foe(target) && target.healthy())
            .any(|target| !target.confusion() && !target.death_sentence())
    }

    fn ai_cancel_charge_check(&mut self, user_id: CombatantId) -> bool {
        let user = self.combatant(user_id);
        if !user.ctr_action.is_some() {
            return false;
        }
        if !user.performing() {
            return true;
        }
        if self.roll_inclusive(0, 1) == 1 {
            return true;
        }
        let mut user = self.combatant_mut(user_id);
        user.ctr_action = None;
        return false;
    }

    fn ai_do_active_turn(&mut self, user_id: CombatantId) {
        let user = self.combatant(user_id);
        if user.dont_act() {
            self.post_action_move(user_id);
            return;
        }

        if self.ai_cancel_charge_check(user_id) {
            // TODO: The AI in reality reconsiders here, will have to learn more.
            self.post_action_move(user_id);
            return;
        }

        let user = self.combatant(user_id);
        let acting_cowardly = user.critical() && self.ai_can_be_cowardly(user);
        let targets = if acting_cowardly {
            &self.combatants[user_id.index()..user_id.index() + 1]
        } else {
            &self.combatants
        };

        let ignore_confusion = self.enemy_team_only_confused(user);
        let basis = {
            let mut cloned = self.prediction_clone();
            cloned.run_slow_actions();
            ai_target_value_sum(user, &cloned.combatants, ignore_confusion)
        };

        {
            let mut actions = self.actions.borrow_mut();
            actions.clear();
            ai_consider_actions(&mut actions, self, user, targets);
        }

        let best_action = self.ai_choose_best_action(user_id, basis, ignore_confusion);
        let mut targeted_self = false;
        if let Some(action) = best_action {
            let user = self.combatant(user_id);

            if let Some(target_id) = action.target.to_target_id(self) {
                targeted_self = user_id == target_id;
            }

            if let Some(target_panel) = action.target.to_panel(self) {
                if !in_range_panel(user, &action, target_panel) {
                    self.pre_action_move(user_id, &action, target_panel);
                }
            }

            if let Some(mut ctr) = action.ctr {
                let user = self.combatant(user_id);
                if user.short_charge() && action.ability.flags & NO_SHORT_CHARGE == 0 {
                    ctr /= 2;
                }
                let mut user = self.combatant_mut(user_id);
                user.ctr_action = Some(SlowAction {
                    ctr,
                    starting_ctr: ctr,
                    action,
                });
                self.log_event(Event::StartedCharging(user_id, action));
            } else {
                if let Some(target_panel) = action.target.to_panel(self) {
                    let mut user = self.combatant_mut(user_id);
                    user.facing = user.panel.facing_towards(target_panel);
                }

                self.log_event(Event::UsingAbility(user_id, action));
                perform_action(self, user_id, action);

                if !self.combatant(user_id).monster() && !self.combatant(user_id).mimic() {
                    self.do_mime_cycle(user_id, action);
                }
            }
            self.combatant_mut(user_id).acted_during_active_turn = true;
        }

        let user = self.combatant(user_id);
        if !user.healthy() || user.jumping() {
            return;
        }

        if user.moved_during_active_turn {
            return;
        }

        if (targeted_self && !acting_cowardly) || !user.acted_during_active_turn {
            self.engage_enemy_blindly(user_id);
            return;
        }

        self.post_action_move(user_id);
    }

    pub fn ai_choose_confused_action(&mut self, user_id: CombatantId) -> Option<Action<'a>> {
        let user = self.combatant(user_id);
        self.actions
            .borrow()
            .iter()
            .flat_map(|action| {
                let mut dist_penalty = 0;
                if let Some(target_panel) = action.target.to_panel(self) {
                    dist_penalty = user.panel.distance(target_panel) as i64;
                }
                let random_val = self.roll_inclusive(1, 10_000) as i64;
                let ordered_val = (random_val + dist_penalty * 10_000) as i64;
                Some((ordered_val, *action))
            })
            .min_by_key(|pair| pair.0)
            .map(|pair| pair.1)
    }

    pub fn ai_choose_best_action(
        &mut self,
        user_id: CombatantId,
        basis: f32,
        ignore_confusion: bool,
    ) -> Option<Action<'a>> {
        let user = self.combatant(user_id);
        if user.confusion() {
            return self.ai_choose_confused_action(user_id);
        }

        self.actions
            .borrow()
            .iter()
            .flat_map(|action| {
                if self.ai_thirteen_rule() {
                    return None;
                }
                // TODO: This isn't strictly correct..
                if let Some(target_id) = action.target.to_target_id(self) {
                    if !can_move_into_range(user, action, self.combatant(target_id)) {
                        return None;
                    }
                }

                let mut simulated_world = self.prediction_clone();
                let user = self.combatant(user_id);
                if let Some(target_panel) = action.target.to_panel(self) {
                    if !in_range_panel(user, action, target_panel) {
                        simulated_world.pre_action_move(user_id, action, target_panel);
                    }
                    let sim_user = simulated_world.combatant(user_id);
                    if !in_range_panel(sim_user, action, target_panel) {
                        return None;
                    }
                }
                perform_action(&mut simulated_world, user_id, *action);
                simulated_world.run_slow_actions();
                let new_value = ai_target_value_sum(
                    simulated_world.combatant(user_id),
                    &simulated_world.combatants,
                    ignore_confusion,
                );
                if new_value <= basis {
                    return None;
                }

                // FIXME: A hack to get around the whole partial ord thing
                let ordered_val = (new_value * 1_000_000.0) as i64;
                Some((ordered_val, *action))
            })
            .max_by_key(|pair| pair.0)
            .map(|pair| pair.1)
    }

    pub fn do_mime_cycle(&mut self, user_id: CombatantId, action: Action<'a>) {
        for c_id in &COMBATANT_IDS {
            let user = self.combatant(user_id);
            let possible_mime = self.combatant(*c_id);

            if possible_mime.mimic() && user.ally(possible_mime) && user_id != *c_id {
                if !possible_mime.healthy()
                    || possible_mime.sleep()
                    || possible_mime.confusion()
                    || possible_mime.berserk()
                    || possible_mime.dont_act()
                    || possible_mime.blood_suck()
                    || possible_mime.sleep()
                    || possible_mime.frog()
                    || possible_mime.chicken()
                    || possible_mime.stop()
                {
                    continue;
                }

                for combatant in &mut self.combatants {
                    combatant.damage_took_during_active_turn = None;
                }
                let user = self.combatant(user_id);
                let possible_mime = self.combatant(*c_id);

                if let Some(original_target_panel) = action.target.to_panel(self) {
                    let original_vec = user.panel.location() - original_target_panel.location();
                    let original_facing = user.panel.facing_towards(original_target_panel);
                    let rotations = original_facing.rotations_to(possible_mime.facing);
                    let vec_on_mime = possible_mime.panel.location() + original_vec;
                    let new_target_loc =
                        vec_on_mime.rotate_around(possible_mime.panel.location(), rotations);
                    let mut new_action = action;
                    // TODO: Fix this, be smarter about layer selection...
                    new_action.target =
                        ActionTarget::Panel(original_target_panel.on_same_layer(new_target_loc));
                    self.log_event(Event::UsingAbility(*c_id, new_action));
                    perform_action(self, *c_id, new_action);
                } else {
                    // If we are using math, then execute the same action.
                    self.log_event(Event::UsingAbility(*c_id, action));
                    perform_action(self, *c_id, action);
                }
            }
        }
    }

    pub fn do_physical_evade(
        &self,
        user: &Combatant,
        target: &Combatant,
        weapon_type: Option<WeaponType>,
        src: Source<'a>,
    ) -> bool {
        if target.blade_grasp() && self.roll_brave_reaction(target) {
            self.log_event(Event::Evaded(target.id(), EvasionType::BladeGrasp, src));
            return true;
        }

        if target.arrow_guard() && self.roll_brave_reaction(target) {
            if weapon_type == Some(WeaponType::Bow)
                || weapon_type == Some(WeaponType::Gun)
                || weapon_type == Some(WeaponType::Crossbow)
            {
                self.log_event(Event::Evaded(target.id(), EvasionType::ArrowGuard, src));
                return true;
            }
        }

        if user.transparent() || user.concentrate() {
            return false;
        }

        let bonus = if user.confusion() { 2.0 } else { 1.0 };
        let facing = user.relative_facing(target);
        let attacker_blind = user.darkness();
        if self.roll_auto_fail() < target.physical_accessory_evasion(attacker_blind) * bonus {
            self.log_event(Event::Evaded(target.id(), EvasionType::Guarded, src));
            true
        } else if facing.is_front_or_side()
            && self.roll_auto_fail() < target.physical_shield_evasion(attacker_blind) * bonus
        {
            self.log_event(Event::Evaded(target.id(), EvasionType::Blocked, src));
            true
        } else if facing.is_front_or_side()
            && self.roll_auto_fail() < target.weapon_evasion(attacker_blind) * bonus
        {
            self.log_event(Event::Evaded(target.id(), EvasionType::Parried, src));
            true
        } else if facing.is_front()
            && self.roll_auto_fail() < target.class_evasion(attacker_blind) * bonus
        {
            self.log_event(Event::Evaded(target.id(), EvasionType::Evaded, src));
            true
        } else {
            false
        }
    }

    pub fn do_magical_evade(&self, user: &Combatant, target: &Combatant, src: Source<'a>) -> bool {
        let bonus = if user.confusion() { 2.0 } else { 1.0 };
        let facing = user.relative_facing(target);

        if self.roll_auto_fail() < target.magical_accessory_evasion() * bonus {
            self.log_event(Event::Evaded(target.id(), EvasionType::Guarded, src));
            true
        } else if facing.is_front_or_side()
            && self.roll_auto_fail() < target.magical_shield_evasion() * bonus
        {
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
        let was_critical = target.critical();
        target.set_hp_within_bounds(target.hp() - amount);
        let now_dead = target.dead();
        let now_critical = target.critical();
        if amount > 0 {
            // TODO: Technically amount == 0 would do this, but, that would require me to distinguish
            //   between damage and healing.
            target.damage_took_during_active_turn = Some(amount);

            self.log_event(Event::HpDamage(target_id, amount, src));
            for condition in &DAMAGE_CANCELS {
                self.cancel_condition(target_id, *condition, src)
            }

            if !was_critical && now_critical {
                self.became_critical_reaction(target_id);
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

    pub fn try_hamedo(&mut self, user_id: CombatantId, target_id: CombatantId) -> bool {
        if !self.trigger_countergrasps || self.prediction_mode {
            return false;
        }

        let target = self.combatant(target_id);
        if target.hamedo() && self.roll_brave_reaction(target) {
            let user = self.combatant(user_id);
            let range = attack_range(self, target, user);

            if !in_range(target, range, user) {
                return false;
            }

            let action = Action::new(&ATTACK_ABILITY, range, None, user_id);
            self.trigger_countergrasps = false;
            perform_action(self, target_id, action);
            self.trigger_countergrasps = true;
            return true;
        }
        false
    }

    pub fn try_countergrasp(&mut self, user_id: CombatantId, target_id: CombatantId) {
        if !self.trigger_countergrasps || self.prediction_mode {
            return;
        }
        let target = self.combatant(target_id);
        if target.dead() || target.sleep() || target.petrify() {
            return;
        }

        if target.dragon_spirit() && self.roll_brave_reaction(target) {
            self.add_condition(
                target_id,
                Condition::Reraise,
                Source::Constant("Dragon Spirit"),
            );
            return;
        }

        if target.meatbone_slash() && target.critical() && self.roll_brave_reaction(target) {
            let user = self.combatant(user_id);
            let range = attack_range(self, target, user);

            if !in_range(target, range, user) {
                return;
            }

            let amount = target.max_hp();
            self.change_target_hp(user_id, amount, Source::Ability);
            return;
        }

        if target.counter() && self.roll_brave_reaction(target) {
            let user = self.combatant(user_id);
            let range = attack_range(self, target, user);

            if !in_range(target, range, user) {
                return;
            }

            let action = Action::new(&ATTACK_ABILITY, range, None, user_id);
            self.trigger_countergrasps = false;
            perform_action(self, target_id, action);
            self.trigger_countergrasps = true;
            return;
        }

        if target.counter_tackle() && self.roll_brave_reaction(target) {
            let user = self.combatant(user_id);
            if !in_range(target, 1, user) {
                return;
            }

            let action = Action::new(&DASH_ABILITY, 1, None, user_id);
            self.trigger_countergrasps = false;
            perform_action(self, target_id, action);
            self.trigger_countergrasps = true;
        }
    }

    pub fn became_critical_reaction(&mut self, target_id: CombatantId) {
        let target = self.combatant(target_id);
        if target.hp_restore() && self.roll_brave_reaction(target) {
            let amount = -target.max_hp();
            self.change_target_hp(target_id, amount, Source::Constant("HP Restore"));
            return;
        }

        if target.mp_restore() && self.roll_brave_reaction(target) {
            let amount = -target.max_hp();
            self.change_target_mp(target_id, amount, Source::Constant("MP Restore"));
            return;
        }

        if target.critical_quick() && self.roll_brave_reaction(target) {
            let mut target = self.combatant_mut(target_id);
            // TODO: There's the whole quick flag thing...
            target.ct = target.ct.max(100);
            self.log_event(Event::CriticalQuick(target_id));
        }
    }

    fn after_damage_reaction(&mut self, user_id: CombatantId, target_id: CombatantId, amount: i16) {
        if user_id == target_id {
            return;
        }

        let target = self.combatant(target_id);
        if amount == 0 || target.dead() || target.sleep() || target.petrify() {
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

        if target.caution() && self.roll_brave_reaction(target) {
            self.add_condition(target_id, Condition::Defending, Source::Constant("Caution"));
            return;
        }

        if target.damage_split() && self.roll_brave_reaction(target) {
            self.change_target_hp(target_id, -(amount / 2), Source::Constant("Damage Split"));
            self.change_target_hp(user_id, amount / 2, Source::Constant("Damage Split"));
            return;
        }

        if target.regenerator() && self.roll_brave_reaction(target) {
            self.add_condition(target_id, Condition::Regen, Source::Constant("Regenerator"));
            return;
        }

        if target.pa_save() && self.roll_brave_reaction(target) {
            self.change_unit_pa(target_id, 1, Source::Constant("PA Save"));
            return;
        }

        if target.ma_save() && self.roll_brave_reaction(target) {
            self.change_unit_ma(target_id, 1, Source::Constant("MA Save"));
            return;
        }

        if target.speed_save() && self.roll_brave_reaction(target) {
            self.change_unit_speed(target_id, 1, Source::Constant("Speed Save"));
            return;
        }
    }

    pub fn change_unit_pa(&mut self, target_id: CombatantId, amount: i8, src: Source<'a>) {
        if amount == 0 {
            return;
        }
        let target = self.combatant_mut(target_id);
        target.pa_mod = (target.pa_mod + amount).min(10).max(-10);
        self.log_event(Event::PhysicalAttackBuff(target_id, amount, src));
    }

    pub fn change_unit_ma(&mut self, target_id: CombatantId, amount: i8, src: Source<'a>) {
        if amount == 0 {
            return;
        }
        let target = self.combatant_mut(target_id);
        target.ma_mod = (target.ma_mod + amount).min(10).max(-10);
        self.log_event(Event::MagicalAttackBuff(target_id, amount, src));
    }

    pub fn change_unit_speed(&mut self, target_id: CombatantId, amount: i8, src: Source<'a>) {
        if amount == 0 {
            return;
        }
        let target = self.combatant_mut(target_id);
        target.speed_mod = (target.speed_mod + amount).min(10).max(-10);
        self.log_event(Event::SpeedBuff(target_id, amount, src));
    }

    pub fn change_unit_brave(&mut self, target_id: CombatantId, amount: i8, src: Source<'a>) {
        if amount == 0 {
            return;
        }
        let target = self.combatant_mut(target_id);
        target.raw_brave = (target.raw_brave + amount).min(100).max(1);
        self.log_event(Event::BraveBuff(target_id, amount, src));
    }

    pub fn change_unit_faith(&mut self, target_id: CombatantId, amount: i8, src: Source<'a>) {
        if amount == 0 {
            return;
        }
        let target = self.combatant_mut(target_id);
        target.raw_faith = (target.raw_faith + amount).min(100).max(1);
        self.log_event(Event::FaithBuff(target_id, amount, src));
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

    fn pre_action_move(&mut self, user_id: CombatantId, action: &Action, target_panel: Panel) {
        let user = self.combatant(user_id);
        if in_range_panel(user, action, target_panel) {
            return;
        }

        let best_panel = {
            self.mark_enemy_occupied_panels(user);
            let movement_info = MovementInfo::new(user);
            let mut pathfinder = self.pathfinder.borrow_mut();
            pathfinder.calculate_reachable_no_reset(&movement_info, user.panel);

            target_panel
                .diamond(action.range as u8)
                .flat_map(|panel| {
                    if action.ability.aoe.is_line() && !panel.lined_up(target_panel) {
                        return None;
                    }
                    if !pathfinder.can_reach_and_end_turn_on(panel) {
                        return None;
                    }
                    let tile = self.tile(panel);
                    if combatant_submerged(&tile, user) {
                        return None;
                    }
                    let enemy_distance = self.enemy_distance_metric(user, panel.location());
                    let crystal = self.crystal_metric(panel);
                    let avoid_aoe = -self.avoid_aoe_metric(panel);
                    Some((enemy_distance + crystal + avoid_aoe, panel))
                })
                .max_by_key(|p| p.0)
                .map(|p| p.1)
        };

        if let Some(panel) = best_panel {
            self.do_move_with_bounds(user_id, panel);
        }
    }

    fn post_action_move(&mut self, user_id: CombatantId) {
        let user = self.combatant(user_id);
        if user.dont_move() || user.dont_move_while_charging() {
            return;
        }
        let best_panel = {
            self.mark_enemy_occupied_panels(user);
            let movement_info = MovementInfo::new(user);
            let mut pathfinder = self.pathfinder.borrow_mut();
            pathfinder.calculate_reachable_no_reset(&movement_info, user.panel);

            pathfinder
                .reachable_set()
                .iter()
                .map(|panel| {
                    let enemy_distance = self.enemy_distance_metric(user, panel.location());
                    let crystal = self.crystal_metric(*panel);
                    let avoid_aoe = -self.avoid_aoe_metric(*panel);
                    // TODO: Add metric based on currently charging slow actions.
                    (enemy_distance + crystal + avoid_aoe, *panel)
                })
                .max_by_key(|p| p.0)
                .map(|p| p.1)
        };

        if let Some(panel) = best_panel {
            self.do_move_with_bounds(user_id, panel);
        }
    }

    fn closest_healthy_enemy_panel(&self, user_id: CombatantId) -> Option<Panel> {
        let user = self.combatant(user_id);
        self.combatants
            .iter()
            .flat_map(|target| {
                if !user.foe(target) || !target.healthy() {
                    return None;
                }
                Some((user.distance(target), target.panel))
            })
            .min_by_key(|p| p.0)
            .map(|p| p.1)
    }

    fn face_closest_enemy(&mut self, user_id: CombatantId) {
        let closest_enemy_panel = self.closest_healthy_enemy_panel(user_id);
        if let Some(target_panel) = closest_enemy_panel {
            let mut user = self.combatant_mut(user_id);
            user.facing = user.panel.facing_towards(target_panel);
        }
    }

    fn engage_enemy_blindly(&mut self, user_id: CombatantId) {
        let user = self.combatant(user_id);
        if user.dont_move() {
            return;
        }
        let closest_enemy_panel = self
            .closest_healthy_enemy_panel(user_id)
            .unwrap_or_else(|| Panel::coords(self.arena.width / 2, self.arena.height / 2, false));

        let best_panel = {
            self.mark_enemy_occupied_panels(user);
            let movement_info = MovementInfo::new(user);
            let mut pathfinder = self.pathfinder.borrow_mut();
            pathfinder.calculate_reachable_no_reset(&movement_info, user.panel);
            pathfinder
                .reachable_set()
                .iter()
                .map(|panel| {
                    let enemy_distance = self.enemy_distance_metric(user, panel.location());
                    let crystal = -self.crystal_metric(*panel);
                    let avoid_aoe = self.avoid_aoe_metric(*panel);
                    // TODO: Add metric based on currently charging slow actions.
                    (enemy_distance + crystal + avoid_aoe, *panel)
                })
                .min_by_key(|p| p.0)
                .map(|p| p.1)
                .unwrap()
        };

        if user.panel != best_panel {
            self.do_move_with_bounds(user_id, best_panel);
        } else {
            // we got stuck.
            let next_panel = {
                self.mark_enemy_occupied_panels(user);
                let movement_info = MovementInfo::new(user);
                let mut pathfinder = self.pathfinder.borrow_mut();
                pathfinder.path_find_no_reset(&movement_info, user.panel, closest_enemy_panel)
            };
            self.do_move_with_bounds(user_id, next_panel);
        }
    }

    fn mark_enemy_occupied_panels(&self, user: &Combatant) {
        let mut pathfinder = self.pathfinder.borrow_mut();
        pathfinder.reset_all();
        for combatant in &self.combatants {
            if combatant.crystal() {
                continue;
            }
            pathfinder.set_occupied(combatant.panel);
            if user.ally(combatant) || combatant.dead() {
                continue;
            }
            pathfinder.set_impassable(combatant.panel);
        }
    }

    fn enemy_distance_metric(&self, user: &Combatant, location: Location) -> i16 {
        let mut metric = 0;
        for combatant in &self.combatants {
            if !user.foe(combatant) {
                continue;
            }
            if !combatant.healthy() {
                continue;
            }

            metric += combatant.panel.location().distance(location);
        }

        metric
    }

    fn crystal_metric(&self, panel: Panel) -> i16 {
        for combatant in &self.combatants {
            if combatant.crystal() && combatant.panel == panel {
                return 200;
            }
        }
        return 0;
    }

    fn avoid_aoe_metric(&self, panel: Panel) -> i16 {
        let mut metric = 0;
        for combatant in &self.combatants {
            if let Some(slow_action) = combatant.ctr_action {
                if let Some(target_panel) = slow_action.action.target.to_panel(self) {
                    if slow_action.action.ability.aoe.inside(target_panel, panel) {
                        metric += 5;
                    }
                }
            }
        }
        metric
    }

    pub fn combatant_on_panel(&self, panel: Panel) -> Option<CombatantId> {
        for combatant in &self.combatants {
            if combatant.panel == panel {
                return Some(combatant.id());
            }
        }
        None
    }

    pub fn do_knockback(&mut self, user_id: CombatantId, target_id: CombatantId) {
        // TODO: I just realized you could knock someone up the map lol.
        let user = self.combatant(user_id);
        let target = self.combatant(target_id);
        let direction = user.panel.facing_towards(target.panel).offset();
        let new_panel = target.panel.plus(direction);
        {
            let pathfinder = self.pathfinder.borrow();
            if !pathfinder.inside_map(new_panel) {
                return;
            }
        }
        if self.combatant_on_panel(new_panel).is_none() {
            let mut target = self.combatant_mut(target_id);
            target.panel = new_panel;
            self.log_event(Event::Knockback(target_id, new_panel));
            let mut target = self.combatant_mut(target_id);
            if target.dont_move_while_charging() {
                target.ctr_action = None;
            }
        }
    }
}

pub fn in_range(user: &Combatant, range: u8, target: &Combatant) -> bool {
    let dist = user.distance(target);
    dist <= range as i16
}

pub fn in_range_panel(user: &Combatant, action: &Action, panel: Panel) -> bool {
    if action.ability.aoe.is_line() {
        user.panel.lined_up(panel) && user.panel.distance(panel) <= action.range as i16
    } else {
        let dist = user.panel.distance(panel);
        dist <= action.range as i16
    }
}

pub fn can_move_into_range(user: &Combatant, action: &Action, target: &Combatant) -> bool {
    if action.ability.aoe.is_line() {
        return can_move_into_range_line(user, action, target);
    }

    let movement = if user.dont_move() { 0 } else { user.movement() };
    user.distance(target) <= action.range as i16 + movement as i16
}

pub fn can_move_into_range_line(user: &Combatant, action: &Action, target: &Combatant) -> bool {
    let movement = if user.dont_move() { 0 } else { user.movement() } as i16;
    let user_loc = user.panel.location();
    let target_loc = target.panel.location();
    let x_diff = (user_loc.x - target_loc.x).abs();
    let y_diff = (user_loc.y - target_loc.y).abs();
    let min_diff = x_diff.min(y_diff);
    let max_diff = x_diff.max(y_diff);
    // TODO: Revisit, I'm nearly certain this isn't correct though it should be... mostly ok.
    movement >= min_diff && movement + action.range as i16 >= min_diff + max_diff
}

//
//
// # IDEAS:
// #
// #  - Need to account for picking up crystals. I think this will go with expanding the
// #      where do I move to selection function? Because I will want to get out of AoEs I guess?
// #  - Pick up crystal Y/N could just happen after movement.
// #      will need a state for 'no longer exists at all?' can I just remove from combatants? do I want to?
// #  - Can I keep statistics on how much different actions happen? Could be a useful part of testing.
// #  - Would be interesting to see if these true positives align with bird's true positives

pub fn combatant_height(tile: &Tile, combatant: &Combatant) -> f32 {
    let tile_height = tile_height(tile);
    let float_bonus = if combatant.float() {
        0.5 + tile.depth as f32
    } else {
        0.0
    };
    tile_height + float_bonus
}

pub fn combatant_submerged(tile: &Tile, combatant: &Combatant) -> bool {
    if tile.depth >= 2 {
        return !combatant.float();
    }
    false
}

pub fn tile_height(tile: &Tile) -> f32 {
    tile.height as f32 + tile.slope_height as f32 / 2.0
}
