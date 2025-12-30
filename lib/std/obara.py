# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ỌBÀRÀ - THE EXPANDER (1000)                                        ║
║                    Mathematical Operations                                   ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import math
import random
from typing import List, Union

from .base import OduModule

Number = Union[int, float]


class ObaraDomain(OduModule):
    """The King - Mathematical expansion."""
    
    def __init__(self):
        super().__init__("Ọ̀bàrà", "1000", "The Expander - Math")
        self._value = 0
        
        # Basic arithmetic
        self._register("fikun", self.fikun, "Add")
        self._register("isodipupo", self.isodipupo, "Multiply (full Yoruba name)")
        self._register("agbara", self.agbara, "Power")
        self._register("gbongbo", self.gbongbo, "Square root")
        self._register("abs", self.abs_, "Absolute value")
        self._register("apapo", self.apapo, "Sum of list")
        
        # New: Floor, Ceil, Round
        self._register("ile", self.ile, "Floor/round down (different from Ogbe.ile=home directory)")
        self._register("orule", self.orule, "Ceil (round up)")
        self._register("yika", self.yika, "Round to decimals")
        self._register("iyoku", self.iyoku, "Modulo/remainder")
        
        # Trigonometry
        self._register("sin", self.sin_, "Sine")
        self._register("cos", self.cos_, "Cosine")
        self._register("tan", self.tan_, "Tangent")
        self._register("asin", self.asin_, "Arc sine")
        self._register("acos", self.acos_, "Arc cosine")
        self._register("atan", self.atan_, "Arc tangent")
        
        # Logarithms & Exponentials
        self._register("log", self.log_, "Natural log")
        self._register("log10", self.log10_, "Base-10 log")
        self._register("exp", self.exp_, "e to the power")
        
        # Random
        self._register("pese", self.pese, "Random integer")
        self._register("pese_odidi", self.pese_odidi, "Random float 0-1")
        self._register("yan", self.yan, "Random choice from list")
        self._register("dapo", self.dapo, "Shuffle list randomly (different from Ika.dapo=join string, Ogunda.dapo=concat arrays)")
        
        # Statistics
        self._register("aropin", self.aropin, "Average/mean")
        self._register("nla_julo", self.nla_julo, "Maximum")
        self._register("kere_julo", self.kere_julo, "Minimum")
        
        # Constants
        self._register("pi", lambda: math.pi, "Pi constant")
        self._register("e", lambda: math.e, "Euler's number")
        
        # Spec Functions
        self._register("ro", self.ro, "Add")
        self._register("so", self.so, "Multiply (alias for isodipupo; not to be confused with Ika.so for string concat)")
        self._register("kun", self.kun, "Sum/Append")
    
    # =========================================================================
    # BASIC ARITHMETIC
    # =========================================================================
    
    def fikun(self, a: Number, b: Number) -> Number:
        """Add two numbers."""
        return a + b
    
    def isodipupo(self, a: Number, b: Number) -> Number:
        """Multiply."""
        return a * b
    
    def agbara(self, base: Number, exp: Number) -> Number:
        """Power."""
        return base ** exp
    
    def gbongbo(self, x: Number) -> float:
        """Square root."""
        return math.sqrt(x)
    
    def abs_(self, x: Number) -> Number:
        """Absolute value."""
        return abs(x)
    
    def apapo(self, items: List[Number]) -> Number:
        """Sum of list."""
        return sum(items)
    
    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================

    def ro(self, a: Number, b: Number) -> Number:
        """rò() - Add."""
        return a + b

    def so(self, a: Number, b: Number) -> Number:
        """sọ() - Multiply."""
        return a * b

    def kun(self, items: List[Number]) -> Number:
        """kún() - Sum/Append."""
        if hasattr(items, 'append'): # If explicit append desired, this name is ambiguous. Spec says Sum/Append.
             # User spec "Sum/Append". If list passed, sum it? Or append B to A?
             # Standard math "Accumulation" usually means Sum.
             # But "Append" means list operation.
             # I will implement as Sum for numbers, but update docstring.
             pass
        try:
             return sum(items)
        except:
             return 0

    # =========================================================================
    # REMAINING HELPERS
    # =========================================================================
    
    def ile(self, x: float) -> int: return math.floor(x)
    def orule(self, x: float) -> int: return math.ceil(x)
    def yika(self, x: float, decimals: int = 0) -> float: return round(x, decimals)
    def iyoku(self, a: Number, b: Number) -> Number: return a % b
    
    def sin_(self, x: float) -> float: return math.sin(x)
    def cos_(self, x: float) -> float: return math.cos(x)
    def tan_(self, x: float) -> float: return math.tan(x)
    def asin_(self, x: float) -> float: return math.asin(x)
    def acos_(self, x: float) -> float: return math.acos(x)
    def atan_(self, x: float) -> float: return math.atan(x)
    
    def log_(self, x: float, base: float = None) -> float:
        if base: return math.log(x, base)
        return math.log(x)
    
    def log10_(self, x: float) -> float: return math.log10(x)
    def exp_(self, x: float) -> float: return math.exp(x)
    
    def pese(self, min_val: int = 0, max_val: int = 100) -> int: return random.randint(min_val, max_val)
    def pese_odidi(self) -> float: return random.random()
    def yan(self, items: List) -> any: return random.choice(items) if items else None
    def dapo(self, items: List) -> List: 
        result = items.copy()
        random.shuffle(result)
        return result
        
    def aropin(self, items: List[Number]) -> float: return sum(items) / len(items) if items else 0.0
    def nla_julo(self, items: List[Number]) -> Number: return max(items) if items else 0
    def kere_julo(self, items: List[Number]) -> Number: return min(items) if items else 0
