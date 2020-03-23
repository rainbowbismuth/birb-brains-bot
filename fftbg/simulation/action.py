from dataclasses import dataclass
from typing import Callable

from fftbg.simulation.combatant import Combatant


@dataclass
class Action:
    range: int
    target: Combatant
    perform: Callable
