# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ÌWÒRÌ - THE REFLECTOR (0110)                                       ║
║                    Time Functions & Iteration                                ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import time
from datetime import datetime, timedelta
from typing import List, Any

from .base import OduModule


class IworiDomain(OduModule):
    """The Mirror - Time functions and iteration."""
    
    def __init__(self):
        super().__init__("Ìwòrì", "0110", "The Reflector - Time & Loops")
        self._timers = {}
        
        # Get Time
        self._register("akoko", self.akoko, "Get current timestamp")
        self._register("oju_ojo", self.oju_ojo, "Get formatted date")
        self._register("wakati", self.wakati, "Get formatted time")
        self._register("odun", self.odun, "Get current year")
        self._register("osu", self.osu, "Get current month")
        self._register("ojo", self.ojo, "Get current day")
        self._register("epoch", self.epoch, "Get Unix epoch")
        self._register("ago", self.akoko, "Get current time (Spec Alias)")
        
        # Parse & Manipulate
        self._register("tumo", self.tumo, "Parse date string")
        self._register("fi_kun", self.fi_kun, "Add time (days, hours...)")
        self._register("yato", self.yato, "Time difference in seconds")
        self._register("is_leap", self.is_leap, "Is leap year")
        
        # Timers & Stopwatches
        self._register("bere", self.bere, "Start timer/stopwatch")
        self._register("da_duro", self.da_duro, "Stop timer & get duration")
        
        # Iteration
        self._register("iye", self.iye, "Range generator")
        
        # Spec Functions
        self._register("royin", self.royin, "Report/Debug")
        self._register("mo", self.mo, "Predict/Know")
        self._register("wo", self.wo, "Introspect/Look")
    
    # =========================================================================
    # GET TIME
    # =========================================================================
    
    def akoko(self) -> str:
        """Current timestamp ISO format."""
        return datetime.now().isoformat()
    
    def oju_ojo(self, fmt: str = "%Y-%m-%d") -> str:
        """Formatted date."""
        return datetime.now().strftime(fmt)
    
    def wakati(self, fmt: str = "%H:%M:%S") -> str:
        """Formatted time."""
        return datetime.now().strftime(fmt)
    
    def odun(self) -> int:
        """Current year."""
        return datetime.now().year
    
    def osu(self) -> int:
        """Current month."""
        return datetime.now().month
    
    def ojo(self) -> int:
        """Current day."""
        return datetime.now().day
    
    def epoch(self) -> float:
        """Unix epoch timestamp."""
        return time.time()
    
    # =========================================================================
    # PARSE & MANIPULATE
    # =========================================================================
    
    def tumo(self, date_str: str, fmt: str = "%Y-%m-%d") -> datetime:
        """Parse date string to object (internal) or ISO string."""
        try:
            return datetime.strptime(date_str, fmt).isoformat()
        except:
            return ""
            
    def fi_kun(self, date_str: str, days: int = 0, hours: int = 0, minutes: int = 0) -> str:
        """Add duration to date string (ISO). Returns new ISO string."""
        try:
            dt = datetime.fromisoformat(date_str)
        except:
            dt = datetime.now()
            
        new_dt = dt + timedelta(days=days, hours=hours, minutes=minutes)
        return new_dt.isoformat()
    
    def yato(self, start_str: str, end_str: str) -> float:
        """Difference between two ISO dates in seconds."""
        try:
            d1 = datetime.fromisoformat(start_str)
            d2 = datetime.fromisoformat(end_str)
            return (d2 - d1).total_seconds()
        except:
            return 0.0
            
    def is_leap(self, year: int) -> bool:
        """Check if year is leap year."""
        return year % 4 == 0 and (year % 100 != 0 or year % 400 == 0)

    # =========================================================================
    # TIMERS & STOPWATCHES
    # =========================================================================
    
    def bere(self, name: str = "default"):
        """Start a named timer."""
        self._timers[name] = time.time()
        
    def da_duro(self, name: str = "default") -> float:
        """Stop timer and return duration in seconds."""
        if name in self._timers:
            duration = time.time() - self._timers[name]
            return round(duration, 4)
        return 0.0
        
    # =========================================================================
    # ITERATION
    # =========================================================================
    
    def iye(self, start: int, end: int, step: int = 1) -> List[int]:
        """Range generator."""
        return list(range(start, end, step))

    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================

    def royin(self, obj: Any = None) -> str:
        """ròyìn() - Report/Debug (Circular Debugger)."""
        return repr(obj) if obj is not None else "Debug Report"

    def mo(self, data: Any = None) -> Any:
        """mọ̀() - Know/Predict (ML Mock)."""
        return data

    def wo(self, obj: Any = None) -> List[str]:
        """wo() - Look/Introspect."""
        if obj is None: return []
        return dir(obj)
