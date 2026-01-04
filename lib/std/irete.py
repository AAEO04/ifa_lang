# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                      ÌRẸTẸ̀ - THE PRESSER (CRYPTO/HASH)                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Domain: Compression, Hashing, Cryptography                                  ║
║  English Aliases: Crypto, Hash, Zip                                          ║
║  The Spirit of Pressure - Trampling, compressing, transforming               ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import hashlib
import base64
import zlib
from typing import Any, Union

from .base import OduModule


class IreteDomain(OduModule):
    """
    Ìrẹtẹ̀ - The Presser / The Earth
    Handles compression, hashing, and cryptographic operations.
    """
    
    def __init__(self):
        super().__init__("Ìrẹtẹ̀", "1101", "The Presser - Crypto & Compression")
        
        # Hashing
        self._register("di", self.di, "Hash/bind value (md5, sha1, sha256, sha512)")
        
        # Compression
        self._register("fun", self.fun, "Compress data (zlib)")
        self._register("tu", self.tu, "Decompress data (zlib)")
        
        # Base64 Encoding
        self._register("si_base64", self.si_base64, "Encode to base64")
        self._register("lati_base64", self.lati_base64, "Decode from base64")
    
    # =========================================================================
    # HASHING
    # =========================================================================
    
    def di(self, value: Any, algorithm: str = "sha256") -> str:
        """
        dì (Hash/Bind) - Compute hash of value (SECURITY HARDENED)
        
        SECURE algorithms only: sha256, sha384, sha512, sha3_256, sha3_512
        MD5 and SHA1 are REMOVED due to known vulnerabilities.
        """
        data = str(value).encode('utf-8')
        
        # Only secure algorithms
        SECURE_ALGORITHMS = {
            'sha256': hashlib.sha256,
            'sha384': hashlib.sha384,
            'sha512': hashlib.sha512,
            'sha3_256': hashlib.sha3_256,
            'sha3_512': hashlib.sha3_512,
        }
        
        algorithm = algorithm.lower().replace('-', '_')
        
        if algorithm in ('md5', 'sha1'):
            print(f"[Security] Insecure algorithm blocked: {algorithm}")
            print("[Security] Using sha256 instead")
            algorithm = 'sha256'
        
        if algorithm not in SECURE_ALGORITHMS:
            algorithm = 'sha256'
        
        hash_func = SECURE_ALGORITHMS[algorithm]
        return hash_func(data).hexdigest()
    
    # =========================================================================
    # COMPRESSION
    # =========================================================================
    
    def fun(self, data: Union[str, bytes]) -> bytes:
        """
        fún (Compress/Give) - Compress data
        
        Yoruba: Ìrẹtẹ̀.fún()
        English: Zip.compress(), Crypto.compress()
        """
        if isinstance(data, str):
            data = data.encode('utf-8')
        return zlib.compress(data)
    
    def tu(self, data: bytes) -> str:
        """
        tú (Decompress/Release) - Decompress data
        
        Yoruba: Ìrẹtẹ̀.tú()
        English: Zip.decompress()
        """
        return zlib.decompress(data).decode('utf-8')
    
    # =========================================================================
    # BASE64 ENCODING
    # =========================================================================
    
    def si_base64(self, data: Union[str, bytes]) -> str:
        """
        sí_base64 (To Base64) - Encode to base64
        
        Yoruba: Ìrẹtẹ̀.sí_base64()
        English: Crypto.encode64()
        """
        if isinstance(data, str):
            data = data.encode('utf-8')
        return base64.b64encode(data).decode('ascii')
    
    def lati_base64(self, data: str) -> bytes:
        """
        láti_base64 (From Base64) - Decode from base64
        
        Yoruba: Ìrẹtẹ̀.láti_base64()
        English: Crypto.decode64()
        """
        return base64.b64decode(data)


# Module-level singleton and functions for backwards compatibility
_irete = IreteDomain()

def di(value: Any, algorithm: str = "sha256") -> str:
    return _irete.di(value, algorithm)

def fun(data: Union[str, bytes]) -> bytes:
    return _irete.fun(data)

def tu(data: bytes) -> str:
    return _irete.tu(data)

def si_base64(data: Union[str, bytes]) -> str:
    return _irete.si_base64(data)

def lati_base64(data: str) -> bytes:
    return _irete.lati_base64(data)

# English aliases
hash = di
compress = fun
decompress = tu
zip = fun
unzip = tu
