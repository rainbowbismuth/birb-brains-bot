STOP = 'Stop'
SLEEP = 'Sleep'
SLOW = 'Slow'
SHELL = 'Shell'
REGEN = 'Regen'
REFLECT = 'Reflect'
PROTECT = 'Protect'
POISON = 'Poison'
INNOCENT = 'Innocent'
HASTE = 'Haste'
FAITH = 'Faith'
DONT_MOVE = "Don't Move"
DONT_ACT = "Don't Act"
CHARM = 'Charm'

CHICKEN = 'Chicken'
FROG = 'Frog'
CHARGING = 'Charging'
BERSERK = 'Berserk'

PETRIFY = 'Petrify'
JUMPING = 'Jumping'

TIME_STATUS_LENGTHS = {
    CHARM: 32,
    DONT_ACT: 24,
    DONT_MOVE: 24,
    FAITH: 32,
    HASTE: 32,
    INNOCENT: 32,
    POISON: 36,
    PROTECT: 32,
    REFLECT: 32,
    REGEN: 36,
    SHELL: 32,
    SLOW: 24,
    SLEEP: 60,
    STOP: 20,
}

TIME_STATUS_LEN = len(TIME_STATUS_LENGTHS)
TIME_STATUS_INDEX = dict([(k, i) for i, k in enumerate(TIME_STATUS_LENGTHS.keys())])
