use std::collections::HashSet;

use crate::dto::rust;
use crate::dto::rust::{BaseStats, Equipment, Patch};
use crate::sim::{
    Action, Condition, ConditionBlock, ConditionFlags, Distance, Element, Gender, Location, Sign,
    SkillBlock, Team, ALL_CONDITIONS,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CombatantId {
    pub id: u8,
}

impl CombatantId {
    pub fn new(id: u8) -> CombatantId {
        CombatantId { id }
    }

    pub fn index(&self) -> usize {
        self.id as usize
    }
}

pub const COMBATANT_IDS_LEN: usize = 8;
pub const COMBATANT_IDS: [CombatantId; COMBATANT_IDS_LEN] = [
    CombatantId { id: 0 },
    CombatantId { id: 1 },
    CombatantId { id: 2 },
    CombatantId { id: 3 },
    CombatantId { id: 4 },
    CombatantId { id: 5 },
    CombatantId { id: 6 },
    CombatantId { id: 7 },
];

#[derive(Copy, Clone, Debug)]
pub struct SlowAction {
    ctr: i8,
    action: Action,
}

#[derive(Clone, Debug)]
pub struct CombatantInfo<'a> {
    pub id: CombatantId,
    pub team: Team,
    pub skill_block: SkillBlock,
    pub base_stats: &'a BaseStats,
    pub number_of_mp_using_abilities: i16,
    pub lowest_mp_cost_ability: i16,
    pub abilities: HashSet<String>,
    pub name: &'a str,
    pub sign: Sign,
    pub job: &'a str,
    pub gender: Gender,
    pub main_hand: Option<&'a Equipment>,
    pub off_hand: Option<&'a Equipment>,
    pub headgear: Option<&'a Equipment>,
    pub armor: Option<&'a Equipment>,
    pub accessory: Option<&'a Equipment>,
    pub starting_brave: i8,
    pub starting_faith: i8,
}

impl<'a> CombatantInfo<'a> {
    pub fn new(
        id: CombatantId,
        team: Team,
        src: &'a rust::Combatant,
        patch: &'a Patch,
    ) -> CombatantInfo<'a> {
        let short_class = src.class.replace(" ", "");
        let base_stats = &patch
            .base_stats
            .by_job_gender
            .get(&(short_class, src.gender))
            .unwrap();
        let mut skills = vec![];
        skills.extend(&base_stats.innates);
        skills.push(&src.action_skill);
        skills.push(&src.reaction_skill);
        skills.push(&src.support_skill);
        skills.push(&src.move_skill);
        CombatantInfo {
            base_stats,
            id,
            team,
            number_of_mp_using_abilities: 0,
            lowest_mp_cost_ability: 0,
            name: &src.name,
            sign: src.sign,
            job: &src.class,
            gender: src.gender,
            skill_block: SkillBlock::new(skills.as_slice()),
            main_hand: patch.equipment.by_name.get(&src.main_hand),
            off_hand: patch.equipment.by_name.get(&src.off_hand),
            headgear: patch.equipment.by_name.get(&src.head),
            armor: patch.equipment.by_name.get(&src.armor),
            accessory: patch.equipment.by_name.get(&src.accessory),
            starting_brave: src.brave,
            starting_faith: src.faith,
            abilities: src.all_abilities.iter().cloned().collect(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Combatant<'a> {
    pub info: &'a CombatantInfo<'a>,
    pub conditions: ConditionBlock,
    pub raw_hp: i16,
    pub crystal_counter: i8,
    pub ct: u8,
    pub speed_mod: i8,
    pub raw_mp: i16,
    pub broken_items: i8,
    pub location: Location,
    pub on_active_turn: bool,
    pub moved_during_active_turn: bool,
    pub acted_during_active_turn: bool,
    pub took_damage_during_active_turn: bool,
    pub raw_brave: i8,
    pub raw_faith: i8,
    pub pa_mod: i8,
    pub ma_mod: i8,
    pub death_sentence_counter: i8,
    pub ctr_action: Option<SlowAction>,
}

impl<'a> Combatant<'a> {
    pub fn new(info: &'a CombatantInfo<'a>) -> Combatant<'a> {
        let mut out = Combatant {
            info,
            raw_hp: 0,
            raw_mp: 0,
            ct: 0,
            speed_mod: 0,
            conditions: ConditionBlock::new(),
            broken_items: 0,
            location: Location::new(0),
            on_active_turn: false,
            moved_during_active_turn: false,
            acted_during_active_turn: false,
            took_damage_during_active_turn: false,
            raw_brave: info.starting_brave,
            raw_faith: info.starting_faith,
            ctr_action: None,
            pa_mod: 0,
            ma_mod: 0,
            crystal_counter: 4,
            death_sentence_counter: 4,
        };
        out.raw_hp = out.max_hp();
        out.raw_mp = out.max_mp();

        let mut initial_flags: ConditionFlags = 0;
        for eq in out.all_equips() {
            initial_flags |= eq.initial;
        }
        for condition in ALL_CONDITIONS.iter() {
            if initial_flags & condition.flag() != 0 {
                out.add_condition(*condition);
            }
        }
        out
    }

    pub fn id(&self) -> CombatantId {
        self.info.id
    }

    pub fn name(&self) -> &'a str {
        self.info.name
    }

    pub fn team(&self) -> Team {
        self.info.team
    }

    pub fn same_team(&self, other: &Combatant) -> bool {
        self.team() == other.team()
    }

    pub fn different_team(&self, other: &Combatant) -> bool {
        self.team() != other.team()
    }

    pub fn base_stats(&self) -> &'a BaseStats {
        self.info.base_stats
    }

    pub fn distance(&self, other: &Combatant) -> Distance {
        self.location.distance(&other.location)
    }

    pub fn knows_ability(&self, ability: &str) -> bool {
        self.info.abilities.contains(ability)
    }

    pub fn max_hp(&self) -> i16 {
        self.base_stats().hp
            + self.headgear().map_or(0, |e| e.hp_bonus)
            + self.armor().map_or(0, |e| e.hp_bonus)
    }

    pub fn max_mp(&self) -> i16 {
        self.base_stats().mp
            + self.headgear().map_or(0, |e| e.mp_bonus)
            + self.armor().map_or(0, |e| e.mp_bonus)
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
        if self.info.number_of_mp_using_abilities == 0 {
            false
        } else {
            self.mp() >= self.info.lowest_mp_cost_ability
        }
    }

    pub fn all_conditions(&self) -> Vec<Condition> {
        let mut conditions = vec![];
        for condition in ALL_CONDITIONS.iter() {
            if self.has_condition(*condition) {
                conditions.push(*condition)
            }
        }
        conditions
    }

    pub fn main_hand(&self) -> Option<&'a Equipment> {
        self.info.main_hand
    }

    pub fn off_hand(&self) -> Option<&'a Equipment> {
        self.info.off_hand
    }

    pub fn headgear(&self) -> Option<&'a Equipment> {
        self.info.headgear
    }

    pub fn armor(&self) -> Option<&'a Equipment> {
        self.info.armor
    }

    pub fn accessory(&self) -> Option<&'a Equipment> {
        self.info.accessory
    }

    pub fn speed(&self) -> i8 {
        self.base_stats().speed
            + self.speed_mod
            + self.main_hand().map_or(0, |e| e.speed_bonus)
            + self.off_hand().map_or(0, |e| e.speed_bonus)
            + self.headgear().map_or(0, |e| e.speed_bonus)
            + self.armor().map_or(0, |e| e.speed_bonus)
            + self.accessory().map_or(0, |e| e.speed_bonus)
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
        self.evasion_multiplier() * (self.base_stats().c_ev as f32 / 100.0)
    }

    pub fn weapon_evasion(&self) -> f32 {
        if !self.parry() {
            0.0
        } else {
            // TODO: Pretty sure this is wrong
            let base_w_ev = self
                .main_hand()
                .map_or(0, |e| e.w_ev)
                .max(self.off_hand().map_or(0, |e| e.w_ev));
            self.evasion_multiplier() * (base_w_ev as f32 / 100.0)
        }
    }

    pub fn physical_shield_evasion(&self) -> f32 {
        let base_phys_ev =
            self.main_hand().map_or(0, |e| e.phys_ev) + self.off_hand().map_or(0, |e| e.phys_ev);
        self.evasion_multiplier() * (base_phys_ev as f32 / 100.0)
    }

    pub fn magical_shield_evasion(&self) -> f32 {
        let base_magical_ev =
            self.main_hand().map_or(0, |e| e.magic_ev) + self.off_hand().map_or(0, |e| e.magic_ev);
        self.evasion_multiplier() * (base_magical_ev as f32 / 100.0)
    }

    pub fn physical_accessory_evasion(&self) -> f32 {
        self.evasion_multiplier() * (self.accessory().map_or(0, |e| e.phys_ev) as f32 / 100.0)
    }

    pub fn magical_accessory_evasion(&self) -> f32 {
        self.evasion_multiplier() * (self.accessory().map_or(0, |e| e.magic_ev) as f32 / 100.0)
    }

    pub fn movement(&self) -> i8 {
        // TODO: Move+ skills
        self.base_stats().movement
            + self.headgear().map_or(0, |e| e.move_bonus)
            + self.armor().map_or(0, |e| e.move_bonus)
            + self.accessory().map_or(0, |e| e.move_bonus)
    }

    pub fn pa_bang(&self) -> i8 {
        self.base_stats().pa + self.pa_mod
    }

    pub fn ma_bang(&self) -> i8 {
        self.base_stats().ma + self.ma_mod
    }

    pub fn pa(&self) -> i8 {
        self.pa_bang()
            + self.main_hand().map_or(0, |e| e.pa_bonus)
            + self.off_hand().map_or(0, |e| e.pa_bonus)
            + self.headgear().map_or(0, |e| e.pa_bonus)
            + self.armor().map_or(0, |e| e.pa_bonus)
            + self.accessory().map_or(0, |e| e.pa_bonus)
    }

    pub fn ma(&self) -> i8 {
        self.ma_bang()
            + self.main_hand().map_or(0, |e| e.ma_bonus)
            + self.off_hand().map_or(0, |e| e.ma_bonus)
            + self.headgear().map_or(0, |e| e.ma_bonus)
            + self.armor().map_or(0, |e| e.ma_bonus)
            + self.accessory().map_or(0, |e| e.ma_bonus)
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

    pub fn gender(&self) -> Gender {
        self.info.gender
    }

    pub fn monster(&self) -> bool {
        self.gender() == Gender::Monster
    }

    pub fn tick_condition(&mut self, condition: Condition) -> Option<bool> {
        self.conditions.tick(condition)
    }

    pub fn has_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Critical => !self.dead() && self.hp() <= self.max_hp() / 5,
            Condition::Death => self.dead(),
            _ => self.conditions.has(condition),
        }
    }

    pub fn cancel_condition(&mut self, condition: Condition) {
        // TODO: Special handling of charging/performing/etc
        self.conditions.remove(condition);
    }

    pub fn add_condition(&mut self, condition: Condition) {
        if condition == Condition::DeathSentence {
            self.reset_death_sentence_counter()
        }
        self.conditions.add(condition);
    }

    pub fn healthy(&self) -> bool {
        !self.dead() && !self.crystal() && !self.petrify()
    }

    pub fn dead(&self) -> bool {
        self.hp() == 0
    }

    pub fn critical(&self) -> bool {
        self.has_condition(Condition::Critical)
    }

    pub fn charging(&self) -> bool {
        self.conditions.has(Condition::Charging)
    }

    pub fn reset_crystal_counter(&mut self) {
        self.crystal_counter = 4;
    }

    pub fn tick_crystal_counter(&mut self) -> bool {
        if self.crystal_counter > 0 {
            self.crystal_counter -= 1;
        }
        self.crystal()
    }

    fn reset_death_sentence_counter(&mut self) {
        self.death_sentence_counter = 4;
    }

    pub fn tick_death_sentence_counter(&mut self) -> bool {
        if self.death_sentence_counter > 0 {
            self.death_sentence_counter -= 1;
        }
        self.death_sentence_counter == 0
    }

    pub fn crystal(&self) -> bool {
        self.crystal_counter == 0
    }

    pub fn reraise(&self) -> bool {
        self.conditions.has(Condition::Reraise)
    }

    pub fn undead(&self) -> bool {
        self.info.skill_block.innate_undead() || self.conditions.has(Condition::Undead)
    }

    pub fn sleep(&self) -> bool {
        self.conditions.has(Condition::Sleep)
    }

    pub fn petrify(&self) -> bool {
        self.conditions.has(Condition::Petrify)
    }

    pub fn haste(&self) -> bool {
        self.conditions.has(Condition::Haste)
    }

    pub fn slow(&self) -> bool {
        self.conditions.has(Condition::Slow)
    }

    pub fn stop(&self) -> bool {
        self.conditions.has(Condition::Stop)
    }

    pub fn regen(&self) -> bool {
        self.conditions.has(Condition::Regen)
    }

    pub fn poison(&self) -> bool {
        self.conditions.has(Condition::Poison)
    }

    pub fn blood_suck(&self) -> bool {
        self.conditions.has(Condition::BloodSuck)
    }

    pub fn berserk(&self) -> bool {
        self.conditions.has(Condition::Berserk)
    }

    pub fn dont_move(&self) -> bool {
        self.conditions.has(Condition::DontMove)
    }

    pub fn dont_act(&self) -> bool {
        self.conditions.has(Condition::DontAct)
    }

    pub fn darkness(&self) -> bool {
        self.conditions.has(Condition::Darkness)
    }

    pub fn confusion(&self) -> bool {
        self.conditions.has(Condition::Confusion)
    }

    pub fn silence(&self) -> bool {
        self.conditions.has(Condition::Silence)
    }

    pub fn oil(&self) -> bool {
        self.conditions.has(Condition::Oil)
    }

    pub fn float(&self) -> bool {
        self.conditions.has(Condition::Float)
    }

    pub fn transparent(&self) -> bool {
        self.conditions.has(Condition::Transparent)
    }

    pub fn chicken(&self) -> bool {
        // TODO: Handle specially like critical?
        self.conditions.has(Condition::Chicken)
    }

    pub fn frog(&self) -> bool {
        self.conditions.has(Condition::Frog)
    }

    pub fn protect(&self) -> bool {
        self.conditions.has(Condition::Protect)
    }

    pub fn shell(&self) -> bool {
        self.conditions.has(Condition::Shell)
    }

    pub fn charm(&self) -> bool {
        self.conditions.has(Condition::Charm)
    }

    pub fn wall(&self) -> bool {
        self.conditions.has(Condition::Wall)
    }

    pub fn faith(&self) -> bool {
        self.conditions.has(Condition::Faith)
    }

    pub fn innocent(&self) -> bool {
        self.conditions.has(Condition::Innocent)
    }

    pub fn reflect(&self) -> bool {
        self.conditions.has(Condition::Reflect)
    }

    pub fn death_sentence(&self) -> bool {
        self.conditions.has(Condition::DeathSentence)
    }

    pub fn barehanded(&self) -> bool {
        self.main_hand().map_or(true, |e| e.weapon_type.is_none())
    }

    // FIXME: temporary solution, want to remove this allocation
    pub fn all_equips(&self) -> Vec<&'a Equipment> {
        let mut out = vec![];
        out.extend(self.main_hand());
        out.extend(self.off_hand());
        out.extend(self.headgear());
        out.extend(self.armor());
        out.extend(self.accessory());
        out
    }

    pub fn any_equip<P>(&self, p: P) -> bool
    where
        P: Fn(&'a Equipment) -> bool,
    {
        self.main_hand().map_or(false, &p)
            || self.off_hand().map_or(false, &p)
            || self.headgear().map_or(false, &p)
            || self.armor().map_or(false, &p)
            || self.accessory().map_or(false, &p)
    }

    pub fn absorbs(&self, element: Element) -> bool {
        self.base_stats().absorbs & element.flag() != 0
            || self.any_equip(|eq| eq.absorbs & element.flag() != 0)
    }

    pub fn halves(&self, element: Element) -> bool {
        self.base_stats().halves & element.flag() != 0
            || self.any_equip(|eq| eq.halves & element.flag() != 0)
    }

    pub fn weak(&self, element: Element) -> bool {
        self.base_stats().weaknesses & element.flag() != 0
            || self.any_equip(|eq| eq.weaknesses & element.flag() != 0)
    }

    pub fn strengthens(&self, element: Element) -> bool {
        self.any_equip(|eq| eq.strengthens & element.flag() != 0)
    }

    pub fn immune_to(&self, condition: Condition) -> bool {
        self.any_equip(|eq| eq.immune_to & condition.flag() != 0)
    }

    pub fn abandon(&self) -> bool {
        self.info.skill_block.abandon()
    }

    pub fn parry(&self) -> bool {
        self.info.skill_block.parry()
    }

    pub fn blade_grasp(&self) -> bool {
        self.info.skill_block.blade_grasp()
    }

    pub fn concentrate(&self) -> bool {
        self.info.skill_block.concentrate()
    }

    pub fn dual_wield(&self) -> bool {
        self.info.skill_block.dual_wield()
    }

    pub fn double_hand(&self) -> bool {
        self.info.skill_block.double_hand()
    }

    pub fn martial_arts(&self) -> bool {
        self.info.skill_block.martial_arts()
    }

    pub fn attack_up(&self) -> bool {
        self.info.skill_block.attack_up()
    }

    pub fn defense_up(&self) -> bool {
        self.info.skill_block.defense_up()
    }

    pub fn counter(&self) -> bool {
        self.info.skill_block.counter()
    }

    pub fn move_hp_up(&self) -> bool {
        self.info.skill_block.move_hp_up()
    }

    pub fn move_mp_up(&self) -> bool {
        self.info.skill_block.move_mp_up()
    }

    pub fn sicken(&self) -> bool {
        self.info.skill_block.sicken()
    }

    pub fn mana_shield(&self) -> bool {
        self.info.skill_block.mana_shield()
    }

    pub fn damage_split(&self) -> bool {
        self.info.skill_block.damage_split()
    }

    pub fn auto_potion(&self) -> bool {
        self.info.skill_block.auto_potion()
    }

    pub fn throw_item(&self) -> bool {
        self.info.skill_block.throw_item()
    }

    pub fn skill_set_item(&self) -> bool {
        self.info.skill_block.skill_set_item()
    }

    pub fn zodiac_compatibility(&self, other: &Combatant) -> f32 {
        let s1 = self.info.sign.index();
        let s2 = other.info.sign.index();
        match ZODIAC_CHART[s1 * 13 + s2] {
            b'O' => 1.0,
            b'+' => 1.25,
            b'-' => 0.75,
            b'?' => {
                if self.monster() || other.monster() {
                    0.75
                } else if self.gender() != other.gender() {
                    1.5
                } else {
                    0.5
                }
            }
            _ => unreachable!("found symbol that shouldn't be in table"),
        }
    }
}

const ZODIAC_CHART: &[u8; 13 * 13] = b"OOO-+O?O+-OOO\
    OOOO-+O?O+-OO\
    OOOOO-+O?O+-O\
    -OOOOO-+O?O+O\
    +-OOOOO-+O?OO\
    O+-OOOOO-+O?O\
    ?O+-OOOOO-+OO\
    O?O+-OOOOO-+O\
    +O?O+-OOOOO-O\
    -+O?O+-OOOOOO\
    O-+O?O+-OOOOO\
    OO-+O?O+-OOOO\
    OOOOOOOOOOOOO";

//     @property
//     def arrow_guard(self) -> bool:
//         return 'Arrow Guard' in self.skills
//
//     @property
//     def has_offhand_weapon(self) -> bool:
//         return self.offhand.weapon_type is not None
//
