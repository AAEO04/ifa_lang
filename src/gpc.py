# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    GPC - GRANDPARENT-PARENT-CHILD HIERARCHY                  ║
║                    "The Ancestral Call Stack"                                ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  GPC Hierarchy - Debug-friendly 3-level call stack visualization.           ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from typing import Any, Dict, List, Optional
from dataclasses import dataclass, field
from collections import deque


@dataclass
class GPCFrame:
    """A single frame in the GPC call stack."""
    name: str
    role: str  # "grandparent", "parent", or "child"
    line: int = 0
    file: str = ""
    local_vars: Dict[str, Any] = field(default_factory=dict)
    odu_domain: str = ""


class GPCStack:
    """
    Grandparent-Parent-Child call stack.
    Maintains a 3-level view of the call hierarchy for debugging.
    """
    
    def __init__(self):
        self._frames: deque = deque(maxlen=256)
        self._current_depth: int = 0
    
    def push(self, name: str, line: int = 0, file: str = "", odu: str = "", **local_vars):
        """Push a new frame onto the stack."""
        frame = GPCFrame(name=name, role="child", line=line, file=file,
                        local_vars=local_vars, odu_domain=odu)
        self._frames.append(frame)
        self._current_depth += 1
    
    def pop(self) -> Optional[GPCFrame]:
        """Pop a frame from the stack."""
        if self._frames:
            self._current_depth -= 1
            return self._frames.pop()
        return None
    
    def get_gpc(self) -> Dict[str, Optional[GPCFrame]]:
        """Get the current Grandparent-Parent-Child view."""
        frames = list(self._frames)
        n = len(frames)
        result = {"grandparent": None, "parent": None, "child": None}
        
        if n >= 1:
            f = frames[-1]
            result["child"] = GPCFrame(name=f.name, role="child", line=f.line,
                                       file=f.file, local_vars=f.local_vars.copy(),
                                       odu_domain=f.odu_domain)
        if n >= 2:
            f = frames[-2]
            result["parent"] = GPCFrame(name=f.name, role="parent", line=f.line,
                                        file=f.file, local_vars=f.local_vars.copy(),
                                        odu_domain=f.odu_domain)
        if n >= 3:
            f = frames[-3]
            result["grandparent"] = GPCFrame(name=f.name, role="grandparent", line=f.line,
                                             file=f.file, local_vars=f.local_vars.copy(),
                                             odu_domain=f.odu_domain)
        return result
    
    def display(self):
        """Pretty-print the GPC hierarchy."""
        gpc = self.get_gpc()
        print("\n╔════════════════════════════════════════════════╗")
        print("║         GPC HIERARCHY (Call Stack)             ║")
        print("╚════════════════════════════════════════════════╝")
        
        if gpc["grandparent"]:
            g = gpc["grandparent"]
            print(f"\n  👴 Bàbá-Nlá (Grandparent): {g.name}")
            print(f"     Line: {g.line}, Odù: {g.odu_domain or 'N/A'}")
        if gpc["parent"]:
            p = gpc["parent"]
            print(f"\n  👨 Bàbá (Parent): {p.name}")
            print(f"     Line: {p.line}, Odù: {p.odu_domain or 'N/A'}")
        if gpc["child"]:
            c = gpc["child"]
            print(f"\n  👶 Ọmọ (Child): {c.name} ← CURRENT")
            print(f"     Line: {c.line}, Odù: {c.odu_domain or 'N/A'}")
            if c.local_vars:
                print(f"     Vars: {c.local_vars}")
        print()
    
    def traceback(self) -> str:
        """Generate a traceback string."""
        lines = ["Ifá Traceback (most recent call last):"]
        for frame in self._frames:
            odu = f" [{frame.odu_domain}]" if frame.odu_domain else ""
            lines.append(f"  File \"{frame.file}\", line {frame.line}, in {frame.name}{odu}")
        return "\n".join(lines)
    
    def __len__(self):
        return len(self._frames)
    
    def __repr__(self):
        gpc = self.get_gpc()
        parts = [f.name for f in [gpc["grandparent"], gpc["parent"], gpc["child"]] if f]
        return "GPCStack(" + " → ".join(parts) + ")"


class GPCDebugger:
    """Debugger with GPC-aware breakpoints and inspection."""
    
    def __init__(self):
        self.stack = GPCStack()
        self.breakpoints: Dict[str, int] = {}
    
    def enter_function(self, name: str, line: int, file: str = "", odu: str = "", **local_vars):
        self.stack.push(name, line, file, odu, **local_vars)
        if name in self.breakpoints:
            print(f"\n🔴 BREAKPOINT: {name} (line {line})")
            self.stack.display()
    
    def exit_function(self):
        self.stack.pop()
    
    def set_breakpoint(self, function_name: str, line: int = 0):
        self.breakpoints[function_name] = line
        print(f"  🔴 Breakpoint set: {function_name}")
    
    def on_error(self, error: Exception) -> str:
        gpc = self.stack.get_gpc()
        report = [
            "\n╔════════════════════════════════════════════════════════════╗",
            "║              BABALAWO ERROR DIAGNOSIS (GPC)                ║",
            "╚════════════════════════════════════════════════════════════╝",
            f"\n  Error: {error}\n",
            self.stack.traceback(), ""
        ]
        if gpc["child"]: report.append(f"  Current (Ọmọ): {gpc['child'].name}")
        if gpc["parent"]: report.append(f"  Caller (Bàbá): {gpc['parent'].name}")
        if gpc["grandparent"]: report.append(f"  Origin (Bàbá-Nlá): {gpc['grandparent'].name}")
        return "\n".join(report)


gpc_debugger = GPCDebugger()

__all__ = ['GPCFrame', 'GPCStack', 'GPCDebugger', 'gpc_debugger']
