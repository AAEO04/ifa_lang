# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ÒDÍ - THE VESSEL (1001)                                            ║
║                    File Operations & Data Storage                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Merged from: opon_fileio.py                                                 ║
║  Instructions: F_OPEN, F_WRITE, F_READ, F_CLOSE                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
from typing import Any, List, Optional

from .base import OduModule


class OdiDomain(OduModule):
    """The Womb - File operations and data storage."""
    
    def __init__(self):
        super().__init__("Òdí", "1001", "The Vessel - Files & Storage")
        self._handles = {}
        self._data = {}
        self._active_handle = None  # For VM-style sequential operations
        self._active_path = None
        
        # High-level API
        self._register("si", self.si, "Open file")
        self._register("pa", self.pa, "Close file")
        self._register("ka", self.ka, "Read entire file contents (different from Ika.ka=string length)")
        self._register("ka_ila", self.ka_ila, "Read file line by line")
        self._register("ka_nomba", self.ka_nomba, "Read next number from file")
        self._register("ko", self.ko, "Write to file")
        self._register("ko_nomba", self.ko_nomba, "Write number to file")
        self._register("fi", self.fi, "Append to file")
        self._register("wa", self.wa, "Check if file exists (different from Ika.wa=find substring, Ogunda.wa=find in array)")
        self._register("pa_faili", self.pa_faili, "Delete file")
        self._register("akojo", self.akojo, "List directory")
        self._register("da_folder", self.da_folder, "Create directory")
        self._register("fi_data", self.fi_data, "Store key-value")
        self._register("gba_data", self.gba_data, "Retrieve key-value")
        
        # Spec Functions
        self._register("pamo", self.pamo, "Save/Commit (Flush)")
        self._register("ti", self.ti, "Close/Lock (Alias)")
        
        # VM-style opcodes (for bytecode execution)
        self.OPCODES = {
            "F_OPEN": "10011111",   # Odi-Ogbe (Open)
            "F_WRITE": "10011100",  # Odi-Irosu (Write)
            "F_READ": "10010110",   # Odi-Iwori (Read)
            "F_CLOSE": "10010000",  # Odi-Oyeku (Close)
        }
    
    # =========================================================================
    # HIGH-LEVEL API
    # =========================================================================
    
    def si(self, path: str, mode: str = "r") -> bool:
        """Open file for reading or writing. Returns success status."""
        try:
            self._handles[path] = open(path, mode, encoding='utf-8')
            self._active_handle = self._handles[path]
            self._active_path = path
            print(f"[Òdí] Womb Opened: '{path}' in mode '{mode}'.")
            return True
        except Exception as e:
            print(f"[Òdí] Error: Could not open vessel. {e}")
            return False
    
    def pa(self, path: str = None):
        """Close file. If path not given, closes active handle."""
        target = path or self._active_path
        if target and target in self._handles:
            self._handles[target].close()
            del self._handles[target]
            print(f"[Òdí] Vessel Sealed: '{target}'")
            if target == self._active_path:
                self._active_handle = None
                self._active_path = None
    
    def ka(self, path: str) -> str:
        """Read entire file."""
        try:
            with open(path, 'r', encoding='utf-8') as f:
                return f.read()
        except Exception as e:
            return f"Error: {e}"
    
    def ka_ila(self, path: str) -> List[str]:
        """Read file line by line."""
        try:
            with open(path, 'r', encoding='utf-8') as f:
                return [line.rstrip('\n') for line in f]
        except Exception:
            return []
    
    def ka_nomba(self) -> Optional[int]:
        """Read next line as number from active file (VM-style)."""
        if self._active_handle and not self._active_handle.closed:
            try:
                line = self._active_handle.readline()
                if line:
                    value = int(line.strip())
                    print(f"[Òdí] Remembered: Retrieved '{value}' from history.")
                    return value
            except (ValueError, Exception):
                pass
        else:
            print("[Òdí] Error: The vessel is not open!")
        return None
    
    def ko(self, path: str, content: str):
        """Write content to file (overwrite)."""
        with open(path, 'w', encoding='utf-8') as f:
            f.write(content)
    
    def ko_nomba(self, value: int):
        """Write number to active file (VM-style)."""
        if self._active_handle and not self._active_handle.closed:
            data = f"{value}\n"
            self._active_handle.write(data)
            print(f"[Òdí] Carved '{value}' into the vessel.")
        else:
            print("[Òdí] Error: The vessel is not open!")
    
    def fi(self, path: str, content: str):
        """Append content to file."""
        with open(path, 'a', encoding='utf-8') as f:
            f.write(content + '\n')
    
    def wa(self, path: str) -> bool:
        """Check if file exists."""
        return os.path.exists(path)
    
    def pa_faili(self, path: str) -> bool:
        """Delete file."""
        try:
            os.remove(path)
            return True
        except (IOError, OSError):
            return False
    
    def akojo(self, path: str = ".") -> List[str]:
        """List directory contents."""
        return os.listdir(path)
    
    def da_folder(self, path: str) -> bool:
        """Create directory."""
        try:
            os.makedirs(path, exist_ok=True)
            return True
        except (IOError, OSError):
            return False
    
    def fi_data(self, key: str, value: Any):
        """Store in internal map."""
        self._data[key] = value
    
    def gba_data(self, key: str, default: Any = None) -> Any:
        """Retrieve from internal map."""
        return self._data.get(key, default)

    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================
    
    def pamo(self):
        """pamọ́() - Save/Commit (Flush)."""
        if self._active_handle and not self._active_handle.closed:
            self._active_handle.flush()
            try:
                os.fsync(self._active_handle.fileno())
            except (IOError, OSError):
                pass

    def ti(self):
        """tì() - Close/Lock (Alias for pa)."""
        self.pa()
    
    # =========================================================================
    # VM-STYLE OPERATIONS (for bytecode execution)
    # =========================================================================
    
    def vm_open(self, mode: int, filename: str = "legacy.ifa") -> bool:
        """VM-style open: mode 0=read, 1=write."""
        m = 'w' if mode == 1 else 'r'
        return self.si(filename, m)
    
    def vm_write(self, value: int):
        """VM-style write: write ISALE value to file."""
        self.ko_nomba(value)
    
    def vm_read(self) -> Optional[int]:
        """VM-style read: read next number into ISALE."""
        return self.ka_nomba()
    
    def vm_close(self):
        """VM-style close."""
        self.pa()
