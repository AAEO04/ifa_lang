# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           BASE ODÙ MODULE CLASS                                              ║
║                    Parent class for all domains                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""


class OduModule:
    """Base class for all Odù domain modules."""
    
    def __init__(self, name: str, binary: str, description: str):
        self.name = name
        self.binary = binary
        self.description = description
        self._methods = {}
    
    def _register(self, name: str, func, desc: str):
        """Register a method (Ese) for this domain."""
        self._methods[name] = {"func": func, "desc": desc}
    
    def help(self):
        """List all available methods for this domain."""
        print(f"\n=== {self.name} ({self.binary}) ===")
        print(f"  {self.description}\n")
        for name, info in self._methods.items():
            print(f"  .{name}() - {info['desc']}")
    
    def __repr__(self):
        return f"<OduModule: {self.name}>"
