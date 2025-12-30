# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ỌWỌNRÍN - THE REVERSER (0011)                                      ║
║                    Random & Chaos Functions                                  ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import random
import uuid as uuid_mod
from typing import Any, List

from .base import OduModule


class OwonrinDomain(OduModule):
    """The Chaotic - Random number generation."""
    
    def __init__(self):
        super().__init__("Ọ̀wọ́nrín", "0011", "The Reverser - Random & Chaos")
        
        self._register("afesona", self.afesona, "Random integer")
        self._register("aidaniloju", self.aidaniloju, "Random float 0-1")
        self._register("yan", self.yan, "Pick random from list")
        self._register("daru", self.daru, "Shuffle list")
        self._register("irugbin", self.irugbin, "Set random seed")
        self._register("uuid", self.uuid, "Generate UUID-like string")
        
        # Spec Functions
        self._register("bo", self.bo, "Random Int")
        self._register("paaro", self.paaro, "Shuffle/Swap")
        self._register("da", self.da, "Fuzz/Flip Bit")
    
    def afesona(self, low: int = 0, high: int = 255) -> int:
        """Random integer in range."""
        return random.randint(low, high)
    
    def aidaniloju(self) -> float:
        """Random float 0-1."""
        return random.random()
    
    def yan(self, items: List[Any]) -> Any:
        """Pick random from list."""
        return random.choice(items) if items else None
    
    def daru(self, items: List[Any]) -> List[Any]:
        """Shuffle list (returns new list)."""
        result = items.copy()
        random.shuffle(result)
        return result
    
    def irugbin(self, seed: int):
        """Set random seed."""
        random.seed(seed)
    
    def uuid(self) -> str:
        """Generate UUID-like string."""
        return str(uuid_mod.uuid4())

    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================

    def bo(self, min_val: int = 0, max_val: int = 9999) -> int:
        """bò() - Random Int."""
        return random.randint(min_val, max_val)

    def paaro(self, items: List[Any]) -> List[Any]:
        """pààrọ̀() - Shuffle/Swap."""
        return self.daru(items)

    def da(self, value: int = 0) -> int:
        """dà() - Fuzz/Flip Bit (Chaos). Bitwise NOT or random bit flip."""
        # Simple chaos: flip a random bit in a 8-bit simulation, or logical NOT 
        if value is None: return 0
        return ~value
