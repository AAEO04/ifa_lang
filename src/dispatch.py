# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    SEMANTIC DISPATCH - ODÙ TYPE SIGNATURES                   ║
║                    "The Path of Resolution"                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Resolves namespace collisions using Odù type signatures.                    ║
║  Example: ge("hello") → Ìká.ge (string), ge([1,2,3]) → Ògúndá.ge (array)    ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from typing import Any, Dict, List, Optional, Tuple, Callable
from dataclasses import dataclass, field
from enum import Enum


class OduType(Enum):
    VOID = "void"
    INTEGER = "int"
    FLOAT = "float"
    STRING = "str"
    BOOLEAN = "bool"
    ARRAY = "array"
    OBJECT = "obj"
    FUNCTION = "fn"
    FILE = "file"
    NETWORK = "network"
    ANY = "any"


@dataclass
class TypeSignature:
    """Type signature for an Odù method."""
    domain: str
    method: str
    param_types: List[OduType]
    return_type: OduType
    priority: int = 0
    
    def matches(self, args: List[Any]) -> bool:
        if len(args) != len(self.param_types):
            return False
        for arg, expected in zip(args, self.param_types):
            if expected == OduType.ANY:
                continue
            if not self._type_matches(arg, expected):
                return False
        return True
    
    def _type_matches(self, value: Any, expected: OduType) -> bool:
        type_map = {
            OduType.INTEGER: int, OduType.FLOAT: float, OduType.STRING: str,
            OduType.BOOLEAN: bool, OduType.ARRAY: (list, tuple), OduType.OBJECT: dict,
        }
        if expected == OduType.ANY: return True
        py_type = type_map.get(expected)
        return isinstance(value, py_type) if py_type else True
    
    def __repr__(self):
        params = ", ".join(t.value for t in self.param_types)
        return f"{self.domain}.{self.method}({params}) → {self.return_type.value}"


class SemanticDispatcher:
    """Resolves method calls to the correct Odù domain based on argument types."""
    
    def __init__(self):
        self._signatures: Dict[str, List[TypeSignature]] = {}
        self._register_stdlib()
    
    def register(self, method: str, domain: str, param_types: List[OduType], 
                 return_type: OduType, priority: int = 0):
        sig = TypeSignature(domain, method, param_types, return_type, priority)
        if method not in self._signatures:
            self._signatures[method] = []
        self._signatures[method].append(sig)
        self._signatures[method].sort(key=lambda s: -s.priority)
    
    def resolve(self, method: str, args: List[Any]) -> Optional[str]:
        if method not in self._signatures:
            return None
        for sig in self._signatures[method]:
            if sig.matches(args):
                return sig.domain
        return None
    
    def dispatch(self, method: str, args: List[Any]) -> Tuple[str, TypeSignature]:
        domain = self.resolve(method, args)
        if not domain:
            raise ValueError(f"Cannot resolve '{method}' with arguments {args}")
        sig = next(s for s in self._signatures[method] if s.domain == domain)
        return domain, sig
    
    def _register_stdlib(self):
        # String operations (Ìká)
        self.register("ge", "IKA", [OduType.STRING], OduType.STRING, 1)
        self.register("wa", "IKA", [OduType.STRING, OduType.STRING], OduType.INTEGER)
        # Array operations (Ògúndá)
        self.register("ge", "OGUNDA", [OduType.ARRAY], OduType.ARRAY, 2)
        self.register("ge", "OGUNDA", [OduType.INTEGER], OduType.ARRAY, 1)
        self.register("wa", "OGUNDA", [OduType.ARRAY, OduType.ANY], OduType.INTEGER)
        # I/O (Ìrosù)
        self.register("fo", "IROSU", [OduType.ANY], OduType.VOID)
        # File (Òdí)
        self.register("si", "ODI", [OduType.STRING, OduType.STRING], OduType.FILE)


STANDARD_VERBS = {
    "ge": "create/generate", "da": "make/construct",
    "ka": "read/retrieve", "gba": "get/receive", "wo": "view/look",
    "fi": "put/add/push", "ko": "write/store", "yi": "change/modify",
    "pa": "close/delete/kill", "ta": "remove/sell", "nu": "erase/clear",
    "wa": "find/search/seek", "ri": "see/locate",
    "so": "connect/concatenate", "pin": "divide/split", "po": "merge/join",
    "si": "open/start/flow-to", "sa": "run/execute/flow-from",
    "je": "is/equals", "ju": "compare/greater",
}

def get_verb_meaning(verb: str) -> str:
    return STANDARD_VERBS.get(verb.lower(), "unknown action")


semantic_dispatcher = SemanticDispatcher()

__all__ = ['OduType', 'TypeSignature', 'SemanticDispatcher', 'semantic_dispatcher',
           'STANDARD_VERBS', 'get_verb_meaning']
