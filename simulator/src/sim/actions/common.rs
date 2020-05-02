use crate::sim::actions::{Ability, AbilityImpl, Action};
use crate::sim::{Combatant, CombatantId, Condition, Element, Event, Simulation, Source};

pub fn should_heal_foe(target: &Combatant, hurts_undead: bool) -> bool {
    hurts_undead && target.undead()
}

pub fn should_heal_ally(target: &Combatant, hurts_undead: bool) -> bool {
    if hurts_undead && target.undead() {
        false
    } else {
        target.hp_percent() <= 0.50
    }
}

pub fn do_hp_damage(
    sim: &mut Simulation,
    target_id: CombatantId,
    mut amount: i16,
    heals_undead: bool,
) {
    let target = sim.combatant(target_id);
    if heals_undead && target.undead() {
        amount = -amount;
    }
    sim.change_target_hp(target_id, amount, Source::Ability);
}

pub fn do_hp_heal(
    sim: &mut Simulation,
    target_id: CombatantId,
    mut amount: i16,
    hurts_undead: bool,
) {
    let target = sim.combatant(target_id);
    if hurts_undead && target.undead() {
        amount = -amount;
    }
    sim.change_target_hp(target_id, -amount, Source::Ability);
}

pub struct AddConditionSpellImpl {
    pub condition: Condition,
    pub can_be_evaded: bool,
    pub ignore_magic_def: bool,
    pub base_chance: i16,
    pub range: u8,
    pub ctr: u8,
}

impl AbilityImpl for AddConditionSpellImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        // TODO: Probably not actually true, but *shrug*
        if target.has_condition(self.condition) {
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
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        if self.can_be_evaded && sim.do_magical_evade(user, target, Source::Ability) {
            return;
        }
        let success_chance = mod_6_formula(
            user,
            target,
            Element::None,
            self.base_chance,
            self.ignore_magic_def,
        );
        if !(sim.roll_auto_succeed() < success_chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }
        sim.add_condition(target_id, self.condition, Source::Ability);
    }
}

pub struct ElementalDamageSpellImpl {
    pub element: Element,
    pub q: i16,
    pub range: u8,
    pub ctr: Option<u8>,
    pub evadable: bool,
}

impl AbilityImpl for ElementalDamageSpellImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.ally(target) && !target.absorbs(self.element) {
            return;
        }
        if user.foe(target) && target.absorbs(self.element) {
            return;
        }
        actions.push(Action::new(ability, self.range, self.ctr, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        if target.cancels(self.element) {
            return;
        }
        if self.evadable && sim.do_magical_evade(user, target, Source::Ability) {
            return;
        }
        let damage_amount = mod_5_formula(user, target, self.element, self.q);
        sim.change_target_hp(target_id, damage_amount, Source::Ability);
    }
}

pub struct ConditionClearSpellImpl {
    pub conditions: &'static [Condition],
    pub base_chance: i16,
    pub ignore_magic_def: bool,
    pub range: u8,
    pub ctr: u8,
}

impl AbilityImpl for ConditionClearSpellImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        // TODO: Probably not actually true, but *shrug*
        if !self
            .conditions
            .iter()
            .any(|cond| target.has_condition(*cond))
        {
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
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        let success_chance = mod_6_formula(
            user,
            target,
            Element::None,
            self.base_chance,
            self.ignore_magic_def,
        );

        if !(sim.roll_auto_succeed() < success_chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }

        for cond in self.conditions {
            sim.cancel_condition(target_id, *cond, Source::Ability);
        }
    }
}

pub fn mod_2_formula_xa(
    sim: &Simulation,
    mut xa: i16,
    user: &Combatant,
    target: &Combatant,
    element: Element,
    crit: bool,
    always_apply_martial_arts: bool,
    ignores_protect_and_defense_up: bool,
) -> i16 {
    //    1. If this is a critical hit, then XA1 = XA0 + (1..XA0) - 1.
    if crit {
        xa += sim.roll_inclusive(1, xa.max(2)) - 1;
    }

    //    2. If the attack is endowed with an Element, and the caster has
    //       equipment that 'Strengthens' that element, then (XA2 = [XA1 * 5/4]),
    //       else XA2 = XA1
    if user.strengthens(element) {
        xa = (xa * 5) / 4;
    }

    xa = mod_3_formula_xa(
        xa,
        user,
        target,
        always_apply_martial_arts,
        ignores_protect_and_defense_up,
    );

    //   11. Apply zodiac multipliers:
    //           If compatibility is 'Good', then (XA11 = XA10 + [(XA10)/4]))
    //           ElseIf compatibility is 'Bad', then (XA11 = XA10 - [(XA10)/4])
    //           ElseIf compatibility is 'Best', then (XA11 = XA10 + [(XA10)/2])
    //           ElseIf compatibility is 'Worst', then (XA11 = XA10 - [(XA10)/2])
    //           Else, XA11 = XA10
    xa = (xa as f32 * user.zodiac_compatibility(target)) as i16;

    xa
}

pub fn mod_3_formula_xa(
    mut xa: i16,
    user: &Combatant,
    target: &Combatant,
    always_apply_martial_arts: bool,
    ignores_protect_and_defense_up: bool,
) -> i16 {
    //    3. If caster has Attack UP, then (XA3 = [XA2 * 4/3]), else XA3 = XA2
    if user.attack_up() {
        xa = (xa * 4) / 3;
    }

    //    4. If caster has Martial Arts AND this is not a wpn-elemental attack,
    //       then (XA4 = [XA3 * 3/2]), else XA4 = XA3
    if user.martial_arts() && (always_apply_martial_arts || user.barehanded()) {
        xa = (xa * 3) / 2;
    }

    //    5. If caster is Berserk, then (XA5 = [XA4 * 3/2]), else XA5 = XA4
    if user.berserk() {
        xa = (xa * 3) / 2;
    }

    //    6. If target has Defense UP, then (XA6 = [XA5 * 2/3]), else XA6 = XA5
    if !ignores_protect_and_defense_up && target.defense_up() {
        xa = (xa * 2) / 3;
    }

    //    7. If target has Protect, then (XA7 = [XA6 * 2/3]), else XA7 = XA6
    if !ignores_protect_and_defense_up && target.protect() {
        xa = (xa * 2) / 3;
    }

    //    8. If target is Charging, then (XA8 = [XA7 * 3/2]), else XA8 = XA7
    if target.charging() {
        xa = (xa * 3) / 2;
    }

    //    9. If target is Sleeping, then (XA9 = [XA8 * 3/2]), else XA9 = XA8
    if target.sleep() {
        xa = (xa * 3) / 2;
    }

    //   10. If target is a Chicken and/or a Frog, then (XA10 = [XA9 * 3/2]),
    //       else XA10 = XA9
    if target.chicken() || target.frog() {
        xa = (xa * 3) / 2;
    }

    xa
}

pub fn mod_5_formula_xa(
    mut xa: i16,
    user: &Combatant,
    target: &Combatant,
    element: Element,
    ignores_shell_and_defense_up: bool,
) -> i16 {
    // 1. If caster has 'Strengthen: [element of spell]', then (MA1 = [MA0 * 5/4])
    //      else MA1 = MA0
    if user.strengthens(element) {
        xa = (xa * 5) / 4;
    }
    //   2. If caster has Magic AttackUP, then (MA2 = [MA1 * 4/3]), else MA2 = MA1
    if user.magic_attack_up() {
        xa = (xa * 4) / 3;
    }

    //   3. If target has Magic DefendUP, then (MA3 = [MA2 * 2/3]), else MA3 = MA2
    if !ignores_shell_and_defense_up && target.magic_defense_up() {
        xa = (xa * 2) / 3;
    }

    //   4. If target has Shell, then (MA4 = [MA3 * 2/3]), else MA5 = MA4
    if !ignores_shell_and_defense_up && target.shell() {
        xa = (xa * 2) / 3;
    }

    //   5. Apply zodiac multipliers:
    //           If compatibility is 'Good', then (MA5 = MA4 + [(MA4)/4]))
    //           ElseIf compatibility is 'Bad', then (MA5 = MA4 - [(MA4)/4])
    //           ElseIf compatibility is 'Best', then (MA5 = MA4 + [(MA4)/2])
    //           ElseIf compatibility is 'Worst', then (MA5 = MA4 - [(MA4)/2])
    //           Else, MA5 = MA
    // TODO: Cheating for now, but I think I do want to fix this.
    xa = (xa as f32 * user.zodiac_compatibility(target)) as i16;

    //   9. If target is 'Weak' against spell's element, then
    //        Frac3 = Frac2 * 2
    //      Else, Frac3 = Frac2
    if target.weak(element) {
        xa *= 2;
    }

    //  10. If target has 'Half' spell's element, then
    //        Frac4 = Frac3 * 1/2
    //      Else, Frac4 = Frac3
    if target.halves(element) {
        xa /= 2;
    }

    //  11. If target has 'Absorb' spell's element, then
    //        Frac5 = -(Frac4)
    //      Else, Frac5 = Frac4
    if target.absorbs(element) {
        xa = -xa;
    }

    xa
}

pub fn mod_5_formula_pass_ma(
    ma: i16,
    user: &Combatant,
    target: &Combatant,
    element: Element,
    q: i16,
) -> i16 {
    let ma = mod_5_formula_xa(ma, user, target, element, false);
    //      damage = [(CFa * TFa * Q * MA5 * N) / (10000 * D)]
    (user.faith_percent() * target.faith_percent() * q as f32 * ma as f32) as i16
}

pub fn mod_5_formula(user: &Combatant, target: &Combatant, element: Element, q: i16) -> i16 {
    mod_5_formula_pass_ma(user.ma() as i16, user, target, element, q)
}

pub fn mod_6_formula(
    user: &Combatant,
    target: &Combatant,
    element: Element,
    base_chance: i16,
    ignore_magic_def: bool,
) -> f32 {
    let mut ma = user.ma();

    //   1. If caster has 'Strengthen: [element of spell]', then (MA1 = [MA0 * 5/4])
    //      else MA1 = MA0
    if user.strengthens(element) {
        ma = (ma * 5) / 4;
    }

    //   2. If caster has Magic AttackUP, then (MA2 = [MA1 * 4/3]), else MA2 = MA1
    if user.magic_attack_up() {
        ma = (ma * 4) / 3;
    }

    //   3. If target has Magic DefendUP, then (MA3 = [MA2 * 2/3]), else MA3 = MA2
    if !ignore_magic_def && target.magic_defense_up() {
        ma = (ma * 2) / 3;
    }
    //   4. If target has Shell, then (MA4 = [MA3 * 2/3]), else MA4 = MA3
    if !ignore_magic_def && target.shell() {
        ma = (ma * 2) / 3;
    }

    //   5. Calculate Z (Zodiac addend):
    //         If compatibility is 'Good', then Z = [MA4 / 4] + [Y / 4]
    //         ElseIf compatibility is 'Bad', then Z = -[MA4 / 4] - [Y / 4]
    //         ElseIf compatibility is 'Best', then Z = [MA4 / 2] + [Y / 2]
    //         ElseIf compatibility is 'Worst', then Z = -[MA4 / 2] - [Y / 2]
    //         Else, Z = 0
    //   6. Apply the spell's success% formula as follows:
    //      success% = [(CFa * TFa * (MA4 + Y + Z)) / 10000]
    //      If caster or target has Faith status, then CFa = 100 or TFa = 100,
    //      respectively.  If caster or target has Innocent status, then CFa = 0
    //      or TFa = 0, respectively.
    let mut success_chance = user.zodiac_compatibility(target);
    success_chance *= user.faith_percent();
    success_chance *= target.faith_percent();
    success_chance *= (ma as f32 + base_chance as f32) / 100.0;
    success_chance
}

pub struct CureSpellImpl {
    pub q: i16,
    pub ctr: Option<u8>,
    pub range: u8,
}

impl AbilityImpl for CureSpellImpl {
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
        actions.push(Action::new(ability, self.range, self.ctr, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let mut heal_amount = 1.0;
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        heal_amount *= user.faith_percent();
        heal_amount *= target.faith_percent();
        heal_amount *= user.ma() as f32;
        heal_amount *= self.q as f32;
        heal_amount *= user.zodiac_compatibility(target);

        do_hp_heal(sim, target_id, heal_amount as i16, true);
    }
}

pub struct DemiImpl {
    pub base_chance: i16,
    pub hp_percent: f32,
    pub range: u8,
    pub ctr: Option<u8>,
}

impl AbilityImpl for DemiImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, self.range, self.ctr, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        if sim.do_magical_evade(user, target, Source::Ability) {
            return;
        }
        let success_chance = mod_6_formula(user, target, Element::None, self.base_chance, false);
        if !(sim.roll_auto_succeed() < success_chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }

        let damage = (target.max_hp() as f32 * self.hp_percent) as i16;
        sim.change_target_hp(target_id, damage, Source::Ability);
    }
}

pub struct EmpowerImpl {
    pub range: u8,
    pub ctr: Option<u8>,
    pub brave_mod: i8,
    pub pa_buff: i8,
    pub ma_buff: i8,
    pub speed_buff: i8,
}

impl AbilityImpl for EmpowerImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, self.range, self.ctr, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        sim.change_unit_brave(target_id, self.brave_mod, Source::Ability);
        sim.change_unit_pa(target_id, self.pa_buff, Source::Ability);
        sim.change_unit_ma(target_id, self.ma_buff, Source::Ability);
        sim.change_unit_speed(target_id, self.speed_buff, Source::Ability);
    }
}
