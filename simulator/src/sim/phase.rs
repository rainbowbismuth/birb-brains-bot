use crate::sim::CombatantId;

#[derive(Copy, Clone)]
pub enum Phase {
    StatusCheck,
    SlowActionCharging,
    SlowAction(CombatantId),
    CtCharging,
    ActiveTurn(CombatantId),
}