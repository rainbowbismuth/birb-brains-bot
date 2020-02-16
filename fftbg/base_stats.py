import re
from dataclasses import dataclass

import config

BASE_STATS_MAP = {}

HUMAN_RE = re.compile(r"^(\w+) (\w+)'s base stats")
MONSTER_RE = re.compile(r"^(.+)'s base stats")

HP_RE = re.compile(r"(\d+) HP")
MP_RE = re.compile(r"(\d+) MP")
MOVE_RE = re.compile(r"(\d+) Move")
JUMP_RE = re.compile(r"(\d+) Jump")
SPEED_RE = re.compile(r"(\d+) Speed")
PA_RE = re.compile(r"(\d+) PA")
MA_RE = re.compile(r"(\d+) MA")
CEV_RE = re.compile(r"(\d+)% C-EV")


@dataclass(frozen=True)
class BaseStats:
    job: str
    gender: str
    hp: int
    mp: int
    move: int
    jump: int
    speed: int
    pa: int
    ma: int
    c_ev: int


def parse_base_stats():
    class_jobs = config.CLASS_HELP_PATH.read_text().splitlines()
    for class_job in class_jobs:
        human_match = HUMAN_RE.match(class_job)
        if human_match:
            job, gender = human_match.groups()
        else:
            monster_match = MONSTER_RE.match(class_job)
            job = monster_match[1]
            gender = 'Monster'
        if gender == 'Eye':
            job = 'FloatingEye'
            gender = 'Monster'

        hp = int(HP_RE.findall(class_job)[0])
        mp = int(MP_RE.findall(class_job)[0])
        move = int(MOVE_RE.findall(class_job)[0])
        jump = int(JUMP_RE.findall(class_job)[0])
        speed = int(SPEED_RE.findall(class_job)[0])
        pa = int(PA_RE.findall(class_job)[0])
        ma = int(MA_RE.findall(class_job)[0])
        c_ev = int(CEV_RE.findall(class_job)[0])

        BASE_STATS_MAP[(job, gender)] = BaseStats(
            job, gender, hp, mp, move, jump, speed, pa, ma, c_ev)


def get_base_stats(job: str, gender: str) -> BaseStats:
    job = job.replace(' ', '')
    if not BASE_STATS_MAP:
        parse_base_stats()
    return BASE_STATS_MAP[(job, gender)]
