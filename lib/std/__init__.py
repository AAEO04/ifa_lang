# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           IFÁ-LANG STANDARD LIBRARY - ÌWÉ ỌGBọ́N                              ║
║                    The 16 Principal Odù Domains                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  This is the CORE library - immutable, foundational.                         ║
║                                                                              ║
║  LAYERS:                                                                     ║
║    lib/std/     -> 16 Principal Odù (THIS - core only)                      ║
║    lib/ext/     -> 256 Compound extensions (Parent_Child)                   ║
║    project/odu/ -> User-created compounds (per-project)                     ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from .base import OduModule
from .ogbe import OgbeDomain
from .oyeku import OyekuDomain
from .iwori import IworiDomain
from .odi import OdiDomain
from .irosu import IrosuDomain
from .owonrin import OwonrinDomain
from .obara import ObaraDomain
from .okanran import OkanranDomain
from .ogunda import OgundaDomain
from .osa import OsaDomain
from .ika import IkaDomain
from .oturupon import OturuponDomain
from .otura import OturaDomain
from .irete import IreteDomain
from .ose import OseDomain
from .ofun import OfunDomain


class StandardLibrary:
    """
    The 16 Principal Odù Standard Library.
    
    For compound modules (Parent_Child), use lib.ext.registry.
    """
    
    def __init__(self):
        # === 16 PRINCIPAL DOMAINS ONLY ===
        self.domains = {
            # Logic/CPU Group
            "ogbe": OgbeDomain(),       # 0x0 - The Opener
            "oyeku": OyekuDomain(),     # 0x1 - The Closer
            "iwori": IworiDomain(),     # 0x2 - The Reflector
            "osa": OsaDomain(),         # 0x9 - The Wind
            
            # Math Group
            "obara": ObaraDomain(),     # 0x6 - The Expander
            "oturupon": OturuponDomain(), # 0xB - The Bearer
            
            # Memory/Data Group
            "ogunda": OgundaDomain(),   # 0x8 - The Cutter
            "irete": IreteDomain(),     # 0xD - The Crusher
            "ika": IkaDomain(),         # 0xA - The Constrictor
            "odi": OdiDomain(),         # 0x3 - The Vessel
            
            # Chaos/IO Group
            "owonrin": OwonrinDomain(), # 0x5 - The Reverser
            "irosu": IrosuDomain(),     # 0x4 - The Voice
            "otura": OturaDomain(),     # 0xC - The Messenger
            "ose": OseDomain(),         # 0xE - The Beautifier
            
            # Meta Group
            "okanran": OkanranDomain(), # 0x7 - The Troublemaker
            "ofun": OfunDomain(),       # 0xF - The Creator
        }
        
        # Binary code lookup
        self.by_binary = {d.binary: d for d in self.domains.values()}
    
    def get_domain(self, name: str):
        """Get a principal domain by name (case-insensitive)."""
        return self.domains.get(name.lower())
    
    def get_ese(self, domain_name: str, ese_name: str):
        """
        Retrieve a specific Ese (method) from a Domain.
        Example: get_ese("otura", "ran") -> OturaDomain.ran
        """
        domain = self.get_domain(domain_name)
        if domain:
            return getattr(domain, ese_name, None)
        return None
    
    def call(self, domain_name: str, ese_name: str, *args, **kwargs):
        """
        Call an Ese directly.
        Example: call("irosu", "fo", "Hello World!")
        """
        method = self.get_ese(domain_name, ese_name)
        if method:
            return method(*args, **kwargs)
        raise AttributeError(f"Unknown Ese: {domain_name}.{ese_name}")
    
    def help(self, domain_name: str = None):
        """Print help for a domain or all domains."""
        if domain_name:
            domain = self.get_domain(domain_name)
            if domain:
                domain.help()
        else:
            print("=== IFÁ STANDARD LIBRARY (16 PRINCIPAL ODÙ) ===")
            print("\n📦 Core Domains:")
            for name, domain in self.domains.items():
                print(f"  {name.ljust(12)} (0x{self.domains.keys().__iter__()}) - {domain.description}")
            print("\n💡 For compound modules, use: from lib.ext import registry")


# Global instance for easy access
stdlib = StandardLibrary()
