pub use crate::sim::Combatant;

pub fn ai_target_value_sum(
    user: &Combatant,
    combatants: &[Combatant],
    ignore_confusion: bool,
) -> f32 {
    combatants
        .iter()
        .map(|target| ai_calculate_target_value(user, target, ignore_confusion))
        .sum()
}

fn ai_calculate_target_value(user: &Combatant, target: &Combatant, ignore_confusion: bool) -> f32 {
    let mut priority = target.hp_percent();
    priority += -0.51 * target.broken_equip_count() as f32;
    priority += ai_calculate_status_target_value_mod(target, ignore_confusion);
    priority += ai_calculate_caster_hate_mod(target);
    // TODO: Golem fear

    priority += ai_calculate_stat_buff_mod(target);

    if user.foe(target) {
        -priority
    } else {
        priority
    }
}

fn ai_calculate_stat_buff_mod(target: &Combatant) -> f32 {
    // The game's actual AI doesn't do this kind of calculation, but I'm going to add
    // a teeny bonus for stat buffs to try to emulate the 'only use stat buffs when there is
    // no other options' behaviour.
    target.pa_mod as f32 * 1e-5
        + target.ma_mod as f32 * 1e-5
        + target.speed_mod as f32 * 1e-5
        + target.raw_brave as f32 * 1e-5
        + target.raw_faith as f32 * 1e-5
}

fn ai_calculate_caster_hate_mod(target: &Combatant) -> f32 {
    if !target.can_cast_mp_ability() {
        0.0
    } else {
        (target.mp_percent() / 16.0) * (target.info.number_of_mp_using_abilities as f32)
    }
}

fn ai_calculate_status_target_value_mod(target: &Combatant, ignore_confusion: bool) -> f32 {
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
    if target.crystal() {
        total -= 1.5;
    }

    if target.dead() {
        total -= 1.5;
    }

    if target.conditions.status_flags == 0 {
        // Purely an optimization
        return total;
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
    if target.darkness() {
        total -= 0.50;
    }

    // # 		0x10 - Confusion				-50% -40(ffc0) (+1 / 4 if slow/stop/sleep/don't move/act/)
    if target.confusion() && !ignore_confusion {
        if target.slow()
            || target.stop()
            || target.sleep()
            || target.dont_move()
            || target.dont_act()
        {
            total += 0.25;
        } else {
            total -= 0.5;
        }
    }

    // # 		0x08 - Silence					-70.3% [-5a(ffa6) * Silence abilities] + 3 / 4
    if target.silence() {
        total -= (0.703 / 4.0) * target.info.silence_mod as f32;
    }

    // # 		0x04 - Blood Suck				-90.6% -74(ff8c) (+1 / 4 if slow/stop/sleep/don't move/act/)
    if target.blood_suck() {
        if target.slow()
            || target.stop()
            || target.sleep()
            || target.dont_move()
            || target.dont_act()
        {
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
        if target.slow()
            || target.stop()
            || target.sleep()
            || target.dont_move()
            || target.dont_act()
        {
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
