# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ÌROSÙ - THE VOICE (1100)                                           ║
║                    Console Input/Output                                      ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import sys
import time
from typing import List, Any

from .base import OduModule


class IrosuDomain(OduModule):
    """The Speaker - Console input/output."""
    
    def __init__(self):
        super().__init__("Ìrosù", "1100", "The Voice - Console I/O")
        
        # Basic I/O
        self._register("so", self.so, "Print to console, no newline (different from Ika.so=concat, Obara.so=multiply)")
        self._register("fo", self.fo, "Print with newline")
        self._register("gbo", self.gbo, "Read input from user")
        self._register("gbo_nomba", self.gbo_nomba, "Read integer input")
        self._register("gbo_odidi", self.gbo_odidi, "Read float input")
        self._register("mo", self.mo, "Clear screen (different from Iwori.mo=predict)")
        
        # Formatting
        self._register("awo", self.awo, "Print with color")
        self._register("pada", self.pada, "Printf-style format (different from Ika.pada=reverse string)")
        self._register("iyipada", self.iyipada, "Format with placeholders")
        
        # Advanced output
        self._register("tabili", self.tabili, "Print table")
        self._register("ilosiwaju", self.ilosiwaju, "Progress bar")
        self._register("yipo", self.yipo, "Spinner animation")
        self._register("apoti", self.apoti, "Print in box")
        self._register("ila", self.ila, "Print horizontal line")
        
        # Debugging
        self._register("asise", self.asise, "Print to stderr")
        self._register("iru", self.iru, "Print type of value")
        
        # Spec Functions
        self._register("san", self.san, "Stream/Flush")
        self._register("pe", self.pe, "Alert/Beep")
        self._register("kigbe", self.kigbe, "Error Log")
    
    # =========================================================================
    # BASIC I/O
    # =========================================================================
    
    def so(self, *args):
        """Print to console (no newline)."""
        print(*args, end='')
    
    def fo(self, *args):
        """Print with newline."""
        print(*args)
    
    def gbo(self, prompt: str = "") -> str:
        """Read input from user."""
        return input(prompt)
    
    def gbo_nomba(self, prompt: str = "") -> int:
        """Read integer input."""
        try:
            return int(input(prompt))
        except ValueError:
            return 0
    
    def gbo_odidi(self, prompt: str = "") -> float:
        """Read float input."""
        try:
            return float(input(prompt))
        except ValueError:
            return 0.0
    
    def mo(self):
        """Clear screen."""
        os.system('cls' if os.name == 'nt' else 'clear')
    
    # =========================================================================
    # FORMATTING
    # =========================================================================
    
    def awo(self, text: str, color: str = "white"):
        """Print with ANSI color."""
        colors = {
            "red": "\033[91m", "green": "\033[92m", "yellow": "\033[93m",
            "blue": "\033[94m", "magenta": "\033[95m", "cyan": "\033[96m",
            "white": "\033[97m", "bold": "\033[1m", "reset": "\033[0m"
        }
        c = colors.get(color, colors["white"])
        print(f"{c}{text}{colors['reset']}")
    
    def pada(self, template: str, *args) -> str:
        """Printf-style format: pada('%s is %d years old', 'Ade', 25)."""
        result = template % args
        print(result)
        return result
    
    def iyipada(self, template: str, **kwargs) -> str:
        """Format with named placeholders: iyipada('{name} is {age}', name='Ade', age=25)."""
        result = template.format(**kwargs)
        print(result)
        return result
    
    # =========================================================================
    # ADVANCED OUTPUT
    # =========================================================================
    
    def tabili(self, headers: List[str], rows: List[List[Any]], border: str = "─"):
        """Print formatted table."""
        # Calculate column widths
        widths = [len(str(h)) for h in headers]
        for row in rows:
            for i, cell in enumerate(row):
                if i < len(widths):
                    widths[i] = max(widths[i], len(str(cell)))
        
        # Print header
        line = border * (sum(widths) + 3 * len(widths) + 1)
        print(line)
        header_row = "│ " + " │ ".join(str(h).ljust(widths[i]) for i, h in enumerate(headers)) + " │"
        print(header_row)
        print(line)
        
        # Print rows
        for row in rows:
            cells = " │ ".join(str(cell).ljust(widths[i]) for i, cell in enumerate(row))
            print(f"│ {cells} │")
        
        print(line)
    
    def ilosiwaju(self, current: int, total: int, width: int = 40, label: str = ""):
        """Print progress bar: ilosiwaju(50, 100) -> [████████████████████░░░░░░░░░░░░░░░░░░░░] 50%"""
        percent = current / total if total > 0 else 0
        filled = int(width * percent)
        bar = "█" * filled + "░" * (width - filled)
        pct = int(percent * 100)
        sys.stdout.write(f"\r{label}[{bar}] {pct}%")
        sys.stdout.flush()
        if current >= total:
            print()  # Newline at end
    
    def yipo(self, message: str = "Loading", duration: float = 2.0):
        """Spinner animation."""
        chars = "⠋⠙⠹⠼⠴⠦⠧⠇⠏"
        end_time = time.time() + duration
        i = 0
        while time.time() < end_time:
            sys.stdout.write(f"\r{chars[i % len(chars)]} {message}...")
            sys.stdout.flush()
            time.sleep(0.1)
            i += 1
        sys.stdout.write("\r" + " " * (len(message) + 10) + "\r")
        sys.stdout.flush()
    
    def apoti(self, text: str, padding: int = 1):
        """Print text in a box."""
        lines = text.split('\n')
        max_len = max(len(line) for line in lines)
        width = max_len + padding * 2
        
        print("╔" + "═" * width + "╗")
        for line in lines:
            print("║" + " " * padding + line.ljust(max_len) + " " * padding + "║")
        print("╚" + "═" * width + "╝")
    
    def ila(self, char: str = "─", width: int = 60):
        """Print horizontal line."""
        print(char * width)
    
    # =========================================================================
    # DEBUGGING & SPEC IMPLEMENTATION
    # =========================================================================
    
    def asise(self, *args):
        """Print to stderr (error output)."""
        self.kigbe(*args)
    
    def iru(self, value: Any) -> str:
        """Print and return type of value."""
        t = type(value).__name__
        print(f"[Ìrosù] Type: {t}")
        return t

    def san(self):
        """san() - Stream/Flush."""
        sys.stdout.flush()
        sys.stderr.flush()

    def pe(self):
        """pè() - Alert/Beep."""
        print('\a') # Bell character

    def kigbe(self, *args):
        """kígbe() - Error Log."""
        print("[ERROR]", *args, file=sys.stderr)
