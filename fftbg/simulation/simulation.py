import logging
import random
from pathlib import Path
from typing import List, Optional

import colored

import fftbg.arena
import fftbg.patch
import fftbg.server
import fftbg.simulation.commands.attack as cmd_attack
import fftbg.simulation.commands.item as cmd_item
import fftbg.tournament
from fftbg.arena import Arena
from fftbg.equipment import Equipment
from fftbg.simulation.abc.simulation import AbstractSimulation
from fftbg.simulation.combatant import Combatant
from fftbg.simulation.status import TIME_STATUS_LEN, TIME_STATUS_INDEX_REVERSE, DAMAGE_CANCELS, DEATH_CANCELS, \
    TRANSPARENT, RERAISE, UNDEAD, DEATH, CANCELLING_STATUS

LOG = logging.getLogger(__name__)

CYAN = colored.fg('cyan')
RED = colored.fg('red')
BLUE = colored.fg('blue')


class Simulation(AbstractSimulation):
    def __init__(self, combatants: List[Combatant], arena: Arena, log_report=False):
        self.combatants = combatants
        self.arena = arena
        self.clock_tick: int = 0
        self.last_action = None  # TODO: mime cycle
        self.slow_actions = []
        self.active_turns = []
        self.left_wins = None
        self.time_out_win = False
        self.log_report = log_report
        self.prepend = ''
        self.set_phase('Init')

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

            # TODO: More involved time out win
            if self.clock_tick > 10000:
                self.left_wins = True
                self.time_out_win = True

    def team_healthy(self, team: int):
        return any([combatant.healthy for combatant in self.combatants if combatant.team == team])

    def tick(self):
        self.phase_status_check()

        self.phase_slow_action_charging()
        if self.slow_actions:
            self.phase_slow_action_resolve()

        self.phase_ct_charging()
        if self.active_turns:
            self.phase_active_turn_resolve()

    def prepend_info(self):
        return f'CT {self.clock_tick}: {self.prepend}'

    def set_phase(self, phase_name):
        if self.log_report:
            self.prepend = colored.stylize(phase_name, CYAN)

    def colored_name(self, combatant: Combatant):
        if combatant.team == 0:
            return colored.stylize(combatant.name, RED)
        else:
            return colored.stylize(combatant.name, BLUE)

    def report(self, s: str):
        if self.log_report:
            LOG.info(f'{self.prepend_info()}: {s}')

    def unit_report(self, combatant: Combatant, s: str):
        if self.log_report:
            LOG.info(f'{self.prepend_info()}: {combatant.name} ({combatant.hp} HP) {s}')

    def phase_status_check(self):
        self.clock_tick += 1
        self.set_phase('Status Check')
        for combatant in self.combatants:
            for i in range(TIME_STATUS_LEN):
                has_condition = combatant.timed_status_conditions[i] > 0
                if has_condition:
                    combatant.timed_status_conditions[i] -= 1
                if has_condition and combatant.timed_status_conditions[i] == 0:
                    condition = TIME_STATUS_INDEX_REVERSE[i]
                    combatant.cancel_status(condition)  # To clear flag
                    self.unit_report(combatant, f'no longer has {condition}')

    def phase_slow_action_charging(self):
        self.set_phase('Slow Action Charge')
        for combatant in self.combatants:
            if not combatant.ctr_action:
                continue

            if combatant.stop:
                continue  # FIXME: Does stop just remove the slow action?

            combatant.ctr -= 1
            if combatant.ctr <= 0:
                self.slow_actions.append(combatant)

    def phase_slow_action_resolve(self):
        self.set_phase('Slow Action Resolve')
        while self.slow_actions:
            combatant = self.slow_actions.pop(0)
            if not combatant.healthy:
                continue
            self.prepend = f'{self.colored_name(combatant)}\'s C'
            action = combatant.ctr_action
            combatant.ctr_action = None
            action()

    def phase_ct_charging(self):
        self.set_phase('CT Charging')
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

    def set_active_turn_status_bar(self, combatant: Combatant):
        if not self.log_report:
            return

        condition_str = ', '.join(combatant.all_statuses)
        if condition_str:
            self.prepend = f'{self.colored_name(combatant)} ({combatant.hp} HP, {condition_str})'
        else:
            self.prepend = f'{self.colored_name(combatant)} ({combatant.hp} HP)'

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
        self.set_phase('Active Turn Resolve')
        while self.active_turns:
            combatant = self.active_turns.pop(0)

            if combatant.petrified or combatant.crystal or combatant.stop or combatant.sleep:
                continue

            if combatant.dead and combatant.reraise and not combatant.undead:
                self.change_target_hp(combatant, combatant.max_hp // 10, RERAISE)
                self.cancel_status(combatant, RERAISE)

            if combatant.dead and combatant.crystal_counter > 0:
                combatant.crystal_counter -= 1

                if combatant.crystal_counter == 0 and combatant.undead and random.randint(1, 2) == 2:
                    self.change_target_hp(combatant, random.randint(1, combatant.max_hp), UNDEAD)
                    combatant.crystal_counter = 4

                elif combatant.crystal_counter == 0:
                    self.unit_report(combatant, 'has become a crystal')
                    continue

            if combatant.dead:
                continue

            self.clear_active_turn_flags()
            combatant.on_active_turn = True

            self.set_active_turn_status_bar(combatant)

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
        return random.random() <= 0.137

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
        self.report(f'moved from {old_location} to {user.location}')

    def move_to_range(self, user: Combatant, range: int, target: Combatant):
        if user.moved_during_active_turn or user.dont_move:
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

    # def closest_foe_location(self, user: Combatant):
    #     foe = None
    #     for target in self.combatants:
    #         if not user.is_foe(target):
    #             continue
    #         if not target.healthy:
    #             # TODO: Do I really need this?
    #             continue
    #         if foe is None:
    #             foe = target
    #         elif user.distance(target) < user.distance(foe):
    #             foe = target
    #     return foe.location

    def move_towards_unit(self, user: Combatant, target: Combatant):
        if user.moved_during_active_turn or user.dont_move:
            return
        if user.location - target.location > 0:
            self.do_move_with_bounds(user, max(target.location, user.location - user.move))
        else:
            self.do_move_with_bounds(user, min(target.location, user.location + user.move))

    def move_out_of_combat(self, user: Combatant):
        if user.moved_during_active_turn or user.dont_move:
            return
        if user.team == 0:
            self.do_move_with_bounds(user, user.location - user.move)
        else:
            self.do_move_with_bounds(user, user.location + user.move)

    def ai_calculate_target_value(self, user: Combatant, target: Combatant) -> float:
        priority = target.hp_percent

        priority += 0.51 * target.broken_items
        priority += self.ai_calculate_status_target_value_mod(target)
        priority += self.ai_calculate_caster_hate_mod(target)
        # TODO: Golem fear

        if user.is_foe(target):
            return -priority
        return priority

    def ai_calculate_all_target_values(self, user: Combatant):
        for target in self.combatants:
            target.target_value = self.ai_calculate_target_value(user, target)

    def ai_calculate_caster_hate_mod(self, target: Combatant) -> float:
        if not target.can_cast_mp_ability:
            return 0.0
        mp_percent = target.mp / target.max_mp
        return (mp_percent / 16.0) * target.num_mp_using_abilities

    def ai_calculate_status_target_value_mod(self, target: Combatant) -> float:
        total = 0.0

        # 0x0058: Current Statuses 1
        # 		0x80 - 							0% (0000)
        # 		0x40 - Crystal					-150% -c0(ff40)
        # 		0x20 - Dead						-150% -c0(ff40)
        # 		0x10 - Undead					-30.5% -27(ffd9)
        # 		0x08 - Charging					0% (0000)
        # 		0x04 - Jump						0% (0000)
        # 		0x02 - Defending				0% (0000)
        # 		0x01 - Performing				0% (0000)
        if target.dead:
            total -= 1.5

        if target.undead:
            total -= 0.305

        # 	0x0059: Current Statuses 2
        # 		0x80 - Petrify					-90.6% -74(ff8c)
        if target.petrified:
            total -= 0.906

        # 		0x40 - Invite					-180.4% -e7(ff19)
        # NOTE: Skipping Invite because it doesn't exist in FFTBG

        # 		0x20 - Darkness					-50% [-40(ffc0) * Evadable abilities] + 3 / 4
        # TODO: Add darkness

        # 		0x10 - Confusion				-50% -40(ffc0) (+1 / 4 if slow/stop/sleep/don't move/act/)
        if target.confusion:
            if target.slow or target.stop or target.sleep or target.dont_move or target.dont_act:
                total += 0.25
            else:
                total -= 0.5

        # 		0x08 - Silence					-70.3% [-5a(ffa6) * Silence abilities] + 3 / 4
        if target.silence:
            total -= 0.703
            # TODO: Calculate number of silenced abilities

        # 		0x04 - Blood Suck				-90.6% -74(ff8c) (+1 / 4 if slow/stop/sleep/don't move/act/)
        if target.blood_suck:
            if target.slow or target.stop or target.sleep or target.dont_move or target.dont_act:
                total += 0.25
            else:
                total -= 0.906

        # 		0x02 - Cursed					0%(0000)
        # 		0x01 - Treasure					-150% -c0(ff40)
        # 	0x005a: Current Statuses 3
        # 		0x80 - Oil						-5.5% -7(fff9)
        if target.oil:
            total -= 0.055

        # 		0x40 - Float					9.4% c(000c)
        if target.float:
            total += 0.094

        # 		0x20 - Reraise					39.8% 33(0033)
        if target.reraise:
            total += 0.398

        # 		0x10 - Transparent				29.7% 26(0026)
        if target.transparent:
            total += 0.297

        # 		0x08 - Berserk					-30.5% -27(ffd9)
        if target.berserk:
            total -= 0.305

        # 		0x04 - Chicken					-20.3% -1a(ffe6)
        if target.chicken:
            total -= 0.203

        # 		0x02 - Frog						-40.6% -34(ffcc)
        if target.frog:
            total -= 0.406
        # 		0x01 - Critical					-25% -20(ffe0)
        if target.critical:
            total -= 0.25

        # 	0x005b: Current Statuses 4
        # 		0x80 - Poison					-20.3% -1a(ffe6)
        if target.poison:
            total -= 0.203

        # 		0x40 - Regen					19.5% 19(0019)
        if target.regen:
            total += 0.195

        # 		0x20 - Protect					19.5% 19(0019)
        if target.protect:
            total += 0.195

        # 		0x10 - Shell					19.5% 19(0019)
        if target.shell:
            total += 0.195

        # 		0x08 - Haste					14.8% 13(0013)
        if target.haste:
            total += 0.148

        # 		0x04 - Slow						-30.5% -27(ffd9) 0 if Confusion/Charm/Blood Suck
        if target.slow and not (target.confusion or target.charm or target.blood_suck):
            total -= 0.305

        # 		0x02 - Stop						-70.3% -5a(ffa6) 0 if Confusion/Charm/Blood Suck
        if target.stop and not (target.confusion or target.charm or target.blood_suck):
            total -= 0.703

        # 		0x01 - Wall						50% 40(0040)
        if target.wall:
            total += 0.50

        # 	0x005c: Current Statuses 5
        # 		0x80 - Faith					4.7% 6(0006)
        if target.faith:
            total += 0.047

        # 		0x40 - Innocent					-5.5% -7(fff9)
        if target.innocent:
            total -= 0.055

        # 		0x20 - Charm					-50% -40(ffc0) (+1 / 4 if slow/stop/sleep/don't move/act/)
        if target.charm:
            if target.slow or target.stop or target.sleep or target.dont_move or target.dont_act:
                total += 0.25
            else:
                total -= 0.50

        # 		0x10 - Sleep					-30.5% -27(ffd9) 0 if Confusion/Charm/Blood Suck
        if target.sleep and not (target.confusion or target.charm or target.blood_suck):
            total -= 0.305

        # 		0x08 - Don't Move				-30.5% -27(ffd9) 0 if Confusion/Charm/Blood Suck
        if target.dont_move and not (target.confusion or target.charm or target.blood_suck):
            total -= 0.305

        # 		0x04 - Don't Act				-50% -40(ffc0) 0 if Confusion/Charm/Blood Suck
        if target.dont_act and not (target.confusion or target.charm or target.blood_suck):
            total -= 0.50

        # 		0x02 - Reflect					19.5% 19(0019)
        if target.reflect:
            total += 0.195

        # 		0x01 - Death Sentence			-80.5% -67(ff99)
        if target.death_sentence:
            total -= 0.805

        return total

    def ai_do_basic_turn(self, user: Combatant):
        if user.dont_act:
            self.move_out_of_combat(user)
            return

        self.ai_calculate_all_target_values(user)

        targets = self.combatants
        acting_cowardly = user.critical and self.ai_can_be_cowardly(user)
        if acting_cowardly:
            targets = [user]

        actions = []
        for target in targets:
            actions.extend(cmd_item.consider_item(self, user, target))
            actions.extend(cmd_attack.consider_attack(self, user, target))

        actions.sort(key=lambda x: x.target.target_value)

        for action in actions:
            if not self.can_move_into_range(user, action.range, action.target):
                continue

            if not self.in_range(user, action.range, action.target):
                self.move_to_range(user, action.range, action.target)

            # TODO: This handles don't move, is there a better way?
            if not self.in_range(user, action.range, action.target):
                continue

            user.acted_during_active_turn = True
            action.perform()
            break

        if user.moved_during_active_turn:
            return

        if actions:
            self.move_towards_unit(user, actions[0].target)
            return

        self.move_out_of_combat(user)

    def in_range(self, user: Combatant, range: int, target: Combatant):
        dist = user.distance(target)
        return dist <= range

    def do_physical_evade(self, user: Combatant, weapon: Equipment, target: Combatant) -> bool:
        if target.blade_grasp and not target.berserk and self.roll_brave_reaction(target):
            self.unit_report(target, f'blade grasped {user.name}\'s attack')
            return True

        if target.arrow_guard and not target.berserk and weapon.weapon_type in (
                'Longbow', 'Bow', 'Gun', 'Crossbow') and self.roll_brave_reaction(target):
            self.unit_report(target, f'arror guarded {user.name}\'s attack')
            return True

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

    def add_status(self, target: Combatant, status: str, src: str):
        if target.immune_to(status):
            return

        if status == DEATH:
            self.target_died(target)
            self.unit_report(target, f'was killed by {status} from {src}')
            return

        had_status = target.has_status(status)
        target.add_status_flag(status)
        if not had_status:
            self.unit_report(target, f'now has {status} from {src}')

        for cancelled in CANCELLING_STATUS.get(status, []):
            self.cancel_status(target, cancelled, status)

    def cancel_status(self, target: Combatant, status: str, src: Optional[str] = None):
        if not target.has_status(status):
            return
        target.cancel_status(status)
        if src:
            self.unit_report(target, f'had {status} cancelled by {src}')
        else:
            self.unit_report(target, f'had {status} cancelled')

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
        if amount == 0 or target.dead:
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
        target.hp = 0
        self.unit_report(target, 'died')
        for status in DEATH_CANCELS:
            self.cancel_status(target, status, 'death')
        target.crystal_counter = 4


# IDEAS:
#
#  - Need to account for picking up crystals. I think this will go with expanding the
#      where do I move to selection function? Because I will want to get out of AoEs I guess?
#  - Pick up crystal Y/N could just happen after movement.
#      will need a state for 'no longer exists at all?' can I just remove from combatants? do I want to?
#  - I still don't entirely understand this target value thing, I should continue to read the docs
#      if it is used to pick what skills to use, can I separate what is used in that calculation
#      into a separate AI data block? That will require rewriting these skills and how they work, bluh.
#  - Add 13% rule skip in the action consideration loop.
#  - Can I keep statistics on how much different actions happen? Could be a useful part of testing.
#  - Would be interesting to see if these true positives align with bird's true positives
#  - If I run many simulations per match I could start calculating log loss as well.
#  - At that point I should bring in multi-processing :)

def show_one():
    tourny = fftbg.tournament.parse_tournament(Path('data/tournaments/1584818551017.json'))
    patch = fftbg.patch.get_patch(tourny.modified)

    for match_up in tourny.match_ups:
        LOG.info(f'Starting match, {match_up.left.color} vs {match_up.right.color}')
        combatants = []
        for d in match_up.left.combatants:
            combatants.append(Combatant(d, patch, 0))
        for d in match_up.right.combatants:
            combatants.append(Combatant(d, patch, 1))
        arena = fftbg.arena.get_arena(match_up.game_map)
        sim = Simulation(combatants, arena, log_report=True)
        sim.run()
        if sim.left_wins:
            LOG.info('Left team wins!')
        else:
            LOG.info('Right team wins!')


def main():
    import tqdm
    fftbg.server.configure_logging('SIMULATION_LOG_LEVEL')

    num_sims = 1
    time_out_wins = 0
    correct = 0
    total = 0

    for path in tqdm.tqdm(list(Path('data/tournaments').glob('*.json'))):
        tourny = fftbg.tournament.parse_tournament(path)
        patch = fftbg.patch.get_patch(tourny.modified)

        for match_up in tourny.match_ups:
            for _ in range(num_sims):
                combatants = []
                for d in match_up.left.combatants:
                    combatants.append(Combatant(d, patch, 0))
                for d in match_up.right.combatants:
                    combatants.append(Combatant(d, patch, 1))
                arena = fftbg.arena.get_arena(match_up.game_map)
                sim = Simulation(combatants, arena, log_report=False)
                sim.run()

                if sim.left_wins and match_up.left_wins:
                    correct += 1
                    total += 1
                else:
                    total += 1
                if sim.time_out_win:
                    time_out_wins += 1

    LOG.info(f'Total correct: {correct}/{total}')
    LOG.info(f'Percent correct: {correct / total:.1%}')
    LOG.info(f'Time outs: {time_out_wins}')


if __name__ == '__main__':
    main()
