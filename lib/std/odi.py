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
    """The Womb - File operations and data storage (SECURITY HARDENED)."""
    
    # Security: Safe directories whitelist
    MAX_FILE_SIZE = 10 * 1024 * 1024  # 10MB
    BLOCKED_EXTENSIONS = {
        '.exe', '.dll', '.so', '.dylib', '.sh', '.bat', '.cmd',
        '.py', '.js', '.php', '.jsp', '.asp', '.aspx', '.rb', '.pl'
    }
    
    def __init__(self):
        super().__init__("Òdí", "1001", "The Vessel - Files & Storage")
        self._handles = {}
        self._data = {}
        self._active_handle = None
        self._active_path = None
        
        # Define safe directories at init time
        from pathlib import Path
        self._safe_dirs = [
            Path.cwd(),
            Path.cwd() / 'data',
            Path.cwd() / 'output',
        ]
        
        # High-level API
        self._register("si", self.si, "Open file")
        self._register("pa", self.pa, "Close file")
        self._register("ka", self.ka, "Read entire file contents")
        self._register("ka_ila", self.ka_ila, "Read file line by line")
        self._register("ka_nomba", self.ka_nomba, "Read next number from file")
        self._register("ko", self.ko, "Write to file")
        self._register("ko_nomba", self.ko_nomba, "Write number to file")
        self._register("fi", self.fi, "Append to file")
        self._register("wa", self.wa, "Check if file exists")
        self._register("pa_faili", self.pa_faili, "Delete file")
        self._register("akojo", self.akojo, "List directory")
        self._register("da_folder", self.da_folder, "Create directory")
        self._register("fi_data", self.fi_data, "Store key-value")
        self._register("gba_data", self.gba_data, "Retrieve key-value")
        
        # Spec Functions
        self._register("pamo", self.pamo, "Save/Commit (Flush)")
        self._register("ti", self.ti, "Close/Lock (Alias)")
        
        self.OPCODES = {
            "F_OPEN": "10011111",
            "F_WRITE": "10011100",
            "F_READ": "10010110",
            "F_CLOSE": "10010000",
        }
    
    def _validate_path(self, path: str, mode: str = 'r'):
        """Validate file path for security - audit hardened."""
        from pathlib import Path
        
        if not path:
            raise ValueError("Empty path")
        
        try:
            abs_path = Path(path).resolve()
        except (OSError, ValueError) as e:
            raise ValueError(f"Invalid path: {e}")
        
        # Block dangerous extensions
        if abs_path.suffix.lower() in self.BLOCKED_EXTENSIONS:
            raise ValueError(f"Blocked file type: {abs_path.suffix}")
        
        # Check if within safe directories
        is_safe = any(
            str(abs_path).startswith(str(safe_dir.resolve()))
            for safe_dir in self._safe_dirs
        )
        
        if not is_safe:
            raise ValueError(f"Path outside safe directories")
        
        # Block hidden files for write operations
        if mode in ('w', 'a') and abs_path.name.startswith('.'):
            raise ValueError("Cannot write to hidden files")
        
        return abs_path
    
    # =========================================================================
    # HIGH-LEVEL API (SECURITY HARDENED)
    # =========================================================================
    
    def si(self, path: str, mode: str = "r") -> bool:
        """Open file (SECURED)."""
        try:
            safe_path = self._validate_path(path, mode)
            self._handles[path] = open(safe_path, mode, encoding='utf-8')
            self._active_handle = self._handles[path]
            self._active_path = path
            print(f"[Òdí] Opened: '{path}'")
            return True
        except Exception as e:
            print(f"[Security] File access denied: {e}")
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
        """Read entire file (SECURED)."""
        try:
            safe_path = self._validate_path(path, 'r')
            # Check file size
            if safe_path.stat().st_size > self.MAX_FILE_SIZE:
                return f"Error: File too large (max {self.MAX_FILE_SIZE} bytes)"
            with open(safe_path, 'r', encoding='utf-8') as f:
                return f.read()
        except Exception as e:
            return f"[Security] Read denied: {e}"
    
    def ka_ila(self, path: str) -> List[str]:
        """Read file line by line (SECURED)."""
        try:
            safe_path = self._validate_path(path, 'r')
            if safe_path.stat().st_size > self.MAX_FILE_SIZE:
                return []
            with open(safe_path, 'r', encoding='utf-8') as f:
                return [line.rstrip('\n') for line in f]
        except Exception:
            return []
    
    def ka_nomba(self) -> Optional[int]:
        """Read next line as number from active file."""
        if self._active_handle and not self._active_handle.closed:
            try:
                line = self._active_handle.readline()
                if line:
                    value = int(line.strip())
                    return value
            except (ValueError, Exception):
                pass
        return None
    
    def ko(self, path: str, content: str):
        """Write content to file (SECURED)."""
        try:
            safe_path = self._validate_path(path, 'w')
            if len(content) > self.MAX_FILE_SIZE:
                print(f"[Security] Content too large")
                return
            with open(safe_path, 'w', encoding='utf-8') as f:
                f.write(content)
        except Exception as e:
            print(f"[Security] Write denied: {e}")
    
    def ko_nomba(self, value: int):
        """Write number to active file."""
        if self._active_handle and not self._active_handle.closed:
            self._active_handle.write(f"{value}\n")
        else:
            print("[Òdí] Error: File not open")
    
    def fi(self, path: str, content: str):
        """Append content to file (SECURED)."""
        try:
            safe_path = self._validate_path(path, 'a')
            with open(safe_path, 'a', encoding='utf-8') as f:
                f.write(content + '\n')
        except Exception as e:
            print(f"[Security] Append denied: {e}")
    
    def wa(self, path: str) -> bool:
        """Check if file exists (SECURED)."""
        try:
            safe_path = self._validate_path(path, 'r')
            return safe_path.exists()
        except:
            return False
    
    def pa_faili(self, path: str) -> bool:
        """Delete file (SECURED)."""
        try:
            safe_path = self._validate_path(path, 'w')
            safe_path.unlink()
            return True
        except Exception as e:
            print(f"[Security] Delete denied: {e}")
            return False
    
    def akojo(self, path: str = ".") -> List[str]:
        """List directory (SECURED)."""
        try:
            safe_path = self._validate_path(path, 'r')
            # Only show non-hidden files
            return [f.name for f in safe_path.iterdir() if not f.name.startswith('.')]
        except Exception:
            return []
    
    def da_folder(self, path: str) -> bool:
        """Create directory (SECURED)."""
        try:
            safe_path = self._validate_path(path, 'w')
            safe_path.mkdir(parents=True, exist_ok=True)
            return True
        except Exception as e:
            print(f"[Security] Mkdir denied: {e}")
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
