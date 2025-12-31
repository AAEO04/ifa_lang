# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ÒTÚRÁ_ÌKÁ - Network Security (Firewall)                   ║
║                    SSL / TLS / Encryption / Access Control                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Compound: Òtúrá (Network) + Ìká (Control/Restriction)                       ║
║  Meaning:  "Controlled Communication"                                        ║
║  Opcode:   0xCA (Parent: 0xC, Child: 0xA)                                    ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import ssl
import hashlib
from typing import List, Set


class OturaIka:
    """Network Security - Firewall, SSL, Encryption"""
    
    # Blocked IPs (in-memory firewall)
    _blocked_ips: Set[str] = set()
    _allowed_ips: Set[str] = set()
    
    @classmethod
    def de_ona(cls, ip: str) -> bool:
        """
        dé ọ̀nà = Block the road (Block an IP)
        
        Args:
            ip: IP address to block
            
        Returns:
            True if successfully blocked
        """
        cls._blocked_ips.add(ip)
        print(f"🚫 [Òtúrá_Ìká] Blocked: {ip}")
        return True
    
    @classmethod
    def si_ona(cls, ip: str) -> bool:
        """
        sí ọ̀nà = Open the road (Allow an IP)
        
        Args:
            ip: IP address to allow
            
        Returns:
            True if successfully allowed
        """
        cls._blocked_ips.discard(ip)
        cls._allowed_ips.add(ip)
        print(f"✅ [Òtúrá_Ìká] Allowed: {ip}")
        return True
    
    @classmethod
    def yẹwo(cls, ip: str) -> bool:
        """
        yẹ̀wò = Inspect (Check if IP is allowed)
        
        Args:
            ip: IP address to check
            
        Returns:
            True if allowed, False if blocked
        """
        if ip in cls._blocked_ips:
            return False
        return True
    
    @staticmethod
    def pamọ(data: str, key: str) -> str:
        """
        pa mọ́ = Hide it (Encrypt/Hash data)
        
        Args:
            data: Data to encrypt
            key: Encryption key
            
        Returns:
            SHA256 hash of data+key
        """
        combined = f"{data}{key}"
        return hashlib.sha256(combined.encode()).hexdigest()
    
    @staticmethod
    def ṣe_ssl() -> dict:
        """
        ṣe SSL = Create SSL context information
        
        Returns:
            Dictionary with SSL version info
        """
        return {
            "version": ssl.OPENSSL_VERSION,
            "default_verify_mode": "CERT_REQUIRED"
        }
    
    @classmethod
    def akojọ_dena(cls) -> List[str]:
        """
        àkójọ déná = List of blocked (Get blocked IPs)
        
        Returns:
            List of blocked IP addresses
        """
        return list(cls._blocked_ips)


# Module-level functions for direct access
def de_ona(ip: str) -> bool:
    return OturaIka.de_ona(ip)

def si_ona(ip: str) -> bool:
    return OturaIka.si_ona(ip)

def yewo(ip: str) -> bool:
    return OturaIka.yẹwo(ip)

def pamo(data: str, key: str) -> str:
    return OturaIka.pamọ(data, key)

def se_ssl() -> dict:
    return OturaIka.ṣe_ssl()

def akojo_dena() -> List[str]:
    return OturaIka.akojọ_dena()
