# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ÒTÚÚRÚPỌ̀N - THE BEARER (0010)                                      ║
║                    Mathematical Subtraction & Division                       ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from typing import List

from .base import OduModule


class OturuponDomain(OduModule):
    """The Bearer - Mathematical reduction."""
    
    def __init__(self):
        super().__init__("Òtúúrúpọ̀n", "0010", "The Bearer - Math Sub/Div")
        
        self._register("din", self.din, "Subtract")
        self._register("pin", self.pin, "Divide numbers (different from Ika.pin=split string)")
        self._register("ku", self.ku, "Modulo")
        self._register("oke", self.oke, "Max of two")
        self._register("isale", self.isale, "Min of two")
        self._register("aropin", self.aropin, "Average of list")
        
        # Spec Functions
        self._register("ge", self.ge, "Divide/Cut numbers (alias for pin; different from Ika.ge=slice, Ogunda.ge=create array)")
        self._register("kekere", self.kekere, "Minimum of two values (different from Ika.kekere=lowercase)")
    
    def din(self, a: int, b: int) -> int:
        """Subtract."""
        return a - b
    
    def pin(self, a: int, b: int) -> float:
        """Divide."""
        return a / b if b != 0 else 0
    
    def ku(self, a: int, b: int) -> int:
        """Modulo."""
        return a % b if b != 0 else 0
    
    def oke(self, a: int, b: int) -> int:
        """Max."""
        return max(a, b)
    
    def isale(self, a: int, b: int) -> int:
        """Min."""
        return min(a, b)
    
    def aropin(self, items: List[int]) -> float:
        """Average."""
        return sum(items) / len(items) if items else 0.0

    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================

    def ge(self, a: int, b: int) -> float:
        """gé() - Divide/Cut (Alias for pin)."""
        return self.pin(a, b)
    
    def kekere(self, a: int, b: int) -> int:
        """kékeré() - Min (Alias for isale)."""
        return self.isale(a, b)
