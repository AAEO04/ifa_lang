# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           IFÁ-LANG EXTENSION LIBRARY - ODÙ ÀFIKÚN                            ║
║                    256/4096 Compound Odù Registry                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  This module manages compound Odù combinations.                              ║
║                                                                              ║
║  SUPPORTED PATTERNS:                                                         ║
║    2-Component (256):  Parent_Child          e.g., otura_ika                ║
║    3-Component (4096): Grandparent_Parent_Child  e.g., ogbe_otura_ika       ║
║                                                                              ║
║  LAYERS:                                                                     ║
║    lib/std/     -> 16 Principal Odù (immutable core)                        ║
║    lib/ext/     -> Official compound extensions (this directory)            ║
║    project/odu/ -> User-created compounds (per-project)                     ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import importlib
from typing import Dict, Any, Optional, List


# =============================================================================
# THE 16 VALID PRINCIPAL ODÙ NAMES
# =============================================================================
VALID_ODU = {
    "ogbe", "oyeku", "iwori", "odi", "irosu", "owonrin", "obara", "okanran",
    "ogunda", "osa", "ika", "oturupon", "otura", "irete", "ose", "ofun"
}

# Hex codes for opcodes
ODU_HEX = {
    "ogbe": 0x0, "oyeku": 0x1, "iwori": 0x2, "odi": 0x3,
    "irosu": 0x4, "owonrin": 0x5, "obara": 0x6, "okanran": 0x7,
    "ogunda": 0x8, "osa": 0x9, "ika": 0xA, "oturupon": 0xB,
    "otura": 0xC, "irete": 0xD, "ose": 0xE, "ofun": 0xF
}


# =============================================================================
# COMPOUND ODÙ REGISTRY
# =============================================================================

class CompoundRegistry:
    """
    Registry for Compound Odù modules.
    
    Supports:
      - 2-component: Parent_Child (256 combinations)
      - 3-component: Grandparent_Parent_Child (4096 combinations)
    """
    
    def __init__(self):
        # 2-component compounds (256: Parent_Child)
        self._compounds_256: Dict[str, Any] = {}
        
        # 3-component compounds (4096: Grandparent_Parent_Child)
        self._compounds_4096: Dict[str, Any] = {}
        
        # User compounds (from project odu/ folder)
        self._user: Dict[str, Any] = {}
        
        # Load official compounds
        self._load_official()
    
    def _load_official(self):
        """Load official compound modules from lib/ext/"""
        # Scan this directory for compound files
        ext_dir = os.path.dirname(__file__)
        
        for filename in os.listdir(ext_dir):
            if not filename.endswith(".py"):
                continue
            if filename.startswith("_") or filename.startswith("TEMPLATE"):
                continue
            
            module_name = filename[:-3]  # Remove .py
            parts = module_name.split("_")
            
            # Validate all parts are valid Odù names
            if not all(p.lower() in VALID_ODU for p in parts):
                continue
            
            try:
                # Import the module
                module = importlib.import_module(f".{module_name}", package="lib.ext")
                
                # Find the main class (CamelCase of filename)
                class_name = "".join(word.title() for word in parts)
                
                if hasattr(module, class_name):
                    cls = getattr(module, class_name)
                    
                    if len(parts) == 2:
                        # 2-component (256)
                        self._compounds_256[module_name] = cls
                        opcode = self._calculate_opcode_8(parts[0], parts[1])
                        cls._opcode = opcode
                    elif len(parts) == 3:
                        # 3-component (4096)
                        self._compounds_4096[module_name] = cls
                        opcode = self._calculate_opcode_12(parts[0], parts[1], parts[2])
                        cls._opcode = opcode
                        
            except Exception as e:
                print(f"⚠️ Failed to load {module_name}: {e}")
    
    def _calculate_opcode_8(self, parent: str, child: str) -> int:
        """Calculate 8-bit opcode for 2-component compound."""
        p = ODU_HEX.get(parent.lower(), 0)
        c = ODU_HEX.get(child.lower(), 0)
        return (p << 4) | c
    
    def _calculate_opcode_12(self, grandparent: str, parent: str, child: str) -> int:
        """Calculate 12-bit opcode for 3-component compound."""
        g = ODU_HEX.get(grandparent.lower(), 0)
        p = ODU_HEX.get(parent.lower(), 0)
        c = ODU_HEX.get(child.lower(), 0)
        return (g << 8) | (p << 4) | c
    
    def register(self, name: str, module_class: Any):
        """
        Register a user-defined compound module.
        
        Args:
            name: Compound name (e.g., "otura_ika" or "ogbe_otura_ika")
            module_class: The class implementing the compound
        
        Example:
            registry.register("otura_osa", MyWebSocketModule)
            registry.register("ogbe_otura_osa", MySystemWebSocketModule)
        """
        parts = name.lower().split("_")
        
        # Validate all parts are valid Odù names
        if not all(p in VALID_ODU for p in parts):
            invalid = [p for p in parts if p not in VALID_ODU]
            raise ValueError(f"Invalid Odù name(s): {invalid}")
        
        if len(parts) == 2:
            self._user[name.lower()] = module_class
            module_class._opcode = self._calculate_opcode_8(parts[0], parts[1])
        elif len(parts) == 3:
            self._user[name.lower()] = module_class
            module_class._opcode = self._calculate_opcode_12(parts[0], parts[1], parts[2])
        else:
            raise ValueError(f"Compound must have 2 or 3 components: {name}")
        
        print(f"📦 [Registry] Registered: {name} (opcode: 0x{module_class._opcode:X})")
    
    def get(self, name: str) -> Optional[Any]:
        """
        Get a compound module by name.
        
        Args:
            name: Compound name (e.g., "otura_ika" or "ogbe_otura_ika")
            
        Returns:
            The module class or None if not found
        """
        name_lower = name.lower()
        
        # Check 256 compounds
        if name_lower in self._compounds_256:
            return self._compounds_256[name_lower]
        
        # Check 4096 compounds
        if name_lower in self._compounds_4096:
            return self._compounds_4096[name_lower]
        
        # Check user-defined
        if name_lower in self._user:
            return self._user[name_lower]
        
        return None
    
    def get_by_opcode(self, opcode: int) -> Optional[Any]:
        """Get a compound by its opcode."""
        # Check 256 compounds (8-bit)
        for name, cls in self._compounds_256.items():
            if hasattr(cls, '_opcode') and cls._opcode == opcode:
                return cls
        
        # Check 4096 compounds (12-bit)
        for name, cls in self._compounds_4096.items():
            if hasattr(cls, '_opcode') and cls._opcode == opcode:
                return cls
        
        # Check user
        for name, cls in self._user.items():
            if hasattr(cls, '_opcode') and cls._opcode == opcode:
                return cls
        
        return None
    
    def list_256(self) -> Dict[str, Any]:
        """List all 2-component (256) compounds."""
        return self._compounds_256.copy()
    
    def list_4096(self) -> Dict[str, Any]:
        """List all 3-component (4096) compounds."""
        return self._compounds_4096.copy()
    
    def list_user(self) -> Dict[str, Any]:
        """List all user-registered compounds."""
        return self._user.copy()
    
    def list_all(self) -> Dict[str, Any]:
        """List all registered compounds."""
        all_compounds = {}
        all_compounds.update(self._compounds_256)
        all_compounds.update(self._compounds_4096)
        all_compounds.update(self._user)
        return all_compounds
    
    def load_from_project(self, project_path: str):
        """
        Load user compounds from a project's odu/ folder.
        
        Args:
            project_path: Path to the project root
        """
        odu_path = os.path.join(project_path, "odu")
        if not os.path.isdir(odu_path):
            return
        
        import sys
        if odu_path not in sys.path:
            sys.path.insert(0, odu_path)
        
        for filename in os.listdir(odu_path):
            if not filename.endswith(".py"):
                continue
            if filename.startswith("_"):
                continue
            
            module_name = filename[:-3]
            parts = module_name.split("_")
            
            # Must be 2 or 3 components
            if len(parts) not in (2, 3):
                continue
            
            # All parts must be valid Odù
            if not all(p.lower() in VALID_ODU for p in parts):
                continue
            
            try:
                module = importlib.import_module(module_name)
                class_name = "".join(word.title() for word in parts)
                
                if hasattr(module, class_name):
                    cls = getattr(module, class_name)
                    self.register(module_name, cls)
                    
            except Exception as e:
                print(f"⚠️ Failed to load user compound {module_name}: {e}")
    
    def help(self):
        """Print registry contents."""
        print("=== COMPOUND ODÙ REGISTRY ===\n")
        
        print("📦 256 Compounds (Parent_Child):")
        for name, cls in self._compounds_256.items():
            opcode = getattr(cls, '_opcode', 0)
            doc = (cls.__doc__ or "").split('\n')[0]
            print(f"  {name.ljust(20)} 0x{opcode:02X}  {doc}")
        
        print("\n📦 4096 Compounds (Grandparent_Parent_Child):")
        for name, cls in self._compounds_4096.items():
            opcode = getattr(cls, '_opcode', 0)
            doc = (cls.__doc__ or "").split('\n')[0]
            print(f"  {name.ljust(25)} 0x{opcode:03X}  {doc}")
        
        print("\n📦 User Compounds:")
        for name, cls in self._user.items():
            opcode = getattr(cls, '_opcode', 0)
            print(f"  {name.ljust(25)} 0x{opcode:03X}")


# =============================================================================
# GLOBAL REGISTRY INSTANCE
# =============================================================================

registry = CompoundRegistry()


# =============================================================================
# CONVENIENCE FUNCTIONS
# =============================================================================

def get_compound(name: str):
    """Get a compound module by name."""
    return registry.get(name)

def register_compound(name: str, module_class: Any):
    """Register a user-defined compound."""
    registry.register(name, module_class)

def load_project_compounds(project_path: str):
    """Load all compounds from a project's odu/ folder."""
    registry.load_from_project(project_path)

def list_compounds():
    """List all registered compounds."""
    registry.help()
