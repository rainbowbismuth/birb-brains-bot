use crate::sim::enums::*;

#[derive(Copy, Clone, Debug)]
pub struct ConditionBlock {
    pub innate_flags: u64,
    pub status_flags: u64,
    pub timed_conditions: [i8; TIMED_CONDITIONS_LEN],
}

impl ConditionBlock {
    #[cfg(test)]
    pub fn new() -> ConditionBlock {
        ConditionBlock {
            innate_flags: 0,
            status_flags: 0,
            timed_conditions: [0; TIMED_CONDITIONS_LEN],
        }
    }

    pub fn new_with_innate(innate_flags: ConditionFlags) -> ConditionBlock {
        ConditionBlock {
            innate_flags,
            status_flags: 0,
            timed_conditions: [0; TIMED_CONDITIONS_LEN],
        }
    }

    pub fn add(&mut self, condition: Condition) {
        if let Some(duration) = condition.condition_duration() {
            if self.innate_flags & condition.flag() != 0 {
                return;
            }
            self.timed_conditions[condition.index()] = duration;
        }
        self.status_flags |= condition.flag();
    }

    pub fn has(&self, condition: Condition) -> bool {
        (self.innate_flags | self.status_flags) & condition.flag() != 0
    }

    pub fn tick(&mut self, condition: Condition) -> Option<bool> {
        if condition.is_timed_condition() {
            let count = self.timed_conditions[condition.index()];
            if count > 0 {
                self.timed_conditions[condition.index()] -= 1;
            }
            let to_remove = count == 1;
            if to_remove {
                self.status_flags &= !condition.flag();
            }
            Some(to_remove)
        } else {
            None
        }
    }

    pub fn remove(&mut self, condition: Condition) {
        if condition.is_timed_condition() {
            self.timed_conditions[condition.index()] = 0;
        }
        self.status_flags &= !condition.flag();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn durations_only_for_timed_conditions() {
        for condition in &TIMED_CONDITIONS {
            if condition.is_timed_condition() {
                assert!(condition.condition_duration().is_some());
            } else {
                assert!(condition.condition_duration().is_none());
            }
        }
    }

    #[test]
    pub fn can_add_remove_any_condition() {
        let mut block = ConditionBlock::new();
        for condition in &TIMED_CONDITIONS {
            assert!(!block.has(*condition));
            block.add(*condition);
            assert!(block.has(*condition));
            block.remove(*condition);
            assert!(!block.has(*condition));
        }
    }

    #[test]
    pub fn tick_condition_status_until_removal() {
        for condition in &TIMED_CONDITIONS {
            let mut block = ConditionBlock::new();
            block.add(*condition);
            for _ in 0..condition.condition_duration().unwrap() - 1 {
                assert_eq!(block.tick(*condition), Some(false));
                assert!(block.has(*condition));
            }
            assert_eq!(block.tick(*condition), Some(true));
            assert!(!block.has(*condition));
        }
    }
}
