use crate::dto::patch::{Ability, BaseStats, Equipment};
use crate::sim::{Condition, ConditionBlock, Distance, Element, Gender, Location, Sign, Team,
                 TIMED_CONDITIONS_LEN, WeaponType};

#[derive(Clone)]
pub struct Combatant<'a> {
    pub name: String,
    pub team: Team,
    pub index: usize,
    pub sign: Sign,
    pub job: String,
    pub gender: Gender,
    pub base_stats: &'a BaseStats,
    pub main_hand: Option<&'a Equipment>,
    pub off_hand: Option<&'a Equipment>,
    pub headgear: Option<&'a Equipment>,
    pub armor: Option<&'a Equipment>,
    pub accessory: Option<&'a Equipment>,
    pub raw_brave: i16,
    pub raw_faith: i16,
    pub skill_flags: u64,
    pub abilities: Vec<&'a Ability>,
    pub raw_hp: i16,
    pub raw_mp: i16,
    pub ct: i16,
    pub pa_mod: i16,
    pub ma_mod: i16,
    pub speed_mod: i16,
    pub conditions: ConditionBlock,
    pub broken_items: i8,
    pub ctr: i8,
    // TODO: ctr_action

    pub on_active_turn: bool,
    pub moved_during_active_turn: bool,
    pub acted_during_active_turn: bool,
    pub took_damage_during_active_turn: bool,

    pub crystal_counter: i8,
    pub location: Location,
    // TODO: Add location module

    pub number_of_mp_using_abilities: i16,
    pub lowest_mp_cost_ability: i16,
}

impl<'a> Combatant<'a> {
    pub fn same_team(&self, other: &Combatant) -> bool {
        self.team == other.team
    }

    pub fn different_team(&self, other: &Combatant) -> bool {
        self.team != other.team
    }

    pub fn distance(&self, other: &Combatant) -> Distance {
        self.location.distance(&other.location)
    }

    pub fn max_hp(&self) -> i16 {
        self.base_stats.hp
            + self.headgear.map_or(0, |e| e.hp_bonus)
            + self.armor.map_or(0, |e| e.hp_bonus)
    }

    pub fn max_mp(&self) -> i16 {
        self.base_stats.mp
            + self.headgear.map_or(0, |e| e.mp_bonus)
            + self.armor.map_or(0, |e| e.mp_bonus)
    }

    pub fn hp(&self) -> i16 {
        self.raw_hp
    }

    pub fn set_hp_within_bounds(&mut self, new_hp: i16) {
        self.raw_hp = 0.max(new_hp.min(self.max_hp()));
    }

    pub fn hp_percent(&self) -> f32 {
        return self.hp() as f32 / self.max_hp() as f32;
    }

    pub fn mp(&self) -> i16 {
        self.raw_mp
    }

    pub fn set_mp_within_bounds(&mut self, new_mp: i16) {
        self.raw_mp = 0.max(new_mp.min(self.max_mp()));
    }

    pub fn mp_percent(&self) -> f32 {
        return self.mp() as f32 / self.max_mp() as f32;
    }

    pub fn can_cast_mp_ability(&self) -> bool {
        if self.number_of_mp_using_abilities == 0 {
            false
        } else {
            self.mp() >= self.lowest_mp_cost_ability
        }
    }

    pub fn speed(&self) -> i16 {
        self.base_stats.speed
            + self.speed_mod
            + self.main_hand.map_or(0, |e| e.speed_bonus)
            + self.off_hand.map_or(0, |e| e.speed_bonus)
            + self.headgear.map_or(0, |e| e.speed_bonus)
            + self.armor.map_or(0, |e| e.speed_bonus)
            + self.accessory.map_or(0, |e| e.speed_bonus)
    }

    pub fn brave_percent(&self) -> f32 {
        self.raw_brave as f32 / 100.0
    }

    pub fn faith_percent(&self) -> f32 {
        self.raw_faith as f32 / 100.0
    }

    fn evasion_multiplier(&self) -> f32 {
        if self.charging() || self.sleep() {
            0.0
        } else if self.abandon() {
            2.0
        } else {
            1.0
        }
    }

    pub fn class_evasion(&self) -> f32 {
        self.evasion_multiplier() * (self.base_stats.c_ev as f32 / 100.0)
    }

    pub fn weapon_evasion(&self) -> f32 {
        if !self.parry() {
            0.0
        } else {
            // TODO: Pretty sure this is wrong
            let base_w_ev = self.main_hand.map_or(0, |e| e.w_ev)
                .max(self.off_hand.map_or(0, |e| e.w_ev));
            self.evasion_multiplier() * (base_w_ev as f32 / 100.0)
        }
    }

    pub fn physical_shield_evasion(&self) -> f32 {
        let base_phys_ev = self.main_hand.map_or(0, |e| e.phys_ev)
            + self.off_hand.map_or(0, |e| e.phys_ev);
        self.evasion_multiplier() * (base_phys_ev as f32 / 100.0)
    }

    pub fn magical_shield_evasion(&self) -> f32 {
        let base_magical_ev = self.main_hand.map_or(0, |e| e.magic_ev)
            + self.off_hand.map_or(0, |e| e.magic_ev);
        self.evasion_multiplier() * (base_magical_ev as f32 / 100.0)
    }

    pub fn physical_accessory_evasion(&self) -> f32 {
        self.evasion_multiplier() *
            (self.accessory.map_or(0, |e| e.phys_ev) as f32 / 100.0)
    }

    pub fn magical_accessory_evasion(&self) -> f32 {
        self.evasion_multiplier() *
            (self.accessory.map_or(0, |e| e.magic_ev) as f32 / 100.0)
    }

    pub fn movement(&self) -> i16 {
        // TODO: Move+ skills
        self.base_stats.movement
            + self.headgear.map_or(0, |e| e.move_bonus)
            + self.armor.map_or(0, |e| e.move_bonus)
            + self.accessory.map_or(0, |e| e.move_bonus)
    }

    pub fn pa_bang(&self) -> i16 {
        self.base_stats.pa + self.pa_mod
    }

    pub fn ma_bang(&self) -> i16 {
        self.base_stats.ma + self.ma_mod
    }

    pub fn pa(&self) -> i16 {
        self.pa_bang()
            + self.main_hand.map_or(0, |e| e.pa_bonus)
            + self.off_hand.map_or(0, |e| e.pa_bonus)
            + self.headgear.map_or(0, |e| e.pa_bonus)
            + self.armor.map_or(0, |e| e.pa_bonus)
            + self.accessory.map_or(0, |e| e.pa_bonus)
    }

    pub fn ma(&self) -> i16 {
        self.ma_bang()
            + self.main_hand.map_or(0, |e| e.ma_bonus)
            + self.off_hand.map_or(0, |e| e.ma_bonus)
            + self.headgear.map_or(0, |e| e.ma_bonus)
            + self.armor.map_or(0, |e| e.ma_bonus)
            + self.accessory.map_or(0, |e| e.ma_bonus)
    }

    //     @property
    //     def jump(self) -> int:
    //         jump = self.stats.jump + sum([e.jump_bonus for e in self.all_equips])
    //         if self.raw_combatant['MoveSkill'].startswith('Jump+'):
    //             jump += int(self.raw_combatant['MoveSkill'][-1])
    //         elif self.raw_combatant['MoveSkill'] == 'Ignore Height':
    //             jump = 20
    //         elif self.raw_combatant['MoveSkill'].startswith('Teleport'):
    //             jump = 20
    //         elif 'Fly' in self.stats.innates or 'Fly' == self.raw_combatant['MoveSkill']:
    //             jump = 20
    //         return jump

    pub fn abandon(&self) -> bool {
        // TODO: implement
        false
    }

    pub fn parry(&self) -> bool {
        // TODO: implement
        false
    }

    pub fn has_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Critical => self.hp() <= self.max_hp() / 5,
            Condition::Death => self.dead(),
            _ => self.conditions.has(condition)
        }
    }

    pub fn cancel_condition(&mut self, condition: Condition) {
        // TODO: Special handling of charging/performing/etc
        self.conditions.remove(condition);
    }

    pub fn dead(&self) -> bool {
        self.hp() == 0
    }

    pub fn charging(&self) -> bool {
        self.conditions.has(Condition::Charging)
    }

    pub fn sleep(&self) -> bool {
        self.conditions.has(Condition::Sleep)
    }

    pub fn barehanded(&self) -> bool {
        self.main_hand.map_or(false, |e| e.weapon_type.is_none())
    }

    pub fn calculate_weapon_xa(&self, weapon: Option<&Equipment>, k: i16) -> i16 {
        let weapon_type = weapon.and_then(|e| e.weapon_type);
        match weapon_type {
            None => (self.pa() + k * self.raw_brave) / 100,
            Some(WeaponType::Knife) | Some(WeaponType::NinjaSword) | Some(WeaponType::Bow) =>
                (self.pa() + k + self.speed() + k) / 2,
            Some(WeaponType::KnightSword) | Some(WeaponType::Katana) =>
                (self.pa() + k * self.raw_brave) / 100,
            Some(WeaponType::Sword) | Some(WeaponType::Pole) | Some(WeaponType::Spear) | Some(WeaponType::Crossbow) =>
                self.pa() + k,
            Some(WeaponType::Staff) =>
                self.ma() + k,
            // TODO: Random roll here!!
            Some(WeaponType::Flail) | Some(WeaponType::Bag) =>
                self.pa() + k,
            Some(WeaponType::Cloth) | Some(WeaponType::Harp) | Some(WeaponType::Book) =>
                (self.pa() + k + self.ma() + k) / 2,
            // TODO: Magical guns
            Some(WeaponType::Gun) =>
                weapon.unwrap().wp + k
        }
    }
}


//     def has_ability(self, name: str) -> bool:
//         return name in self.ability_by_name
//

//     @property
//     def healthy(self) -> bool:
//         return self.hp > 0 and not self.petrified
//
//     @property
//     def dead(self) -> bool:
//         return self.hp == 0
//
//     @property
//     def crystal(self) -> bool:
//         return self.crystal_counter == 0
//
//     @property
//     def undead(self) -> bool:
//         return self.status_flags & UNDEAD_FLAG != 0
//
//     @property
//     def death_sentence(self) -> bool:
//         return self.status_flags & DEATH_SENTENCE_FLAG != 0
//
//     @property
//     def reraise(self) -> bool:
//         return self.status_flags & RERAISE_FLAG != 0
//
//     @property
//     def critical(self) -> bool:
//         return self.has_status(CRITICAL)
//
//     @property
//     def dont_move(self) -> bool:
//         return self.status_flags & DONT_MOVE_FLAG != 0
//
//     @property
//     def dont_act(self) -> bool:
//         return self.status_flags & DONT_ACT_FLAG != 0
//
//     @property
//     def silence(self) -> bool:
//         return self.status_flags & SILENCE_FLAG != 0
//
//     @property
//     def innocent(self) -> bool:
//         return self.status_flags & INNOCENT_FLAG != 0
//
//     @property
//     def reflect(self) -> bool:
//         return self.status_flags & REFLECT_FLAG != 0
//
//     @property
//     def charging(self) -> bool:
//         return self.has_status(CHARGING)
//
//     @property
//     def transparent(self) -> bool:
//         return self.status_flags & TRANSPARENT_FLAG != 0
//
//     @property
//     def berserk(self) -> bool:
//         return self.status_flags & BERSERK_FLAG != 0
//
//     @property
//     def blood_suck(self) -> bool:
//         return self.status_flags & BLOOD_SUCK_FLAG != 0
//
//     @property
//     def oil(self) -> bool:
//         return self.status_flags & OIL_FLAG != 0
//
//     @property
//     def f32(self) -> bool:
//         return self.status_flags & f32_FLAG != 0
//
//     @property
//     def sleep(self) -> bool:
//         return self.status_flags & SLEEP_FLAG != 0
//
//     @property
//     def shell(self) -> bool:
//         return self.status_flags & SHELL_FLAG != 0
//
//     @property
//     def protect(self) -> bool:
//         return self.status_flags & PROTECT_FLAG != 0
//
//     @property
//     def wall(self) -> bool:
//         return self.status_flags & WALL_FLAG != 0
//
//     @property
//     def haste(self) -> bool:
//         return self.status_flags & HASTE_FLAG != 0
//
//     @property
//     def slow(self) -> bool:
//         return self.status_flags & SLOW_FLAG != 0
//
//     @property
//     def stop(self) -> bool:
//         return self.status_flags & STOP_FLAG != 0
//
//     @property
//     def regen(self) -> bool:
//         return self.status_flags & REGEN_FLAG != 0
//
//     @property
//     def poison(self) -> bool:
//         return self.status_flags & POISON_FLAG != 0
//
//     @property
//     def chicken(self) -> bool:
//         return self.status_flags & CHICKEN_FLAG != 0
//
//     @property
//     def frog(self) -> bool:
//         return self.status_flags & FROG_FLAG != 0
//
//     @property
//     def petrified(self) -> bool:
//         return self.status_flags & PETRIFY_FLAG != 0
//
//     @property
//     def charm(self) -> bool:
//         return self.status_flags & CHARM_FLAG != 0
//
//     @property
//     def confusion(self) -> bool:
//         return self.status_flags & CONFUSION_FLAG != 0
//
//     @property
//     def abandon(self) -> bool:
//         return 'Abandon' in self.skills
//
//     @property
//     def parry(self) -> bool:
//         return 'Parry' in self.skills
//
//     @property
//     def blade_grasp(self) -> bool:
//         return 'Blade Grasp' in self.skills
//
//     @property
//     def arrow_guard(self) -> bool:
//         return 'Arrow Guard' in self.skills
//
//     @property
//     def throw_item(self) -> bool:
//         return 'Throw Item' in self.skills
//
//     @property
//     def attack_up(self) -> bool:
//         return 'Attack UP' in self.skills
//
//     @property
//     def defense_up(self) -> bool:
//         return 'Defense UP' in self.skills
//
//     @property
//     def concentrate(self) -> bool:
//         return 'Concentrate' in self.skills
//
//     @property
//     def martial_arts(self) -> bool:
//         return 'Martial Arts' in self.skills
//
//     @property
//     def barehanded(self) -> bool:
//         return self.mainhand.weapon_type is None or self.mainhand.weapon_type == 'Shield'
//
//     @property
//     def double_hand(self) -> bool:
//         return 'Doublehand' in self.skills
//
//     @property
//     def auto_potion(self) -> bool:
//         return 'Auto Potion' in self.skills
//
//     @property
//     def dual_wield(self) -> bool:
//         return 'Dual Wield' in self.skills
//
//     @property
//     def has_offhand_weapon(self) -> bool:
//         return self.offhand.weapon_type is not None
//
//     @property
//     def mana_shield(self) -> bool:
//         return 'Mana Shield' in self.skills
//
//     @property
//     def damage_split(self) -> bool:
//         return 'Damage Split' in self.skills
//
//     def zodiac_compatibility(self, other: 'Combatant') -> f32:
//         s1 = ZODIAC_INDEX[self.sign]
//         s2 = ZODIAC_INDEX[other.sign]
//         if ZODIAC_CHART[s1][s2] == 'O':
//             return 1.0
//         elif ZODIAC_CHART[s1][s2] == '+':
//             return 1.25
//         elif ZODIAC_CHART[s1][s2] == '-':
//             return 0.75
//         elif ZODIAC_CHART[s1][s2] == '?':
//             if self.gender == 'Monster' or other.gender == 'Monster':
//                 return 0.75
//             elif self.gender != other.gender:
//                 return 1.5
//             else:
//                 return 0.5
//         else:
//             raise Exception(f"Missing case in zodiac compatibility calculation\
//              {self.sign} {self.gender} vs {other.sign} {other.gender}")
//
//     def absorbs(self, element) -> bool:
//         return element in self.stats.absorbs or any((element in e.absorbs for e in self.all_equips))
//
//     def halves(self, element) -> bool:
//         return element in self.stats.halves or any((element in e.halves for e in self.all_equips))
//
//     def weak(self, element) -> bool:
//         return element in self.stats.weaknesses or any((element in e.weaknesses for e in self.all_equips))
//
//     def strengthens(self, element) -> bool:
//         return any((element in e.strengthens for e in self.all_equips))
//
//     def immune_to(self, element) -> bool:
//         return any((element in e.immune_to for e in self.all_equips))
//

