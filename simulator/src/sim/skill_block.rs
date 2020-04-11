const ABANDON_FLAG: u64 = 1 << 0;
const PARRY_FLAG: u64 = 1 << 1;
const BLADE_GRASP_FLAG: u64 = 1 << 2;
const CONCENTRATE_FLAG: u64 = 1 << 3;
const DUAL_WIELD_FLAG: u64 = 1 << 4;
const DOUBLE_HAND_FLAG: u64 = 1 << 5;
const MARTIAL_ARTS_FLAG: u64 = 1 << 6;
const ATTACK_UP_FLAG: u64 = 1 << 7;
const DEFENSE_UP_FLAG: u64 = 1 << 8;
// const INNATE_UNDEAD_FLAG: u64 = 1 << 9;
const COUNTER_FLAG: u64 = 1 << 10;
const MOVE_HP_UP_FLAG: u64 = 1 << 11;
const MOVE_MP_UP_FLAG: u64 = 1 << 12;
const SICKEN_FLAG: u64 = 1 << 13;
const MANA_SHIELD_FLAG: u64 = 1 << 14;
const DAMAGE_SPLIT_FLAG: u64 = 1 << 15;
const AUTO_POTION_FLAG: u64 = 1 << 16;
const THROW_ITEM_FLAG: u64 = 1 << 17;
const MAGIC_ATTACK_UP: u64 = 1 << 18;
const MAGIC_DEFENSE_UP: u64 = 1 << 19;
const SHORT_CHARGE: u64 = 1 << 20;
const HALVE_MP: u64 = 1 << 21;
const REGENERATOR: u64 = 1 << 22;
const PA_SAVE: u64 = 1 << 23;
const MA_SAVE: u64 = 1 << 24;
const SPEED_SAVE: u64 = 1 << 25;
const DRAGON_SPIRIT: u64 = 1 << 26;
const RETREAT: u64 = 1 << 27;
const HP_RESTORE: u64 = 1 << 28;
const MP_RESTORE: u64 = 1 << 29;
const CRITICAL_QUICK: u64 = 1 << 30;
const MIMIC: u64 = 1 << 31;
const NO_MP: u64 = 1 << 32;
const CAUTION: u64 = 1 << 33;

#[derive(Clone, Copy, Debug)]
pub struct SkillBlock {
    flags: u64,
}

impl SkillBlock {
    pub fn new(skills: &[&str]) -> SkillBlock {
        let mut block = SkillBlock { flags: 0 };
        for skill in skills {
            match *skill {
                "Abandon" => block.flags |= ABANDON_FLAG,
                "Parry" => block.flags |= PARRY_FLAG,
                "Blade Grasp" => block.flags |= BLADE_GRASP_FLAG,
                "Concentrate" => block.flags |= CONCENTRATE_FLAG,
                "Dual Wield" => block.flags |= DUAL_WIELD_FLAG,
                "Doublehand" => block.flags |= DOUBLE_HAND_FLAG,
                "Martial Arts" => block.flags |= MARTIAL_ARTS_FLAG,
                "Attack UP" => block.flags |= ATTACK_UP_FLAG,
                "Defense UP" => block.flags |= DEFENSE_UP_FLAG,
                "Counter" => block.flags |= COUNTER_FLAG,
                "Move-HP Up" => block.flags |= MOVE_HP_UP_FLAG,
                "Move-MP Up" => block.flags |= MOVE_MP_UP_FLAG,
                "Sicken" => block.flags |= SICKEN_FLAG,
                "Mana Shield" => block.flags |= MANA_SHIELD_FLAG,
                "Damage Split" => block.flags |= DAMAGE_SPLIT_FLAG,
                "Auto Potion" => block.flags |= AUTO_POTION_FLAG,
                "Throw Item" => block.flags |= THROW_ITEM_FLAG,
                "Magic Attack UP" => block.flags |= MAGIC_ATTACK_UP,
                "Magic Defense UP" => block.flags |= MAGIC_DEFENSE_UP,
                "Short Charge" => block.flags |= SHORT_CHARGE,
                "Halve MP" => block.flags |= HALVE_MP,
                "Regenerator" => block.flags |= REGENERATOR,
                "PA Save" => block.flags |= PA_SAVE,
                "MA Save" => block.flags |= MA_SAVE,
                "Speed Save" => block.flags |= SPEED_SAVE,
                "Dragon Spirit" => block.flags |= DRAGON_SPIRIT,
                "Retreat" => block.flags |= RETREAT,
                "HP Restore" => block.flags |= HP_RESTORE,
                "MP Restore" => block.flags |= MP_RESTORE,
                "Critical Quick" => block.flags |= CRITICAL_QUICK,
                "Mimic" => block.flags |= MIMIC,
                "No MP" => block.flags |= NO_MP,
                "Caution" => block.flags |= CAUTION,
                _ => {}
            }
        }
        block
    }

    pub fn abandon(&self) -> bool {
        self.flags & ABANDON_FLAG != 0
    }

    pub fn parry(&self) -> bool {
        self.flags & PARRY_FLAG != 0
    }

    pub fn blade_grasp(&self) -> bool {
        self.flags & BLADE_GRASP_FLAG != 0
    }

    pub fn concentrate(&self) -> bool {
        self.flags & CONCENTRATE_FLAG != 0
    }

    pub fn dual_wield(&self) -> bool {
        self.flags & DUAL_WIELD_FLAG != 0
    }

    pub fn double_hand(&self) -> bool {
        self.flags & DOUBLE_HAND_FLAG != 0
    }

    pub fn martial_arts(&self) -> bool {
        self.flags & MARTIAL_ARTS_FLAG != 0
    }

    pub fn attack_up(&self) -> bool {
        self.flags & ATTACK_UP_FLAG != 0
    }

    pub fn defense_up(&self) -> bool {
        self.flags & DEFENSE_UP_FLAG != 0
    }

    pub fn counter(&self) -> bool {
        self.flags & COUNTER_FLAG != 0
    }

    pub fn move_hp_up(&self) -> bool {
        self.flags & MOVE_HP_UP_FLAG != 0
    }

    pub fn move_mp_up(&self) -> bool {
        self.flags & MOVE_MP_UP_FLAG != 0
    }

    pub fn sicken(&self) -> bool {
        self.flags & SICKEN_FLAG != 0
    }

    pub fn mana_shield(&self) -> bool {
        self.flags & MANA_SHIELD_FLAG != 0
    }

    pub fn damage_split(&self) -> bool {
        self.flags & DAMAGE_SPLIT_FLAG != 0
    }

    pub fn auto_potion(&self) -> bool {
        self.flags & AUTO_POTION_FLAG != 0
    }

    pub fn throw_item(&self) -> bool {
        self.flags & THROW_ITEM_FLAG != 0
    }

    pub fn magic_attack_up(&self) -> bool {
        self.flags & MAGIC_ATTACK_UP != 0
    }

    pub fn magic_defense_up(&self) -> bool {
        self.flags & MAGIC_DEFENSE_UP != 0
    }

    pub fn short_charge(&self) -> bool {
        self.flags & SHORT_CHARGE != 0
    }

    pub fn halve_mp(&self) -> bool {
        self.flags & HALVE_MP != 0
    }

    pub fn regenerator(&self) -> bool {
        self.flags & REGENERATOR != 0
    }

    pub fn pa_save(&self) -> bool {
        self.flags & PA_SAVE != 0
    }

    pub fn ma_save(&self) -> bool {
        self.flags & MA_SAVE != 0
    }

    pub fn speed_save(&self) -> bool {
        self.flags & SPEED_SAVE != 0
    }

    pub fn dragon_spirit(&self) -> bool {
        self.flags & DRAGON_SPIRIT != 0
    }

    pub fn retreat(&self) -> bool {
        self.flags & RETREAT != 0
    }

    pub fn hp_restore(&self) -> bool {
        self.flags & HP_RESTORE != 0
    }

    pub fn mp_restore(&self) -> bool {
        self.flags & MP_RESTORE != 0
    }

    pub fn critical_quick(&self) -> bool {
        self.flags & CRITICAL_QUICK != 0
    }

    pub fn mimic(&self) -> bool {
        self.flags & MIMIC != 0
    }

    pub fn no_mp(&self) -> bool {
        self.flags & NO_MP != 0
    }

    pub fn caution(&self) -> bool {
        self.flags & CAUTION != 0
    }
}
