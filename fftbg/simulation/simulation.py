import logging
import random
from pathlib import Path
from typing import List

import colored

import fftbg.arena
import fftbg.patch
import fftbg.server
import fftbg.tournament
from fftbg.arena import Arena
from fftbg.equipment import Equipment
from fftbg.simulation.combatant import Combatant
from fftbg.simulation.commands import attack
from fftbg.simulation.status import TIME_STATUS_LEN, TIME_STATUS_INDEX_REVERSE, DAMAGE_CANCELS, DEATH_CANCELS, \
    ALL_CONDITIONS, TRANSPARENT

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
        self.prepend = self.colored_phase('Init')

        # initialize location based on arena
        for combatant in self.combatants:
            if combatant.team == 0:
                combatant.location = int(-arena.max_dimension * arena.team_distance)
            else:
                combatant.location = int(arena.max_dimension * arena.team_distance)

        self.report(f'Fighting on {arena.name}')

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
        return f'CT {self.clock_tick}: {self.prepend}'

    def colored_phase(self, phase_name):
        return colored.stylize(phase_name, colored.fg("cyan"))

    def colored_name(self, combatant: Combatant):
        if combatant.team == 0:
            return colored.stylize(combatant.name, colored.fg("red"))
        else:
            return colored.stylize(combatant.name, colored.fg("blue"))

    def report(self, s: str):
        LOG.info(f'{self.prepend_info()}: {s}')

    def unit_report(self, combatant: Combatant, s: str):
        LOG.info(f'{self.prepend_info()}: {combatant.name} ({combatant.hp} HP) {s}')

    def phase_status_check(self):
        self.clock_tick += 1
        self.prepend = self.colored_phase('Status Check')
        for combatant in self.combatants:
            for i in range(TIME_STATUS_LEN):
                has_condition = combatant.timed_status_conditions[i] > 0
                if has_condition:
                    combatant.timed_status_conditions[i] -= 1
                if has_condition and combatant.timed_status_conditions[i] == 0:
                    condition = TIME_STATUS_INDEX_REVERSE[i]
                    self.unit_report(combatant, f'no longer has {condition}')

    def phase_slow_action_charging(self):
        self.prepend = self.colored_phase('Slow Action Charge')
        for combatant in self.combatants:
            if not combatant.ctr_action:
                continue

            if combatant.stop:
                continue  # FIXME: Does stop just remove the slow action?

            combatant.ctr -= 1
            if combatant.ctr <= 0:
                self.slow_actions.append(combatant)

    def phase_slow_action_resolve(self):
        self.prepend = self.colored_phase('Slow Action Resolve')
        while self.slow_actions:
            combatant = self.slow_actions.pop(0)
            if not combatant.healthy:
                continue
            self.prepend = f'{self.colored_name(combatant)}\'s C'
            action = combatant.ctr_action
            combatant.ctr_action = None
            action()

    def phase_ct_charging(self):
        self.prepend = self.colored_phase('CT Charging')
        for combatant in self.combatants:
            if combatant.stop or combatant.sleep:
                continue

            speed = combatant.speed

            if combatant.haste:
                speed = (speed * 3) // 2
            if combatant.slow:
                speed = (speed * 2) // 3

            combatant.ct += speed

            if combatant.ct >= 100:
                self.active_turns.append(combatant)

    def active_turn_status_bar(self, combatant: Combatant):
        conditions = []
        for condition in ALL_CONDITIONS:
            if combatant.has_status(condition):
                conditions.append(condition)
        condition_str = ', '.join(conditions)
        if condition_str:
            return f'{self.colored_name(combatant)} ({combatant.hp} HP, {condition_str})'
        else:
            return f'{self.colored_name(combatant)} ({combatant.hp} HP)'

    def clear_active_turn_flags(self):
        for combatant in self.combatants:
            combatant.on_active_turn = False
            combatant.moved_during_active_turn = False
            combatant.acted_during_active_turn = False
            combatant.took_damage_during_active_turn = False

    def end_of_active_turn_checks(self):
        for combatant in self.combatants:

            if combatant.acted_during_active_turn or combatant.took_damage_during_active_turn:
                combatant.cancel_status(TRANSPARENT)

            if combatant.on_active_turn:
                minus_ct = 60
                if combatant.moved_during_active_turn:
                    minus_ct += 20
                if combatant.acted_during_active_turn:
                    minus_ct += 20
                self.set_ct_from_acting(combatant, -minus_ct)

                if not (combatant.moved_during_active_turn or combatant.acted_during_active_turn):
                    self.unit_report(combatant, 'did nothing')

            combatant.on_active_turn = False

    def phase_active_turn_resolve(self):
        self.prepend = self.colored_phase('Active Turn Resolve')
        while self.active_turns:
            combatant = self.active_turns.pop(0)
            if not combatant.healthy:
                continue

            if combatant.stop or combatant.sleep:
                continue

            self.clear_active_turn_flags()
            combatant.on_active_turn = True

            self.prepend = self.active_turn_status_bar(combatant)

            if combatant.regen:
                self.change_target_hp(combatant, -(combatant.max_hp // 8), 'regen')

            self.ai_do_basic_turn(combatant)

            if combatant.poison:
                self.change_target_hp(combatant, combatant.max_hp // 8, 'poison')

            self.end_of_active_turn_checks()

    def set_ct_from_acting(self, user: Combatant, amount: int):
        new_amount = min(60, user.ct + amount)
        user.ct = new_amount

    def ai_thirteen_rule(self) -> bool:
        return random.random() <= 0.13

    def roll_brave_reaction(self, user: Combatant) -> bool:
        if user.berserk:
            return False
        return random.random() <= user.brave

    def ai_can_be_cowardly(self, user: Combatant):
        any_healthy = any([c.healthy for c in self.combatants if user.team == c.team and user is not c])
        all_critical = all([c.critical for c in self.combatants if user.team == c.team and user is not c])
        return any_healthy and not all_critical

    def can_move_into_range(self, user: Combatant, range: int, target: Combatant):
        return user.distance(target) <= range + user.move

    def do_move_with_bounds(self, user: Combatant, new_location: int):
        old_location = user.location
        user.location = max(-self.arena.max_dimension, min(new_location, self.arena.max_dimension))
        if old_location == user.location:
            return
        user.moved_during_active_turn = True
        self.report(f'moved to from {old_location} to {user.location}')

    def move_to_range(self, user: Combatant, range: int, target: Combatant):
        if user.moved_during_active_turn:
            return
        # TODO: Charm?
        if user.team == 0:
            desired = target.location - range
        else:
            desired = target.location + range

        if user.location == desired:
            return

        v = desired - user.location
        diff = min(user.move, abs(v))
        if v > 0:
            sign = 1
        else:
            sign = -1
        self.do_move_with_bounds(user, user.location + diff * sign)

    def move_into_combat(self, user: Combatant):
        if user.moved_during_active_turn:
            return
        if user.team == 0:
            self.do_move_with_bounds(user, user.location + user.move)
        else:
            self.do_move_with_bounds(user, user.location - user.move)

    def move_out_of_combat(self, user: Combatant):
        if user.moved_during_active_turn:
            return
        if user.team == 0:
            self.do_move_with_bounds(user, user.location - user.move)
        else:
            self.do_move_with_bounds(user, user.location + user.move)

    def ai_calculate_friendly_targets(self, user: Combatant):
        if user.confusion:
            all_targets = list(self.combatants)
            random.shuffle(all_targets)
            return all_targets
        if user.charm:
            return [target for target in self.combatants if user.is_foe(target)]
        return [target for target in self.combatants if user.is_friend(target)]

    def ai_calculate_enemy_targets(self, user: Combatant):
        if user.confusion:
            all_targets = list(self.combatants)
            random.shuffle(all_targets)
            return all_targets
        if user.charm:
            return [target for target in self.combatants if
                    user.is_friend(target) and target.healthy and target is not user]
        return [target for target in self.combatants if user.is_foe(target) and target.healthy]

    def ai_do_basic_turn(self, user: Combatant):
        friendly_targets = self.ai_calculate_friendly_targets(user)
        acting_cowardly = user.critical and self.ai_can_be_cowardly(user)
        if acting_cowardly:
            friendly_targets = [user]

        self.ai_try_friendly_action(user, friendly_targets)
        if user.acted_during_active_turn:
            self.move_out_of_combat(user)
            return
        elif not user.acted_during_active_turn and acting_cowardly:
            self.move_out_of_combat(user)
            return

        enemy_targets = self.ai_calculate_enemy_targets(user)
        if not enemy_targets:
            return

        self.ai_try_enemy_action(user, enemy_targets)
        if user.acted_during_active_turn:
            self.move_out_of_combat(user)
        else:
            self.move_into_combat(user)

    def ai_try_friendly_action(self, user: Combatant, targets: List[Combatant]):
        if user.berserk:
            return

        for target in targets:
            if not target.healthy:
                self.ai_try_raise(user, target)
                if user.acted_during_active_turn:
                    return
            if target.critical:
                self.ai_try_heal(user, target)
                if user.acted_during_active_turn:
                    return
        return

    def do_cmd_item_heal(self, user, item: str, target):
        range = 1
        if user.throw_item:
            range = 4
        if not self.can_move_into_range(user, range, target):
            return
        self.move_to_range(user, range, target)
        if item == 'Phoenix Down':
            heal_amount = random.randint(1, 20)
        elif item == 'Elixir':
            heal_amount = target.max_hp
        elif item == 'X-Potion':
            heal_amount = 150
        elif item == 'Hi-Potion':
            heal_amount = 120
        elif item == 'Potion':
            heal_amount = 100
        else:
            raise Exception(f'{item} isn\'t a known healing item')
        self.change_target_hp(target, -heal_amount, item)
        user.acted_during_active_turn = True

    def ai_try_raise(self, user, target):
        if user.berserk:
            return

        for ability in user.abilities:
            # FIXME: super hack rn
            if ability.name == 'Phoenix Down':
                if self.ai_thirteen_rule():
                    continue
                self.do_cmd_item_heal(user, ability.name, target)
                if user.acted_during_active_turn:
                    return

    def ai_try_heal(self, user, target):
        if user.berserk:
            return
        if not target.healthy:
            return

        for ability in user.abilities:
            # FIXME: super hack rn
            if ability.name in ('Elixir', 'X-Potion', 'Hi-Potion', 'Potion'):
                if self.ai_thirteen_rule():
                    continue
                self.do_cmd_item_heal(user, ability.name, target)
                if user.acted_during_active_turn:
                    return
        return

    def ai_try_enemy_action(self, user: Combatant, targets: List[Combatant]):
        # TODO: Skipping thirteen rule here for now.
        # if self.ai_thirteen_rule() or not targets:
        #     return
        if not targets:
            return

        # Target critical enemies first
        for target in targets:
            if not target.critical:
                continue

            if not self.can_move_into_range(user, user.mainhand.range, target):
                continue

            self.move_to_range(user, user.mainhand.range, target)
            self.do_cmd_attack(user, target)
            return

        # Otherwise target the tankiest enemy
        highest_hp = targets[0]
        for target in targets:
            if target.hp > highest_hp.hp:
                highest_hp = target

        self.move_to_range(user, user.mainhand.range, highest_hp)
        if self.in_range(user, user.mainhand.range, highest_hp):
            self.do_cmd_attack(user, highest_hp)

    def in_range(self, user: Combatant, range: int, target: Combatant):
        dist = user.distance(target)
        return dist <= range

    def do_cmd_attack(self, user: Combatant, target: Combatant):
        user.acted_during_active_turn = True
        damage, crit = self.do_single_weapon_attack(user, user.mainhand, target)
        if user.dual_wield and target.healthy:
            if crit and random.randint(1, 2) == 1:
                self.unit_report(target, 'was pushed out of range of a second attack')
            else:
                damage, crit = self.do_single_weapon_attack(user, user.offhand, target)
        if damage > 0:
            self.after_damage_reaction(target, user, damage)
        user.acted_during_active_turn = True

    def do_physical_evade(self, user: Combatant, weapon: Equipment, target: Combatant) -> bool:
        if user.transparent or user.concentrate:
            return False
        # TODO: Arrow Guard, etc?
        if random.random() < target.physical_accessory_evasion:
            self.unit_report(target, f'guarded {user.name}\'s attack')
            return True
        if random.random() < target.physical_shield_evasion / 2.0:
            self.unit_report(target, f'blocked {user.name}\'s attack')
            return True
        if random.random() < target.weapon_evasion / 2.0:
            self.unit_report(target, f'parried {user.name}\'s attack')
            return True
        if random.random() < target.class_evasion / 2.0:
            self.unit_report(target, f'evaded {user.name}\'s attack')
            return True
        return False

    def do_single_weapon_attack(self, user: Combatant, weapon: Equipment, target: Combatant) -> (int, bool):
        if not self.in_range(user, weapon.range, target):
            self.unit_report(target, 'not in range!')
            return 0, False

        if self.do_physical_evade(user, weapon, target):
            return 0, False

        damage, crit = attack.damage(user, weapon, target)
        if not crit:
            src = f'{user.name}\'s {weapon.weapon_name}'
        else:
            src = f'{user.name}\'s {weapon.weapon_name} (critical!)'
        self.change_target_hp(target, damage, src)
        self.weapon_chance_to_add_or_cancel_status(user, weapon, target)
        return damage, crit

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

    def weapon_chance_to_add_or_cancel_status(self, user: Combatant, weapon: Equipment, target: Combatant):
        if not target.healthy:
            return  # FIXME: this doesn't strictly make sense I don't think...

        if not (weapon.chance_to_add or weapon.chance_to_cancel):
            return

        for status in weapon.chance_to_add:
            if random.random() >= 0.19:
                continue
            self.add_status(target, status, f'{user.name}\'s {weapon.weapon_name}')

        for status in weapon.chance_to_cancel:
            if random.random() >= 0.19:
                continue
            self.cancel_status(target, status, f'{user.name}\'s {weapon.weapon_name}')

    def change_target_hp(self, target: Combatant, amount, source: str):
        if amount > 0:
            if not target.healthy:
                return
            if target.mana_shield and target.mp > 0 and self.roll_brave_reaction(target):
                self.change_target_mp(target, amount, source + ' (mana shield)')

        target.hp = min(target.max_hp, max(0, target.hp - amount))
        if amount >= 0:
            self.unit_report(target, f'took {amount} damage from {source}')
            target.took_damage_during_active_turn = True
            for status in DAMAGE_CANCELS:
                self.cancel_status(target, status, source)
        else:
            self.unit_report(target, f'was healed for {abs(amount)} from {source}')
        if target.hp == 0:
            self.target_died(target)

    def change_target_mp(self, target: Combatant, amount, source: str):
        if not target.healthy:
            return
        target.mp = min(target.max_mp, max(0, target.mp - amount))
        if amount >= 0 and source:
            self.unit_report(target, f'took {amount} MP damage from {source}')
        elif amount < 0 and source:
            self.unit_report(target, f'recovered {abs(amount)} MP from {source}')

    def after_damage_reaction(self, target: Combatant, inflicter: Combatant, amount: int):
        if amount == 0:
            return

        if target.auto_potion and self.roll_brave_reaction(target):
            # FIXME: Need to consider UNDEAD
            self.change_target_hp(target, -100, 'auto potion')
            return

        if target.damage_split and self.roll_brave_reaction(target):
            self.change_target_hp(target, -(amount // 2), 'damage split')
            self.change_target_hp(inflicter, amount // 2, 'damage split')
            return

    def target_died(self, target: Combatant):
        self.unit_report(target, 'died')
        for status in DEATH_CANCELS:
            self.cancel_status(target, status, 'death')


def main():
    fftbg.server.configure_logging('SIMULATION_LOG_LEVEL')
    tourny = fftbg.tournament.parse_tournament(Path('data/tournaments/1584818551017.json'))
    patch = fftbg.patch.get_patch(tourny.modified)

    num_sims = 100
    correct = 0
    total = 0

    for match_up in tourny.match_ups:
        for _ in range(num_sims):
            LOG.info(f'Starting match, {match_up.left.color} vs {match_up.right.color}')
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

            if sim.left_wins and match_up.left_wins:
                correct += 1
                total += 1
            else:
                total += 1

    LOG.info(f'Total correct: {correct}/{total}')


if __name__ == '__main__':
    main()
