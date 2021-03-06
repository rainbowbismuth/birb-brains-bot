use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::dto::python;
use crate::sim::{Condition, ConditionFlags, Element, ElementFlags, Facing, WeaponType};
use crate::sim::{Gender, Sign};

#[derive(Serialize, Deserialize)]
pub struct MatchUp {
    pub tournament_id: u64,
    pub modified: u64,
    pub left: Team,
    pub right: Team,
    pub left_wins: Option<bool>,
    pub arena_name: String,
    pub arena: Arena,
}

impl MatchUp {
    pub fn from_python(match_up: python::MatchUp, arena: python::Arena) -> MatchUp {
        MatchUp {
            tournament_id: match_up.tournament_id,
            modified: match_up.tournament_id as u64,
            left: Team::from_python(match_up.left),
            right: Team::from_python(match_up.right),
            left_wins: match_up.left_wins,
            arena_name: match_up.game_map,
            arena: Arena::from_python(arena),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Team {
    pub color: String,
    pub combatants: Vec<Combatant>,
}

impl Team {
    pub fn from_python(team: python::Team) -> Team {
        Team {
            color: team.color,
            combatants: team
                .combatants
                .into_iter()
                .map(Combatant::from_python)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Combatant {
    pub name: String,
    pub gender: Gender,
    pub sign: Sign,
    pub brave: i8,
    pub faith: i8,
    pub class: String,
    pub action_skill: String,
    pub reaction_skill: String,
    pub support_skill: String,
    pub move_skill: String,
    pub main_hand: String,
    pub off_hand: String,
    pub head: String,
    pub armor: String,
    pub accessory: String,
    pub all_abilities: Vec<String>,
}

impl Combatant {
    pub fn from_python(mut combatant: python::Combatant) -> Combatant {
        let mut all_abilities = combatant.class_skills;
        all_abilities.append(&mut combatant.extra_skills);
        Combatant {
            name: combatant.name,
            gender: Gender::parse(&combatant.gender).unwrap(),
            sign: Sign::parse(&combatant.sign).unwrap(),
            brave: combatant.brave,
            faith: combatant.faith,
            class: combatant.class,
            action_skill: combatant.action_skill,
            reaction_skill: combatant.reaction_skill,
            support_skill: combatant.support_skill,
            move_skill: combatant.move_skill,
            main_hand: combatant.mainhand,
            off_hand: combatant.offhand,
            head: combatant.head,
            armor: combatant.armor,
            accessory: combatant.accessory,
            all_abilities,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Patch {
    pub time: u64,
    pub ability: AbilityData,
    pub equipment: EquipmentData,
    pub base_stats: BaseStatsData,
}

impl Patch {
    pub fn from_python(patch: python::Patch) -> Patch {
        Patch {
            time: patch.time as u64,
            ability: AbilityData::from_python(patch.ability),
            equipment: EquipmentData::from_python(patch.equipment),
            base_stats: BaseStatsData::from_python(patch.base_stats),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AbilityData {
    pub by_name: HashMap<String, Ability>,
}

impl AbilityData {
    pub fn from_python(ability_data: python::AbilityData) -> AbilityData {
        AbilityData {
            by_name: ability_data
                .by_name
                .into_iter()
                .map(|(k, v)| (k, Ability::from_python(v)))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Ability {
    pub name: String,
    pub multiplier: Option<String>,
    pub hit_chance: HitChance,
    pub damage: bool,
    pub heals: bool,
    pub element: Option<Element>,
    pub range: u8,
    pub aoe: Option<u8>,
    pub ct: i8,
    pub mp: i8,
    pub ma_constant: Option<i8>,
    pub adds: ConditionFlags,
    pub cancels: ConditionFlags,
    pub chance_to_add: ConditionFlags,
    pub chance_to_cancel: ConditionFlags,
}

impl Ability {
    pub fn from_python(ability: python::Ability) -> Ability {
        let mut new_ability = Ability {
            name: ability.name,
            multiplier: ability.multiplier,
            hit_chance: HitChance::from_python(ability.hit_chance),
            damage: ability.damage,
            heals: ability.heals,
            element: ability.element.map(|x| Element::parse(&x).unwrap()),
            range: ability.range,
            aoe: ability.aoe,
            ct: ability.ct,
            mp: ability.mp,
            ma_constant: ability.ma_constant,
            adds: 0,
            cancels: 0,
            chance_to_add: 0,
            chance_to_cancel: 0,
        };

        ability
            .adds
            .iter()
            .for_each(|cond| new_ability.adds |= Condition::parse(cond).unwrap().flag());
        ability
            .cancels
            .iter()
            .for_each(|cond| new_ability.cancels |= Condition::parse(cond).unwrap().flag());
        ability
            .chance_to_add
            .iter()
            .for_each(|cond| new_ability.chance_to_add |= Condition::parse(cond).unwrap().flag());
        ability.chance_to_cancel.iter().for_each(|cond| {
            new_ability.chance_to_cancel |= Condition::parse(cond).unwrap().flag()
        });

        new_ability
    }
}

#[derive(Serialize, Deserialize)]
pub struct HitChance {
    pub ma_plus: Option<u8>,
    pub pa_plus: Option<u8>,
    pub speed_plus: Option<u8>,
    pub pa_wp_plus: Option<u8>,
    pub times_faith: bool,
}

impl HitChance {
    pub fn from_python(hit_chance: python::HitChance) -> HitChance {
        HitChance {
            ma_plus: hit_chance.ma_plus,
            pa_plus: hit_chance.pa_plus,
            speed_plus: hit_chance.speed_plus,
            pa_wp_plus: hit_chance.pa_wp_plus,
            times_faith: hit_chance.times_faith,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentData {
    pub by_name: HashMap<String, Equipment>,
}

impl EquipmentData {
    pub fn from_python(equipment_data: python::EquipmentData) -> EquipmentData {
        EquipmentData {
            by_name: equipment_data
                .by_name
                .into_iter()
                .map(|(k, v)| (k, Equipment::from_python(v)))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Equipment {
    pub name: String,
    pub hp_bonus: i16,
    pub mp_bonus: i16,
    pub speed_bonus: i8,
    pub pa_bonus: i8,
    pub ma_bonus: i8,
    pub wp: i8,
    pub absorbs_hp: bool,
    pub range: u8,
    pub w_ev: i8,
    pub phys_ev: i8,
    pub magic_ev: i8,
    pub move_bonus: i8,
    pub jump_bonus: i8,
    pub weapon_type: Option<WeaponType>,
    pub weapon_element: Option<Element>,
    pub strengthens: ElementFlags,
    pub absorbs: ElementFlags,
    pub halves: ElementFlags,
    pub weaknesses: ElementFlags,
    pub cancels: ConditionFlags,
    pub cancels_element: ElementFlags,
    pub initial: ConditionFlags,
    pub permanent: ConditionFlags,
    pub chance_to_add: ConditionFlags,
    pub chance_to_cancel: ConditionFlags,
    pub immune_to: ConditionFlags,
}

impl Equipment {
    pub fn from_python(equipment: python::Equipment) -> Equipment {
        let mut new_equipment = Equipment {
            name: equipment.name,
            hp_bonus: equipment.hp_bonus,
            mp_bonus: equipment.mp_bonus,
            speed_bonus: equipment.speed_bonus,
            pa_bonus: equipment.pa_bonus,
            ma_bonus: equipment.ma_bonus,
            wp: equipment.wp,
            absorbs_hp: equipment.absorbs_hp,
            range: equipment.range,
            w_ev: equipment.w_ev,
            phys_ev: equipment.phys_ev,
            magic_ev: equipment.magic_ev,
            move_bonus: equipment.move_bonus,
            jump_bonus: equipment.jump_bonus,
            weapon_type: equipment
                .weapon_type
                .map(|x| WeaponType::parse(&x).unwrap()),
            weapon_element: equipment
                .weapon_element
                .map(|x| Element::parse(&x).unwrap()),
            strengthens: 0,
            absorbs: 0,
            halves: 0,
            weaknesses: 0,
            cancels: 0,
            cancels_element: 0,
            initial: 0,
            permanent: 0,
            chance_to_add: 0,
            chance_to_cancel: 0,
            immune_to: 0,
        };

        equipment
            .strengthens
            .iter()
            .for_each(|el| new_equipment.strengthens |= Element::parse(el).unwrap().flag());
        equipment
            .absorbs
            .iter()
            .for_each(|el| new_equipment.absorbs |= Element::parse(el).unwrap().flag());
        equipment
            .weaknesses
            .iter()
            .for_each(|el| new_equipment.weaknesses |= Element::parse(el).unwrap().flag());
        equipment
            .cancels_element
            .iter()
            .for_each(|el| new_equipment.cancels_element |= Element::parse(el).unwrap().flag());

        equipment
            .initial
            .iter()
            .for_each(|cond| new_equipment.initial |= Condition::parse(cond).unwrap().flag());
        equipment
            .permanent
            .iter()
            .for_each(|cond| new_equipment.permanent |= Condition::parse(cond).unwrap().flag());
        equipment
            .cancels
            .iter()
            .for_each(|cond| new_equipment.cancels |= Condition::parse(cond).unwrap().flag());
        equipment
            .chance_to_add
            .iter()
            .for_each(|cond| new_equipment.chance_to_add |= Condition::parse(cond).unwrap().flag());
        equipment.chance_to_cancel.iter().for_each(|cond| {
            new_equipment.chance_to_cancel |= Condition::parse(cond).unwrap().flag()
        });
        equipment
            .immune_to
            .iter()
            .for_each(|cond| new_equipment.immune_to |= Condition::parse(cond).unwrap().flag());

        new_equipment
    }
}

#[derive(Serialize, Deserialize)]
pub struct BaseStatsData {
    pub by_job_gender: HashMap<(String, Gender), BaseStats>,
}

impl BaseStatsData {
    pub fn from_python(base_stats_data: python::BaseStatsData) -> BaseStatsData {
        BaseStatsData {
            by_job_gender: base_stats_data
                .by_job_gender
                .into_iter()
                .map(|(k, v)| {
                    let split: Vec<_> = k.split(",").collect();
                    let gender = Gender::parse(&split[1]).unwrap();
                    let key = (split[0].to_owned(), gender);
                    (key, BaseStats::from_python(v))
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BaseStats {
    pub job: String,
    pub gender: Gender,
    pub hp: i16,
    pub mp: i16,
    pub movement: i8,
    pub jump: i8,
    pub speed: i8,
    pub pa: i8,
    pub ma: i8,
    pub c_ev: i8,
    pub innates: Vec<String>,
    pub innate_conditions: ConditionFlags,
    pub skills: Vec<String>,
    pub absorbs: ElementFlags,
    pub halves: ElementFlags,
    pub weaknesses: ElementFlags,
    pub cancels: ElementFlags,
}

impl BaseStats {
    pub fn from_python(base_stats: python::BaseStats) -> BaseStats {
        let mut new_base_stats = BaseStats {
            job: base_stats.job,
            gender: Gender::parse(&base_stats.gender).unwrap(),
            hp: base_stats.hp,
            mp: base_stats.mp,
            movement: base_stats.movement,
            jump: base_stats.jump,
            speed: base_stats.speed,
            pa: base_stats.pa,
            ma: base_stats.ma,
            c_ev: base_stats.c_ev,
            innates: base_stats.innates,
            innate_conditions: 0,
            skills: base_stats.skills,
            absorbs: 0,
            halves: 0,
            weaknesses: 0,
            cancels: 0,
        };

        let mut innate_conditions = 0;
        new_base_stats
            .innates
            .iter()
            .flat_map(|el| Condition::parse(&el))
            .for_each(|cond| innate_conditions |= cond.flag());
        new_base_stats.innate_conditions = innate_conditions;

        base_stats
            .absorbs
            .iter()
            .for_each(|el| new_base_stats.absorbs |= Element::parse(el).unwrap().flag());
        base_stats
            .halves
            .iter()
            .for_each(|el| new_base_stats.halves |= Element::parse(el).unwrap().flag());
        base_stats
            .weaknesses
            .iter()
            .for_each(|el| new_base_stats.weaknesses |= Element::parse(el).unwrap().flag());
        base_stats
            .cancels
            .iter()
            .for_each(|el| new_base_stats.cancels |= Element::parse(el).unwrap().flag());

        new_base_stats
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StartingLocation {
    pub x: u8,
    pub y: u8,
    pub facing: Facing,
    pub left_team: bool,
    pub unit: u8,
    pub layer: bool,
}

impl StartingLocation {
    pub fn from_python(arena: python::StartingLocation) -> StartingLocation {
        StartingLocation {
            x: arena.x,
            y: arena.y,
            facing: match arena.facing.as_str() {
                "North" => Facing::North,
                "East" => Facing::East,
                "South" => Facing::South,
                "West" => Facing::West,
                _ => panic!("unknown facing"),
            },
            left_team: arena.team == "Player 1",
            unit: arena.unit,
            layer: arena.layer,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Arena {
    pub lower: Vec<Tile>,
    pub upper: Vec<Tile>,
    pub width: u8,
    pub height: u8,
    pub starting_locations: Vec<StartingLocation>,
}

impl Arena {
    pub fn from_python(arena: python::Arena) -> Arena {
        let mut new_arena = Arena {
            lower: vec![],
            upper: vec![],
            width: arena.width,
            height: arena.height,
            starting_locations: arena
                .starting_locations
                .into_iter()
                .map(StartingLocation::from_python)
                .collect(),
        };
        for y in 0..arena.height as usize {
            for x in 0..arena.width as usize {
                new_arena.lower.push(Tile::from_python(&arena.lower[y][x]));
                new_arena.upper.push(Tile::from_python(&arena.upper[y][x]));
            }
        }
        new_arena
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Tile {
    pub height: u8,
    pub depth: u8,
    pub slope_type: u8,
    pub surface_type: u8,
    pub slope_height: u8,
    pub no_cursor: bool,
    pub no_walk: bool,
}

impl Tile {
    pub fn from_python(tile: &python::Tile) -> Tile {
        Tile {
            height: tile.height,
            depth: tile.depth,
            slope_type: tile.slope_type_numeric,
            surface_type: tile.surface_type_numeric,
            slope_height: tile.slope_height,
            no_cursor: tile.no_cursor,
            no_walk: tile.no_walk,
        }
    }
}
