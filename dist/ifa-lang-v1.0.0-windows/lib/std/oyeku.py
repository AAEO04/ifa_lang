# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ỌYẸKÚ - THE CLOSER (0000)                                          ║
║                    Process Termination & Sleep                               ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import sys
import time

from .base import OduModule


class OyekuDomain(OduModule):
    """The Darkness - Process termination and sleep."""
    
    def __init__(self):
        super().__init__("Ọ̀yẹ̀kú", "0000", "The Closer - Exit & Sleep")
        
        self._register("duro", self.duro, "Halt execution")
        self._register("ku", self.ku, "Exit with code")
        self._register("sun", self.sun, "Sleep for seconds")
        self._register("sun_ms", self.sun_ms, "Sleep for milliseconds")
    
    def duro(self):
        """Halt - graceful stop."""
        print("[Ọ̀yẹ̀kú] Process terminated gracefully.")
    
    def ku(self, code: int = 0):
        """Exit with code."""
        sys.exit(code)
    
    def sun(self, seconds: float):
        """Sleep for seconds."""
        time.sleep(seconds)
    
    def sun_ms(self, ms: int):
        """Sleep for milliseconds."""
        time.sleep(ms / 1000.0)

    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================

    def gbale(self):
        """gbálẹ̀() - Garbage Collect."""
        import gc
        gc.collect()
        
    def pana(self):
        """paná() - Shutdown (alias to exit or more severe)."""
        sys.exit(0)

