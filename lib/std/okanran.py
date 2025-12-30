# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    Ọ̀KÀNRÀN - THE TROUBLE-MAKER (ERRORS)                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Domain: Error Handling, Assertions, Testing                                 ║
║  English Aliases: Error, Except, Test                                        ║
║  The Spirit of Justice through struggle - Hard truths and validation         ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from typing import Any, Optional, Callable

from .base import OduModule


class IfaException(Exception):
    """Custom exception for Ifá-Lang errors."""
    pass


class OkanranDomain(OduModule):
    """
    Ọ̀kànràn - The Trouble-Maker
    Handles error throwing, assertions, and testing.
    """
    
    def __init__(self):
        super().__init__("Ọ̀kànràn", "0001", "The Troublemaker - Error Handling")
        
        # Error Throwing
        self._register("binu", self.binu, "Throw/raise exception")
        
        # Assertions
        self._register("je", self.je, "Assert condition is true")
        self._register("daju", self.daju, "Expect/verify expected equals actual")
        
        # Spec Functions
        self._register("gbe", self.gbe, "Catch/rescue exception handler")
    
    # =========================================================================
    # ERROR THROWING
    # =========================================================================
    
    def binu(self, message: str) -> None:
        """
        bínú (Throw/Get Angry) - Throw an exception
        """
        print(f"[Ọ̀kànràn] Error thrown: {message}")
        raise IfaException(message)
    
    # =========================================================================
    # ASSERTIONS
    # =========================================================================
    
    def je(self, condition: Any, message: str = "Assertion failed") -> bool:
        """
        jẹ́ (Assert/It is so) - Assert a condition is true
        """
        if not condition:
            print(f"[Ọ̀kànràn] Assertion failed: {message}")
            raise IfaException(f"Assertion failed: {message}")
        return True
    
    def daju(self, expected: Any, actual: Any, message: str = "") -> bool:
        """
        dájú (Expect/Verify) - Check expected equals actual
        """
        if expected != actual:
            msg = f"Expected {expected}, got {actual}. {message}".strip()
            print(f"[Ọ̀kànràn] Verification failed: {msg}")
            raise IfaException(msg)
        return True
    
    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================
    
    def gbe(self, func: Callable = None) -> Callable:
        """
        gbé (Catch/Rescue) - Catch exception context handler/decorator
        
        Returns a wrapper that suppresses errors.
        """
        def wrapper(*args, **kwargs):
            try:
                if callable(func):
                    return func(*args, **kwargs)
            except Exception as e:
                print(f"[Ọ̀kànràn] Rescued error: {e}")
                return None
        return wrapper


# Module-level singleton and functions for backwards compatibility
_okanran = OkanranDomain()

def binu(message: str) -> None:
    return _okanran.binu(message)

def je(condition: Any, message: str = "Assertion failed") -> bool:
    return _okanran.je(condition, message)

def daju(expected: Any, actual: Any, message: str = "") -> bool:
    return _okanran.daju(expected, actual, message)

def gbe(func: Callable = None) -> Callable:
    return _okanran.gbe(func)

# English aliases
throw = binu
assert_test = je
assert_ = je  # 'assert' is reserved keyword
expect = daju
equal = daju
catch = gbe
rescue = gbe
