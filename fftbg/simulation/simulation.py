import logging
import random
from pathlib import Path
from typing import List

import fftbg.arena
import fftbg.patch
import fftbg.server
import fftbg.tournament
from fftbg.arena import Arena
from fftbg.equipment import Equipment
from fftbg.simulation.combatant import Combatant
from fftbg.simulation.commands import attack
from fftbg.simulation.status import TIME_STATUS_LEN, TIME_STATUS_INDEX_REVERSE, DAMAGE_CANCELS, DEATH_CANCELS

LOG = logging.getLogger(__name__)


class Simulation:
    def __init__(self, combatants: List[Combatant], arena: Arena):
        self.combatants = combatants
        self.arena = arena
        self.clock_tick: int = 0
        self.last_action = None  # TODO: mime cycle
        self.slow_actions = []
        self.active_turns = []
        self.left_wins = None
        self.prepend = ''

    def run(self):
        while self.left_wins is None:
            self.tick()
            if not self.team_healthy(0):
                self.left_wins = False
            if not self.team_healthy(1):
                self.left_wins = True

    def team_healthy(self, team: int):
        return any([combatant.healthy for combatant in self.combatants if combatant.team == team])

    def tick(self):
        self.prepend = ''
        self.phase_status_check()

        self.phase_slow_action_charging()
        if self.slow_actions:
            self.phase_slow_action_resolve()

        self.phase_ct_charging()
        if self.active_turns:
            self.phase_active_turn_resolve()

    def prepend_info(self):
        return f'{self.clock_tick:4d}:{self.prepend:>20}'

    def report(self, s: str):
        LOG.info(f'{self.prepend_info()}: {s}')

    def unit_report(self, combatant: Combatant, s: str):
        LOG.info(f'{self.prepend_info()}: {combatant.name:>15} ({combatant.hp:3d} HP) {s}')

    def phase_status_check(self):
        self.clock_tick += 1
        self.prepend = 'Status Check'
        for combatant in self.combatants:
            for i in range(TIME_STATUS_LEN):
                has_condition = combatant.timed_status_conditions[i] > 0
                if has_condition:
                    combatant.timed_status_conditions[i] -= 1
                if has_condition and combatant.timed_status_conditions[i] == 0:
                    condition = TIME_STATUS_INDEX_REVERSE[i]
                    self.unit_report(combatant, f'no longer has {condition}')

    def phase_slow_action_charging(self):
        self.prepend = 'Slow Action Charge'
        for combatant in self.combatants:
            if not combatant.ctr_action:
                continue
            combatant.ctr -= 1
            if combatant.ctr <= 0:
                self.slow_actions.append(combatant)

    def phase_slow_action_resolve(self):
        self.prepend = 'Slow Action Resolve'
        while self.slow_actions:
            combatant = self.slow_actions.pop(0)
            if not combatant.healthy:
                continue
            self.prepend = f'{combatant.name}\'s C'
            action = combatant.ctr_action
            combatant.ctr_action = None
            action()

    def phase_ct_charging(self):
        self.prepend = 'CT Charging'
        for combatant in self.combatants:
            speed = combatant.speed

            if combatant.haste:
                speed = (speed * 3) // 2
            if combatant.slow:
                speed = (speed * 2) // 3

            combatant.ct += speed

            if combatant.ct >= 100:
                self.active_turns.append(combatant)

    def phase_active_turn_resolve(self):
        self.prepend = 'Active Turn Resolve'
        while self.active_turns:
            combatant = self.active_turns.pop(0)
            if not combatant.healthy:
                continue

            self.prepend = f'{combatant.name}\'s AT'

            if combatant.regen:
                self.change_target_hp(combatant, -(combatant.max_hp // 8), 'regen')

            self.ai_do_basic_turn(combatant)

            if combatant.poison:
                self.change_target_hp(combatant, combatant.max_hp // 8, 'poison')

    def set_ct_from_acting(self, user: Combatant, amount: int):
        new_amount = min(60, user.ct + amount)
        user.ct = new_amount

    def ai_can_be_cowardly(self, user: Combatant):
        return any([c.healthy for c in self.combatants if user.team == c.team and user is not c])

    def ai_do_basic_turn(self, user: Combatant):
        if user.critical and self.ai_can_be_cowardly(user):
            self.unit_report(user, 'is cowardly skipping their turn')
            self.set_ct_from_acting(user, -80)
            return

        targets = [target for target in self.combatants if user.is_foe(target) and target.healthy]
        if not targets:
            self.set_ct_from_acting(user, -80)
            return

        # going to do an ATTACK
        target = random.choice(targets)
        self.do_cmd_attack(user, target)

    def do_cmd_attack(self, user: Combatant, target: Combatant):
        damage_1, crit = attack.damage(user, user.mainhand, target)
        self.change_target_hp(target, damage_1, f'{user.name}\'s mainhand ATTACK')
        self.chance_to_add_status(user, user.mainhand, target)

        # TODO: Assuming critical hits always make your next attack miss
        if user.dual_wield and not crit:
            damage_2, _ = attack.damage(user, user.offhand, target)
            self.change_target_hp(target, damage_2, f'{user.name}\'s offhand ATTACK')
            self.chance_to_add_status(user, user.offhand, target)

        self.set_ct_from_acting(user, -100)

    def add_status(self, target: Combatant, status: str, src: str):
        had_status = target.has_status(status)
        target.add_status(status)
        if not had_status:
            self.unit_report(target, f'now has {status} from {src}')

    def cancel_status(self, target: Combatant, status: str, src: str):
        if not target.has_status(status):
            return
        target.cancel_status(status)
        self.unit_report(target, f'had {status} cancelled by {src}')

    def chance_to_add_status(self, user: Combatant, weapon: Equipment, target: Combatant):
        if not target.healthy:
            return  # FIXME: this doesn't strictly make sense I don't think...

        if not (weapon.chance_to_add or weapon.chance_to_cancel):
            return

        for status in weapon.chance_to_add:
            if random.random() >= 0.19:
                continue
            self.add_status(target, status, f'{user.name}\'s {weapon.name}')

        for status in weapon.chance_to_cancel:
            if random.random() >= 0.19:
                continue
            self.cancel_status(target, status, f'{user.name}\'s {weapon.name}')

    def change_target_hp(self, target: Combatant, amount, source: str):
        if not target.healthy:
            return
        target.hp = min(target.max_hp, max(0, target.hp - amount))
        if amount >= 0:
            self.unit_report(target, f'took {amount:3d} damage from {source}')
            for status in DAMAGE_CANCELS:
                self.cancel_status(target, status, source)
        else:
            self.unit_report(target, f'was healed for {amount:3d} from {source}')
        if target.hp == 0:
            self.target_died(target)

    def target_died(self, target: Combatant):
        self.unit_report(target, 'died')
        for status in DEATH_CANCELS:
            self.cancel_status(target, status, 'death')


def main():
    fftbg.server.configure_logging('SIMULATION_LOG_LEVEL')
    tourny = fftbg.tournament.parse_tournament(Path('data/tournaments/1584818551017.json'))
    match_up = tourny.match_ups[0]
    patch = fftbg.patch.get_patch(tourny.modified)
    combatants = []
    for d in match_up.left.combatants:
        combatants.append(Combatant(d, patch, 0))
    for d in match_up.right.combatants:
        combatants.append(Combatant(d, patch, 1))
    arena = fftbg.arena.get_arena(match_up.game_map)
    sim = Simulation(combatants, arena)
    sim.run()
    if sim.left_wins:
        LOG.info('Left team wins!')
    else:
        LOG.info('Right team wins!')


if __name__ == '__main__':
    main()
