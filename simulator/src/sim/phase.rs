use crate::sim::CombatantId;

#[derive(Copy, Clone, Debug)]
pub enum Phase {
    StatusCheck,
    SlowActionCharging,
    SlowAction(CombatantId),
    CtCharging,
    ActiveTurn(CombatantId),
}