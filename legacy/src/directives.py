# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    DIRECTIVE PARSER - #opon, #target, #ewọ                   ║
║                    "The Sacred Instructions"                                 ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Parses preprocessing directives from .ifa files.                            ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import re
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from enum import Enum


class OponSize(Enum):
    KEKERE = "kekere"  # 4KB
    GIDI = "gidi"      # 16KB  
    NLA = "nla"        # 64KB
    MEGA = "mega"      # 1MB


class TargetPlatform(Enum):
    PYTHON = "python"
    BYTECODE = "bytecode"
    RUST = "rust"
    WASM = "wasm"


@dataclass
class EwoDirective:
    source_domain: str
    target_domain: str
    line_number: int


@dataclass
class ImportDirective:
    module: str
    alias: Optional[str]
    line_number: int


@dataclass
class ParsedDirectives:
    opon_size: OponSize = OponSize.GIDI
    target: TargetPlatform = TargetPlatform.PYTHON
    ewos: List[EwoDirective] = field(default_factory=list)
    imports: List[ImportDirective] = field(default_factory=list)
    raw_lines: Dict[int, str] = field(default_factory=dict)


class DirectiveParser:
    """Parses preprocessing directives from .ifa source files."""
    
    OPON_PATTERN = re.compile(r'#opon\s+(kekere|gidi|nla|mega)', re.IGNORECASE)
    TARGET_PATTERN = re.compile(r'#target\s+(python|bytecode|rust|wasm)', re.IGNORECASE)
    EWO_PATTERN = re.compile(r'#ew[oọ]\s+(\w+)\s*[-→>]+\s*(\w+)', re.IGNORECASE)
    IMPORT_PATTERN = re.compile(r'#import\s+"([^"]+)"(?:\s+as\s+(\w+))?', re.IGNORECASE)
    
    def parse(self, source: str) -> tuple:
        """Parse directives from source code. Returns (ParsedDirectives, clean_source)."""
        directives = ParsedDirectives()
        clean_lines = []
        
        for line_num, line in enumerate(source.split('\n'), 1):
            stripped = line.strip()
            if stripped.startswith('#'):
                parsed = self._parse_line(stripped, line_num, directives)
                if parsed:
                    directives.raw_lines[line_num] = stripped
                    clean_lines.append('')
                    continue
            clean_lines.append(line)
        
        return directives, '\n'.join(clean_lines)
    
    def _parse_line(self, line: str, line_num: int, directives: ParsedDirectives) -> bool:
        match = self.OPON_PATTERN.match(line)
        if match:
            directives.opon_size = OponSize(match.group(1).lower())
            print(f"  [Directive] Line {line_num}: #opon {match.group(1)}")
            return True
        
        match = self.TARGET_PATTERN.match(line)
        if match:
            directives.target = TargetPlatform(match.group(1).lower())
            print(f"  [Directive] Line {line_num}: #target {match.group(1)}")
            return True
        
        match = self.EWO_PATTERN.match(line)
        if match:
            ewo = EwoDirective(match.group(1).upper(), match.group(2).upper(), line_num)
            directives.ewos.append(ewo)
            print(f"  [Directive] Line {line_num}: #ewọ {ewo.source_domain} → {ewo.target_domain}")
            return True
        
        match = self.IMPORT_PATTERN.match(line)
        if match:
            imp = ImportDirective(match.group(1), match.group(2), line_num)
            directives.imports.append(imp)
            alias_str = f" as {imp.alias}" if imp.alias else ""
            print(f"  [Directive] Line {line_num}: #import \"{imp.module}\"{alias_str}")
            return True
        
        return False
    
    def apply_to_context(self, directives: ParsedDirectives, context: Dict[str, Any]) -> Dict:
        opon_bytes = {OponSize.KEKERE: 4*1024, OponSize.GIDI: 16*1024, 
                      OponSize.NLA: 64*1024, OponSize.MEGA: 1024*1024}
        context['opon_size'] = opon_bytes[directives.opon_size]
        context['target'] = directives.target.value
        context['ewos'] = [(e.source_domain, e.target_domain) for e in directives.ewos]
        context['imports'] = {(imp.alias or imp.module): imp.module for imp in directives.imports}
        return context


def parse_directives(source: str) -> tuple:
    return DirectiveParser().parse(source)


__all__ = ['OponSize', 'TargetPlatform', 'EwoDirective', 'ImportDirective',
           'ParsedDirectives', 'DirectiveParser', 'parse_directives']
