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
TIME_STATUS_INDEX_REVERSE = dict([(i, k) for i, k in enumerate(TIME_STATUS_LENGTHS.keys())])

BLOOD_SUCK = 'Blood Suck'
CONFUSION = 'Confusion'
CRITICAL = 'Critical'
DEATH_SENTENCE = 'Death Sentence'
DEFENDING = 'Defending'
FLOAT = 'Float'
PERFORMING = 'Performing'
TRANSPARENT = 'Transparent'

DAMAGE_CANCELS = (CHARM, CONFUSION)
DEATH_CANCELS = (BERSERK, BLOOD_SUCK, CONFUSION, CHARM, CHARGING, DEATH_SENTENCE,
                 DEFENDING, DONT_MOVE, DONT_ACT, FAITH, FLOAT, HASTE, INNOCENT, PERFORMING,
                 POISON, PROTECT, REFLECT, REGEN, SHELL, SLOW, STOP, TRANSPARENT)
