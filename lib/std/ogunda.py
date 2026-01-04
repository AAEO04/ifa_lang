# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ÒGÚNDÁ - THE CUTTER (1110)                                         ║
║                    Arrays, Lists & Process Control                           ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import subprocess
from typing import Any, List, Callable

from .base import OduModule


class OgundaDomain(OduModule):
    """The Iron Cutter - Arrays and process control."""
    
    def __init__(self):
        super().__init__("Ògúndá", "1110", "The Cutter - Arrays & Process")
        self._arrays = {}
        
        # Array lifecycle
        self._register("ge", self.ge, "Create array (different from Ika.ge=slice, Oturupon.ge=divide)")
        self._register("da", self.da, "Create list from values")
        self._register("nu", self.nu, "Clear array (different from Ika.nu=trim, Ose.nu=clear canvas)")
        self._register("eda", self.eda, "Copy array")
        
        # Stack operations
        self._register("fi", self.fi, "Push to array")
        self._register("gba", self.gba, "Pop from array (different from Ogbe.gba=args, Otura.gba=TCP recv)")
        self._register("fi_iwaju", self.fi_iwaju, "Insert at front")
        self._register("gba_iwaju", self.gba_iwaju, "Remove from front")
        
        # Access
        self._register("wo", self.wo, "Get element at index")
        self._register("eto", self.eto, "Set element at index")
        self._register("gigun", self.gigun, "Array length")
        self._register("wa", self.wa, "Find element index (different from Ika.wa=find substring, Odi.wa=file exists)")
        self._register("ni_ninu", self.ni_ninu, "Check if contains")
        
        # Transformation
        self._register("seto", self.seto, "Sort array")
        self._register("yipada", self.yipada, "Reverse array")
        self._register("dapo", self.dapo, "Join/concat arrays (different from Ika.dapo=join string, Obara.dapo=shuffle)")
        self._register("ge_lati", self.ge_lati, "Slice array")
        self._register("oto", self.oto, "Unique elements")
        
        # Functional
        self._register("alemo", self.alemo, "Filter array")
        self._register("maapu", self.maapu, "Map over array")
        self._register("dinku", self.dinku, "Reduce array")
        self._register("kokan", self.kokan, "For each element")
        self._register("gbogbo", self.gbogbo, "All match predicate")
        self._register("eyikeyi", self.eyikeyi, "Any match predicate")
        
        # Process control
        self._register("si", self.si, "Execute command")
        self._register("si_pipe", self.si_pipe, "Execute and capture output")
        
        # Spec Functions
        self._register("ya", self.ya, "Split (at index)")
        self._register("to", self.to, "Sort (Alias)")
        self._register("mu", self.mu, "Pick/Pop (Alias)")
        # ge exists as Create Array, spec says Slice/Cut. I will alias ge to ge_lati if it's called with args, 
        # but Python doesn't support easy overloading. 
        # Spec says "ge() (Slice/Cut)". Existing "ge" is "Create array".
        # I will change 'ge' to be smart: if 1 arg (name) -> create empty? Or use a new method name for the existing one?
        # User said "do not change any name". 
        # So 'ge' must remain 'Create array'. 
        # I will add 'ge_fun_ge' or purely alias 'ge' to Slice if arguments match? 
        # Actually, `ge(name, size, fill)` vs `ge(arr, start, end)`.
        # I cannot overload easily. I'll stick to `ge_lati` for Slice and leave `ge` as Create.
        # But wait, user spec explicitly maps Ògúndá.gé to Slice/Cut. 
        # I will add `ge_slice` as alias to `ge_lati` and leave `ge` alone to avoid breaking `ge("myList", 10)`.
        # Wait, I can allow `ge` to handle both?
        # def ge(self, arg1, arg2=None, arg3=None):
        #    if isinstance(arg1, list) or (isinstance(arg1, str) and arg2 is None): ...
        # Too risky. I will just add the missing ones `ya`, `to`, `mu`.
    
    # =========================================================================
    # ARRAY LIFECYCLE
    # =========================================================================
    
    def ge(self, name: str, size: int = 0, fill: Any = 0) -> List:
        """Create array with optional size and fill value."""
        self._arrays[name] = [fill] * size
        return self._arrays[name]
    
    def da(self, *values) -> List:
        """Create list from values: da(1, 2, 3) -> [1, 2, 3]."""
        return list(values)
    
    def nu(self, name: str):
        """Clear array."""
        if name in self._arrays:
            self._arrays[name] = []
    
    def eda(self, name: str) -> List:
        """Copy array."""
        return self._arrays.get(name, []).copy()
    
    # =========================================================================
    # STACK OPERATIONS
    # =========================================================================
    
    def fi(self, name: str, value: Any):
        """Push to array (append)."""
        if name not in self._arrays:
            self._arrays[name] = []
        self._arrays[name].append(value)
    
    def gba(self, name: str) -> Any:
        """Pop from array (end)."""
        if name in self._arrays and self._arrays[name]:
            return self._arrays[name].pop()
        return None
    
    def fi_iwaju(self, name: str, value: Any):
        """Insert at front."""
        if name not in self._arrays:
            self._arrays[name] = []
        self._arrays[name].insert(0, value)
    
    def gba_iwaju(self, name: str) -> Any:
        """Remove from front."""
        if name in self._arrays and self._arrays[name]:
            return self._arrays[name].pop(0)
        return None
    
    # =========================================================================
    # ACCESS
    # =========================================================================
    
    def wo(self, name: str, index: int) -> Any:
        """Get element at index."""
        if name in self._arrays and 0 <= index < len(self._arrays[name]):
            return self._arrays[name][index]
        return None
    
    def eto(self, name: str, index: int, value: Any):
        """Set element at index."""
        if name in self._arrays and 0 <= index < len(self._arrays[name]):
            self._arrays[name][index] = value
    
    def gigun(self, name: str) -> int:
        """Array length."""
        return len(self._arrays.get(name, []))
    
    def wa(self, name: str, value: Any) -> int:
        """Find element index (-1 if not found)."""
        arr = self._arrays.get(name, [])
        return arr.index(value) if value in arr else -1
    
    def ni_ninu(self, name: str, value: Any) -> bool:
        """Check if array contains value."""
        return value in self._arrays.get(name, [])
    
    # =========================================================================
    # TRANSFORMATION
    # =========================================================================
    
    def seto(self, arr: List, reverse: bool = False) -> List:
        """Sort array. Works on direct list or by name."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        return sorted(arr, reverse=reverse)
    
    def yipada(self, arr: List) -> List:
        """Reverse array."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        return list(reversed(arr))
    
    def dapo(self, *lists) -> List:
        """Join/concatenate arrays."""
        result = []
        for lst in lists:
            if isinstance(lst, str):
                lst = self._arrays.get(lst, [])
            result.extend(lst if isinstance(lst, list) else [lst])
        return result
    
    def ge_lati(self, arr: List, start: int, end: int = None) -> List:
        """Slice array."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        return arr[start:end]
    
    def oto(self, arr: List) -> List:
        """Unique elements (preserves order)."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        seen = set()
        return [x for x in arr if not (x in seen or seen.add(x))]
    
    # =========================================================================
    # FUNCTIONAL OPERATIONS
    # =========================================================================
    
    def alemo(self, arr: List, predicate: Callable = None) -> List:
        """Filter array with predicate function."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        if predicate is None:
            # Default: filter out falsy values
            return [x for x in arr if x]
        return [x for x in arr if predicate(x)]
    
    def maapu(self, arr: List, func: Callable = None) -> List:
        """Map function over array."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        if func is None:
            return arr
        return [func(x) for x in arr]
    
    def dinku(self, arr: List, func: Callable, initial: Any = None) -> Any:
        """Reduce array with function."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        from functools import reduce
        if initial is not None:
            return reduce(func, arr, initial)
        return reduce(func, arr)
    
    def kokan(self, arr: List, func: Callable):
        """Execute function for each element."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        for item in arr:
            func(item)
    
    def gbogbo(self, arr: List, predicate: Callable) -> bool:
        """All elements match predicate."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        return all(predicate(x) for x in arr)
    
    def eyikeyi(self, arr: List, predicate: Callable) -> bool:
        """Any element matches predicate."""
        if isinstance(arr, str):
            arr = self._arrays.get(arr, [])
        return any(predicate(x) for x in arr)
    
    # SECURE: Very restrictive command whitelist with arg limits
    ALLOWED_COMMANDS = {
        'echo': {'max_args': 5},
        'date': {'max_args': 0},
        'pwd': {'max_args': 0},
        'whoami': {'max_args': 0},
        'hostname': {'max_args': 0},
    }
    
    # Block ALL dangerous characters
    BLOCKED_CHARS = set(';&|<>`$(){}[]\n\r\\\'\"')
    MAX_CMD_LENGTH = 100
    
    def _validate_command(self, cmd: str) -> bool:
        """Strict command validation - audit hardened."""
        if not cmd or len(cmd) > self.MAX_CMD_LENGTH:
            print("[Security] Command empty or too long")
            return False
        
        # Block dangerous characters
        if any(c in self.BLOCKED_CHARS for c in cmd):
            print("[Security] Illegal character in command")
            return False
        
        parts = cmd.strip().split()
        if not parts:
            return False
        
        base_cmd = parts[0].lower()
        
        # Check if command is whitelisted
        if base_cmd not in self.ALLOWED_COMMANDS:
            print(f"[Security] Command not allowed: {base_cmd}")
            print(f"[Security] Allowed: {', '.join(self.ALLOWED_COMMANDS.keys())}")
            return False
        
        # Validate argument count
        max_args = self.ALLOWED_COMMANDS[base_cmd].get('max_args', 0)
        if len(parts) - 1 > max_args:
            print(f"[Security] Too many arguments for {base_cmd}")
            return False
        
        # Validate arguments are alphanumeric only
        for arg in parts[1:]:
            if not all(c.isalnum() or c in ' _-.' for c in arg):
                print(f"[Security] Illegal characters in argument")
                return False
        
        return True
    
    def si(self, cmd: str, use_sandbox: bool = True) -> int:
        """Execute shell command (SECURE - uses Ìgbálẹ̀ sandbox by default)."""
        if not self._validate_command(cmd):
            return -1
        
        try:
            import subprocess
            import os
            parts = cmd.strip().split()
            
            # Use sandbox for isolation if available
            if use_sandbox:
                try:
                    from src.sandbox import IgbaleRuntime, OgbeConfig
                    
                    runtime = IgbaleRuntime()
                    config = OgbeConfig(
                        name=f"ogunda_cmd_{int(time.time())}",
                        command=parts,
                        auto_remove=True
                    )
                    container = runtime.create(config)
                    container.iwori_monitor.max_runtime = 5  # 5 second limit
                    container.start()
                    
                    if container.process:
                        try:
                            container.process.wait(timeout=5)
                            result = container.process.returncode
                        except subprocess.TimeoutExpired:
                            container.stop(force=True)
                            print("[Security] Command timeout (sandbox)")
                            result = -1
                    else:
                        result = 0
                    
                    container.stop()
                    runtime.gc()
                    return result
                    
                except ImportError:
                    pass  # Fall back to direct execution
            
            # Fallback: direct execution with restricted env
            result = subprocess.run(
                parts,
                capture_output=True,
                timeout=5,
                text=True,
                env={'PATH': '/usr/bin:/bin'} if os.name != 'nt' else None
            )
            return result.returncode
        except subprocess.TimeoutExpired:
            print("[Security] Command timeout")
            return -1
        except Exception as e:
            print(f"[Security] Command failed: {e}")
            return -1
    
    def si_pipe(self, cmd: str) -> str:
        """Execute command and capture output (HEAVILY restricted)."""
        if not self._validate_command(cmd):
            return "[Security] Command blocked"
        try:
            import subprocess
            import os
            parts = cmd.strip().split()
            result = subprocess.run(
                parts,
                capture_output=True,
                text=True,
                timeout=5,
                env={'PATH': '/usr/bin:/bin'} if os.name != 'nt' else None
            )
            return result.stdout
        except subprocess.TimeoutExpired:
            return "[Security] Command timeout"
        except Exception as e:
            return f"Error: {e}"

    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================
    
    def ya(self, arr: List, index: int) -> List[List]:
        """yà() - Split (at index). Returns [before, after]."""
        if isinstance(arr, str): arr = self._arrays.get(arr, [])
        if index < 0 or index >= len(arr): return [arr, []]
        return [arr[:index], arr[index:]]

    def to(self, arr: List) -> List:
        """tò() - Sort (Alias for seto)."""
        return self.seto(arr)

    def mu(self, arr: List, index: int = -1) -> Any:
        """mu() - Pick/Pop (Alias for gba/pop)."""
        # If List passed directly
        if isinstance(arr, list):
             try: return arr.pop(index)
             except: return None
        # If name passed
        return self.gba(arr)
