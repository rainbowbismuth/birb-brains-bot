import re
from dataclasses import dataclass
from typing import Tuple

import fftbg.config as config

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

ABSORB_RE = re.compile(r'Absorb-([\w&]+)')
HALF_RE = re.compile(r'Absorb-([\w&]+)')
WEAK_RE = re.compile(r'Weak-([\w&]+)')
CANCEL_RE = re.compile(r'Cancel-([\w&]+)')


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
    absorbs: Tuple[str]
    halves: Tuple[str]
    weaknesses: Tuple[str]
    cancels: Tuple[str]


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

        absorbs = element_match(ABSORB_RE, class_job)
        halves = element_match(HALF_RE, class_job)
        weaknesses = element_match(WEAK_RE, class_job)
        cancels = element_match(CANCEL_RE, class_job)

        BASE_STATS_MAP[(job, gender)] = BaseStats(
            job, gender, hp, mp, move, jump, speed, pa, ma, c_ev,
            absorbs, halves, weaknesses, cancels)


def element_match(regex, s):
    result = tuple()
    match = regex.findall(s)
    if match:
        result = tuple(match[0].split('&'))
    return result


def get_base_stats(job: str, gender: str) -> BaseStats:
    job = job.replace(' ', '')
    if not BASE_STATS_MAP:
        parse_base_stats()
    return BASE_STATS_MAP[(job, gender)]


if __name__ == '__main__':
    parse_base_stats()
    for bs in BASE_STATS_MAP.values():
        print(bs)
