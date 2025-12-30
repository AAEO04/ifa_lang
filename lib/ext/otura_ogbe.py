# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ÒTÚRÁ_OGBÈ - Network Initialization                       ║
║                    DNS / DHCP / Network Discovery                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Compound: Òtúrá (Network) + Ogbè (Source/Origin)                            ║
║  Meaning:  "Communication with the Source"                                   ║
║  Opcode:   0xC0 (Parent: 0xC, Child: 0x0)                                    ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import socket


class OturaOgbe:
    """Network Initialization - DNS, DHCP, Discovery"""
    
    @staticmethod
    def wa_oruko(domain: str) -> str:
        """
        wá orúkọ = Seek the name (DNS Lookup)
        
        Args:
            domain: Domain name to resolve
            
        Returns:
            IP address string
        """
        try:
            ip = socket.gethostbyname(domain)
            return ip
        except socket.gaierror:
            return "0.0.0.0"
    
    @staticmethod
    def wa_ile(ip: str) -> str:
        """
        wá ilé = Seek the home (Reverse DNS)
        
        Args:
            ip: IP address to resolve
            
        Returns:
            Hostname string
        """
        try:
            hostname = socket.gethostbyaddr(ip)
            return hostname[0]
        except socket.herror:
            return "unknown"
    
    @staticmethod
    def bi_asopọ() -> dict:
        """
        bí àsopọ̀ = Birth connection (Get Network Info)
        
        Returns:
            Dictionary with hostname and IP
        """
        hostname = socket.gethostname()
        try:
            ip = socket.gethostbyname(hostname)
        except:
            ip = "127.0.0.1"
        
        return {
            "hostname": hostname,
            "ip": ip
        }


# Module-level functions for direct access
def wa_oruko(domain: str) -> str:
    return OturaOgbe.wa_oruko(domain)

def wa_ile(ip: str) -> str:
    return OturaOgbe.wa_ile(ip)

def bi_asopo() -> dict:
    return OturaOgbe.bi_asopo()
