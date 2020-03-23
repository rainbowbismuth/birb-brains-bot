from abc import ABCMeta
from typing import Optional

from fftbg.equipment import Equipment
from fftbg.simulation.combatant import Combatant


class AbstractSimulation(metaclass=ABCMeta):
    def report(self, s: str):
        pass

    def unit_report(self, combatant: Combatant, s: str):
        pass

    def ai_thirteen_rule(self) -> bool:
        pass

    def roll_brave_reaction(self, user: Combatant) -> bool:
        pass

    def ai_can_be_cowardly(self, user: Combatant) -> bool:
        pass

    def can_move_into_range(self, user: Combatant, range: int, target: Combatant) -> bool:
        pass

    def do_move_with_bounds(self, user: Combatant, new_location: int):
        pass

    def move_to_range(self, user: Combatant, range: int, target: Combatant):
        pass

    def move_into_combat(self, user: Combatant):
        pass

    def move_out_of_combat(self, user: Combatant):
        pass

    def in_range(self, user: Combatant, range: int, target: Combatant):
        pass

    def do_physical_evade(self, user: Combatant, weapon: Equipment, target: Combatant) -> bool:
        pass

    def add_status(self, target: Combatant, status: str, src: str):
        pass

    def cancel_status(self, target: Combatant, status: str, src: Optional[str] = None):
        pass

    def weapon_chance_to_add_or_cancel_status(self, user: Combatant, weapon: Equipment, target: Combatant):
        pass

    def change_target_hp(self, target: Combatant, amount, source: str):
        pass

    def change_target_mp(self, target: Combatant, amount, source: str):
        pass

    def after_damage_reaction(self, target: Combatant, inflicter: Combatant, amount: int):
        pass

    def target_died(self, target: Combatant):
        pass
