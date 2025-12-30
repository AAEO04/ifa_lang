# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                      ÒFÚN - THE ELDER (PERMISSIONS/META)                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Domain: Permissions, Configuration, Reflection, Root Access                 ║
║  English Aliases: Meta, Reflect, Root                                        ║
║  The Eldest Spirit - Authority, wisdom, and introspection                    ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import sys
from typing import Any, Dict, Optional

from .base import OduModule


class IfaException(Exception):
    """Custom exception for Ifá-Lang errors."""
    pass


class OfunDomain(OduModule):
    """
    Òfún - The Elder / The Wise
    Handles permissions, configuration, and reflection.
    """
    
    # Configuration store (class-level for persistence)
    _config: Dict[str, Any] = {}
    _elevated: bool = False
    
    def __init__(self):
        super().__init__("Òfún", "0101", "The Elder - Permissions & Meta")
        
        # Permissions
        self._register("ase", self.ase, "Sudo/Elevate permissions")
        self._register("fun", self.fun, "Grant permission")
        
        # Configuration
        self._register("ka", self.ka, "Read config value (different from Ika.ka=string length, Odi.ka=read file)")
        self._register("fi", self.fi, "Set config value")
        
        # Reflection
        self._register("iru", self.iru, "Get type of value (typeof)")
        self._register("oruko", self.oruko, "Get current user/hostname")
        
        # Spec Functions
        self._register("ka_iwe", self.ka_iwe, "Read manifest/docs")
    
    # =========================================================================
    # PERMISSIONS
    # =========================================================================
    
    def ase(self) -> bool:
        """
        àṣẹ (Sudo/Elevate) - Request elevated permissions
        
        Yoruba: Òfún.àṣẹ()
        English: Root.sudo(), Meta.elevate()
        """
        OfunDomain._elevated = True
        print("[Òfún] Elevated permissions granted")
        return True
    
    def fun(self, permission: str, granted: bool = True) -> bool:
        """
        fún (Grant) - Grant a permission
        
        Yoruba: Òfún.fún()
        English: Root.grant(), Meta.allow()
        """
        print(f"[Òfún] Permission '{permission}' = {granted}")
        return granted
    
    # =========================================================================
    # CONFIGURATION
    # =========================================================================
    
    def ka(self, key: str, default: Any = None) -> Any:
        """
        kà (Read Config) - Read configuration value
        
        Yoruba: Òfún.kà()
        English: Meta.config(), Config.get()
        """
        # First check internal config
        if key in OfunDomain._config:
            return OfunDomain._config[key]
        
        # Then check environment
        env_val = os.environ.get(key)
        if env_val is not None:
            return env_val
        
        return default
    
    def fi(self, key: str, value: Any) -> None:
        """
        fi (Set Config) - Set configuration value
        
        Yoruba: Òfún.fi()
        English: Config.set()
        """
        OfunDomain._config[key] = value
    
    # =========================================================================
    # REFLECTION
    # =========================================================================
    
    def iru(self, value: Any) -> str:
        """
        iru (Type Of) - Get the type name of a value
        
        Yoruba: Òfún.iru()
        English: Meta.type(), Reflect.typeof()
        """
        type_name = type(value).__name__
        
        # Map to Ifá types
        type_map = {
            'int': 'Int',
            'float': 'Float',
            'str': 'Str',
            'bool': 'Bool',
            'list': 'List',
            'dict': 'Map',
            'NoneType': 'Null',
        }
        
        return type_map.get(type_name, type_name)
    
    def oruko(self) -> str:
        """
        orúkọ (Name/Identity) - Get current user/hostname
        
        Yoruba: Òfún.orúkọ()
        English: System.user(), Meta.identity()
        """
        try:
            import getpass
            return getpass.getuser()
        except:
            return os.environ.get('USER', os.environ.get('USERNAME', 'unknown'))
    
    # =========================================================================
    # SPEC IMPLEMENTATION
    # =========================================================================
    
    def ka_iwe(self, module_name: str = None) -> str:
        """kà_ìwé() - Read Manifest/Docs."""
        return f"Manifest for {module_name or 'Ifá-Lang System'}"


# Module-level singleton and functions for backwards compatibility
_ofun = OfunDomain()

def ase() -> bool:
    return _ofun.ase()

def fun(permission: str, granted: bool = True) -> bool:
    return _ofun.fun(permission, granted)

def ka(key: str, default: Any = None) -> Any:
    return _ofun.ka(key, default)

def fi(key: str, value: Any) -> None:
    return _ofun.fi(key, value)

def iru(value: Any) -> str:
    return _ofun.iru(value)

def oruko() -> str:
    return _ofun.oruko()

def ka_iwe(module_name: str = None) -> str:
    return _ofun.ka_iwe(module_name)

# English aliases
sudo = ase
elevate = ase
grant = fun
allow = fun
config = ka
get = ka
set_config = fi
typeof = iru
user = oruko
identity = oruko
read_manifest = ka_iwe
