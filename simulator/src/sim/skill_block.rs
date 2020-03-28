const ABANDON_FLAG: u64 = 1 << 0;
const PARRY_FLAG: u64 = 1 << 1;
const BLADE_GRASP_FLAG: u64 = 1 << 2;
const CONCENTRATE_FLAG: u64 = 1 << 3;
const DUAL_WIELD_FLAG: u64 = 1 << 4;
const DOUBLE_HAND_FLAG: u64 = 1 << 5;
const MARTIAL_ARTS_FLAG: u64 = 1 << 6;
const ATTACK_UP_FLAG: u64 = 1 << 7;
const DEFENSE_UP_FLAG: u64 = 1 << 8;

#[derive(Clone, Copy, Debug)]
pub struct SkillBlock {
    flags: u64,
}

impl SkillBlock {
    pub fn new(skills: &[&String]) -> SkillBlock {
        let mut block = SkillBlock { flags: 0 };
        for skill in skills {
            match skill.as_str() {
                "Abandon" => block.flags |= ABANDON_FLAG,
                "Parry" => block.flags |= PARRY_FLAG,
                "Blade Grasp" => block.flags |= BLADE_GRASP_FLAG,
                "Concentrate" => block.flags |= CONCENTRATE_FLAG,
                "Dual Wield" => block.flags |= DUAL_WIELD_FLAG,
                "Doublehand" => block.flags |= DOUBLE_HAND_FLAG,
                "Martial Arts" => block.flags |= MARTIAL_ARTS_FLAG,
                "Attack UP" => block.flags |= ATTACK_UP_FLAG,
                "Defense UP" => block.flags |= DEFENSE_UP_FLAG,
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
}