from abc import ABCMeta, abstractmethod
from typing import Optional

from fftbg.equipment import Equipment
from fftbg.simulation.combatant import Combatant


class AbstractSimulation(metaclass=ABCMeta):
    @abstractmethod
    def report(self, s: str):
        pass

    @abstractmethod
    def unit_report(self, combatant: Combatant, s: str):
        pass

    @abstractmethod
    def ai_thirteen_rule(self) -> bool:
        pass

    @abstractmethod
    def roll_brave_reaction(self, user: Combatant) -> bool:
        pass

    @abstractmethod
    def ai_can_be_cowardly(self, user: Combatant) -> bool:
        pass

    @abstractmethod
    def can_move_into_range(self, user: Combatant, range: int, target: Combatant) -> bool:
        pass

    @abstractmethod
    def in_range(self, user: Combatant, range: int, target: Combatant):
        pass

    @abstractmethod
    def do_physical_evade(self, user: Combatant, weapon: Equipment, target: Combatant) -> bool:
        pass

    @abstractmethod
    def add_status(self, target: Combatant, status: str, src: str):
        pass

    @abstractmethod
    def cancel_status(self, target: Combatant, status: str, src: Optional[str] = None):
        pass

    @abstractmethod
    def weapon_chance_to_add_or_cancel_status(self, user: Combatant, weapon: Equipment, target: Combatant):
        pass

    @abstractmethod
    def change_target_hp(self, target: Combatant, amount, source: str):
        pass

    @abstractmethod
    def change_target_mp(self, target: Combatant, amount, source: str):
        pass

    @abstractmethod
    def after_damage_reaction(self, target: Combatant, inflicter: Combatant, amount: int):
        pass

    @abstractmethod
    def target_died(self, target: Combatant):
        pass
