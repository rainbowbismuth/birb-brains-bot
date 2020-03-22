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
UNDEAD = 'Undead'
SILENCE = 'Silence'
OIL = 'Oil'
RERAISE = 'Reraise'
WALL = 'Wall'

DARKNESS = 'Darkness'
DEATH = 'Death'

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

DAMAGE_CANCELS = (CHARM, CONFUSION, SLEEP)
DEATH_CANCELS = (BERSERK, BLOOD_SUCK, CONFUSION, CHARM, CHARGING, DEATH_SENTENCE,
                 DEFENDING, DONT_MOVE, DONT_ACT, FAITH, FLOAT, HASTE, INNOCENT, PERFORMING,
                 POISON, PROTECT, REFLECT, REGEN, SHELL, SLOW, STOP, TRANSPARENT)

ALL_CONDITIONS = sorted([STOP,
                         SLEEP,
                         SLOW,
                         SHELL,
                         REGEN,
                         REFLECT,
                         PROTECT,
                         POISON,
                         INNOCENT,
                         HASTE,
                         FAITH,
                         DONT_MOVE,
                         DONT_ACT,
                         CHARM,
                         CHICKEN,
                         FROG,
                         CHARGING,
                         BERSERK,
                         PETRIFY,
                         JUMPING,
                         BLOOD_SUCK,
                         CONFUSION,
                         CRITICAL,
                         DEATH_SENTENCE,
                         DEFENDING,
                         FLOAT,
                         PERFORMING,
                         TRANSPARENT,
                         PETRIFY,
                         JUMPING,
                         UNDEAD,
                         SILENCE,
                         OIL,
                         RERAISE,
                         WALL,
                         DARKNESS,
                         DEATH
                         ])

assert len(ALL_CONDITIONS) < 64

STATUS_FLAGS = {}
for i, status in enumerate(ALL_CONDITIONS):
    STATUS_FLAGS[status] = 1 << i

STOP_FLAG = STATUS_FLAGS[STOP]
SLEEP_FLAG = STATUS_FLAGS[SLEEP]
SLOW_FLAG = STATUS_FLAGS[SLOW]
SHELL_FLAG = STATUS_FLAGS[SHELL]
REGEN_FLAG = STATUS_FLAGS[REGEN]
REFLECT_FLAG = STATUS_FLAGS[REFLECT]
PROTECT_FLAG = STATUS_FLAGS[PROTECT]
POISON_FLAG = STATUS_FLAGS[POISON]
INNOCENT_FLAG = STATUS_FLAGS[INNOCENT]
HASTE_FLAG = STATUS_FLAGS[HASTE]
FAITH_FLAG = STATUS_FLAGS[FAITH]
DONT_MOVE_FLAG = STATUS_FLAGS[DONT_MOVE]
DONT_ACT_FLAG = STATUS_FLAGS[DONT_ACT]
CHARM_FLAG = STATUS_FLAGS[CHARM]
CHICKEN_FLAG = STATUS_FLAGS[CHICKEN]
FROG_FLAG = STATUS_FLAGS[FROG]
CHARGING_FLAG = STATUS_FLAGS[CHARGING]
BERSERK_FLAG = STATUS_FLAGS[BERSERK]
PETRIFY_FLAG = STATUS_FLAGS[PETRIFY]
JUMPING_FLAG = STATUS_FLAGS[JUMPING]
BLOOD_SUCK_FLAG = STATUS_FLAGS[BLOOD_SUCK]
CONFUSION_FLAG = STATUS_FLAGS[CONFUSION]
CRITICAL_FLAG = STATUS_FLAGS[CRITICAL]
DEATH_SENTENCE_FLAG = STATUS_FLAGS[DEATH_SENTENCE]
DEFENDING_FLAG = STATUS_FLAGS[DEFENDING]
FLOAT_FLAG = STATUS_FLAGS[FLOAT]
PERFORMING_FLAG = STATUS_FLAGS[PERFORMING]
TRANSPARENT_FLAG = STATUS_FLAGS[TRANSPARENT]
PETRIFY_FLAG = STATUS_FLAGS[PETRIFY]
JUMPING_FLAG = STATUS_FLAGS[JUMPING]
UNDEAD_FLAG = STATUS_FLAGS[UNDEAD]
SILENCE_FLAG = STATUS_FLAGS[SILENCE]
OIL_FLAG = STATUS_FLAGS[OIL]
RERAISE_FLAG = STATUS_FLAGS[RERAISE]
WALL_FLAG = STATUS_FLAGS[WALL]
DARKNESS_FLAG = STATUS_FLAGS[DARKNESS]
DEATH_FLAG = STATUS_FLAGS[DEATH]
