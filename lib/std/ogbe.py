# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           OGBÈ - THE OPENER (1111)                                           ║
║                    System Initialization & CLI Arguments                     ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import sys
import platform
import subprocess
import time
from datetime import datetime
from typing import List

from .base import OduModule


class OgbeDomain(OduModule):
    """The Light - System initialization, OS info, and Process control."""
    
    def __init__(self):
        super().__init__("Ogbè", "1111", "The Opener - System & CLI Args")
        self._args = sys.argv[1:]
        self._env = dict(os.environ)
        
        # Spec Functions: bi, gba, env, oruko
        self._register("bi", self.bi, "Initialize system (Boot)")
        self._register("gba", self.gba, "Get Input/Args")
        self._register("env", self.env, "Get environment variable")
        self._register("oruko", self.oruko, "Get Hostname/User")
        
        # Extra Utility
        self._register("version", self.version, "Get Ifá version")
        self._register("ero_ise", self.ero_ise, "Get OS name")
        self._register("bit", self.bit, "Get architecture")
        self._register("gba_arg", self.gba_arg, "Get specific argument by index")
        self._register("set_env", self.set_env, "Set environment variable")
        self._register("cwd", self.cwd, "Get current working directory")
        self._register("ile", self.ile, "Get home directory")
        self._register("pa_system", self.pa_system, "Shutdown/Reboot computer")
        self._register("jade", self.jade, "Exit program")
        self._register("sun", self.sun, "Sleep execution")
        self._register("pa_pip", self.pa_pip, "Install package (pip)")
    
    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================
    
    def bi(self) -> str:
        """bí() - System boot/init."""
        return f"Ifá System Initialized at {datetime.now()}"
        
    def gba(self, prompt: str = None) -> str:
        """gbà() - Get Input. If prompt provided, Acts as Input(), else returns all Args joined."""
        if prompt is not None:
             return input(prompt)
        return " ".join(self._args)
        
    def env(self, key: str, default: str = "") -> str:
        """env() - Get environment variable."""
        return self._env.get(key, default)
        
    def oruko(self) -> str:
        """orúkọ() - Get Hostname / User ID."""
        return platform.node()

    # =========================================================================
    # EXTRAS
    # =========================================================================
    
    def version(self) -> str: return "Ifá-Lang v1.0 (Amúlù Edition)"
    def ero_ise(self) -> str: return platform.system()
    def bit(self) -> str: return platform.machine()
    
    def gba_all(self) -> List[str]: return self._args # Legacy access to list
    
    def gba_arg(self, index: int, default: str = "") -> str:
        if 0 <= index < len(self._args): return self._args[index]
        return default
    
    def set_env(self, key: str, value: str):
        os.environ[key] = str(value)
        self._env[key] = str(value)
    
    def cwd(self) -> str: return os.getcwd()
    def ile(self) -> str: return os.path.expanduser("~")
    
    def pa_system(self, action: str = "shutdown"):
        """[DISABLED] System shutdown/reboot - disabled for security."""
        print("[Security] pa_system() is disabled for security reasons")
        print("[Security] Use operating system controls for shutdown/reboot")
        return False
    
    def jade(self, code: int = 0): sys.exit(code)
    def sun(self, seconds: float): time.sleep(seconds)
    
    def pa_pip(self, package: str):
        """Install pip package with validation."""
        # Security: Only allow alphanumeric package names
        if not package or not package.replace('-', '').replace('_', '').isalnum():
            print(f"[Security] Invalid package name: {package}")
            return False
        # Security: Block known dangerous packages
        blocked = {'os', 'sys', 'subprocess', 'eval', 'exec'}
        if package.lower() in blocked:
            print(f"[Security] Package blocked: {package}")
            return False
        subprocess.check_call([sys.executable, "-m", "pip", "install", package])
        return True
