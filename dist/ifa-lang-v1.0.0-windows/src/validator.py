# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           IFA SMART COMPILER - THE ÌWÀ ENGINE                                ║
║              "The Babalawo at the Gate"                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  A semantic analysis layer that enforces Ìwà-Pẹ̀lẹ́ (Good Character).         ║
║  Every opening action MUST have a paired closing action.                     ║
║                                                                              ║
║  The Rule of Ibeji (Twins):                                                  ║
║    - Ogbè (Birth) requires Ọ̀yẹ̀kú (Death)                                    ║
║    - Òdí.open requires Òdí.close                                             ║
║    - Òtúrá.bind requires Òtúrá.close                                         ║
║    - Ògúndá.alloc requires Ìrẹtẹ̀.free                                        ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import re
import unicodedata
from typing import List, Dict, Tuple, Optional

# =============================================================================
# THE TWIN REGISTRY (Lifecycle Rules)
# =============================================================================
LIFECYCLE_RULES = {
    # File I/O (Òdí)
    "odi.si":     "odi.pa",       # Open file -> Close file
    "odi.ko":     "odi.pa",       # Write file -> Close file
    
    # Network (Òtúrá)
    "otura.de":   "otura.pa",     # Bind -> Close
    "otura.so":   "otura.pa",     # Connect -> Close
    
    # Memory (Ògúndá/Ìrẹtẹ̀)
    "ogunda.ge":  "irete.tu",     # Alloc -> Free
    "ogunda.da":  "irete.tu",     # Create -> Free
    
    # Objects (Òfún)
    "ofun.da":    "ofun.pa",      # Create object -> Delete object
    
    # System (Ogbè/Ọ̀yẹ̀kú)
    "ogbe.bi":    "oyeku.duro",   # Init -> Halt
    "ogbe.bere":  "oyeku.duro",   # Start -> Stop
    
    # Graphics (Ọ̀ṣẹ́)
    "ose.nu":     None,           # Clear doesn't need closing
    
    # Loops (special tracking)
    "iwori.yipo": "iwori.pada",   # Loop start -> Return
}

# Resources that auto-close at program end (don't need explicit close)
AUTO_CLOSE = {"ogbe.bi", "ogbe.bere"}


# =============================================================================
# ÈÈWỌ̀ ENFORCER (Taboo - Architectural Constraints)
# =============================================================================

class TabooEnforcer:
    """
    The Èèwọ̀ Enforcer - Validates architectural constraints.
    
    Taboos are forbidden patterns that guarantee architectural purity:
    - èèwọ̀: Ọ̀ṣẹ́(UI) -> Òdí(DB);   # UI can't call DB directly
    - èèwọ̀: Òtúrá.*;               # No network calls allowed in file
    """
    
    def __init__(self):
        self.taboos: List[Dict] = []
        self.violations: List[Dict] = []
        self.current_context: str = ""
    
    def add_taboo(self, source_domain: str, source_context: str = "",
                  target_domain: str = "", target_context: str = "",
                  is_wildcard: bool = False):
        """Register a taboo rule."""
        taboo = {
            "source_domain": source_domain.lower(),
            "source_context": source_context,
            "target_domain": target_domain.lower() if target_domain else "",
            "target_context": target_context,
            "is_wildcard": is_wildcard,
        }
        self.taboos.append(taboo)
        
        if is_wildcard:
            print(f"  [ÈÈWỌ̀] Taboo registered: {source_domain}.* is FORBIDDEN")
        else:
            src = f"{source_domain}({source_context})" if source_context else source_domain
            tgt = f"{target_domain}({target_context})" if target_context else target_domain
            print(f"  [ÈÈWỌ̀] Taboo registered: {src} -> {tgt} is FORBIDDEN")
    
    def set_context(self, context: str):
        """Set current code context (e.g., 'UI', 'Backend')."""
        self.current_context = context
    
    def check_call(self, caller_domain: str, callee_domain: str, 
                   line: int = 0) -> bool:
        """
        Check if a call violates any taboo.
        Returns True if call is allowed, False if forbidden.
        """
        caller = caller_domain.lower()
        callee = callee_domain.lower()
        
        for taboo in self.taboos:
            # Wildcard taboo: Block all calls from this domain
            if taboo["is_wildcard"]:
                if callee == taboo["source_domain"]:
                    self.violations.append({
                        "type": "wildcard",
                        "domain": callee,
                        "line": line,
                        "taboo": taboo,
                    })
                    return False
            
            # Specific taboo: Block source -> target
            else:
                source_match = (caller == taboo["source_domain"] or 
                               taboo["source_domain"] == "")
                context_match = (self.current_context == taboo["source_context"] or
                                taboo["source_context"] == "")
                target_match = callee == taboo["target_domain"]
                
                if source_match and context_match and target_match:
                    self.violations.append({
                        "type": "dependency",
                        "caller": caller,
                        "callee": callee,
                        "context": self.current_context,
                        "line": line,
                        "taboo": taboo,
                    })
                    return False
        
        return True
    
    def report_violations(self):
        """Report all taboo violations."""
        if not self.violations:
            return
        
        print("\n╔════════════════════════════════════════════════════════════╗")
        print("║              !!! ÈÈWỌ̀ VIOLATION - TABOO BROKEN !!!         ║")
        print("╚════════════════════════════════════════════════════════════╝")
        print()
        
        for v in self.violations:
            if v["type"] == "wildcard":
                print(f"  ⛔ Line {v['line']}: Called forbidden domain '{v['domain']}'")
                print(f"     └── Taboo: {v['domain']}.* is not allowed in this file")
            else:
                print(f"  ⛔ Line {v['line']}: '{v['caller']}' called '{v['callee']}'")
                if v.get("context"):
                    print(f"     └── Context '{v['context']}' cannot access '{v['callee']}'")
                else:
                    print(f"     └── This dependency is forbidden by architectural rules")
        
        print()
        print("┌────────────────────────────────────────────────────────────┐")
        print("│  PROVERB: \"Ẹni tó bá fọwọ́ kan èèwọ̀, yóò rí àṣèdá\"             │")
        print("│  (Whoever touches a taboo will see the consequences)       │")
        print("└────────────────────────────────────────────────────────────┘")
        print()
    
    def is_clean(self) -> bool:
        """Return True if no taboos were violated."""
        return len(self.violations) == 0



class IwaEngine:
    """
    The Ìwà Engine - Semantic analyzer for resource lifecycle.
    Ensures every opening action has a corresponding closing action.
    """
    
    def __init__(self, strict_mode: bool = True):
        self.strict_mode = strict_mode
        self.debt_ledger: List[Dict] = []
        self.errors: List[str] = []
        self.warnings: List[str] = []
    
    def normalize(self, text: str) -> str:
        """Normalize Yoruba characters to ASCII."""
        text = unicodedata.normalize('NFD', text.lower())
        result = ""
        for char in text:
            if unicodedata.category(char) == 'Mn':  # Combining mark
                continue
            if char in 'ọẹṣ':
                result += {'ọ': 'o', 'ẹ': 'e', 'ṣ': 's'}.get(char, char)
            else:
                result += char
        return result
    
    def check(self, tokens: List[str], source_lines: List[str] = None) -> bool:
        """
        Check token stream for balance violations.
        Returns True if balanced, False otherwise.
        """
        print("\n╔════════════════════════════════════════════════════════╗")
        print("║         ÌWÀ ENGINE - Checking for Balance              ║")
        print("╚════════════════════════════════════════════════════════╝")
        
        self.debt_ledger = []
        self.errors = []
        self.warnings = []
        
        for i, token in enumerate(tokens):
            normalized = self.normalize(token)
            line_num = i + 1
            
            # Is this an OPENING action?
            if normalized in LIFECYCLE_RULES:
                required_close = LIFECYCLE_RULES[normalized]
                
                if required_close is not None:
                    debt = {
                        "opener": normalized,
                        "required": required_close,
                        "line": line_num
                    }
                    self.debt_ledger.append(debt)
                    print(f"  [+] Line {line_num}: Opened '{normalized}' → Owes '{required_close}'")
            
            # Is this a CLOSING action?
            else:
                # Check if this closes any open resource
                for j, debt in enumerate(self.debt_ledger):
                    if debt["required"] == normalized:
                        print(f"  [-] Line {line_num}: Closed '{normalized}' → Debt paid!")
                        self.debt_ledger.pop(j)
                        break
        
        # Remove auto-close items
        self.debt_ledger = [
            d for d in self.debt_ledger 
            if d["opener"] not in AUTO_CLOSE
        ]
        
        # Final Judgment
        if len(self.debt_ledger) > 0:
            self._report_balance_error()
            return False
        
        print("\n┌────────────────────────────────────────────────────────┐")
        print("│  ✓ SUCCESS: The code possesses Ìwà-Pẹ̀lẹ́ (Good Character) │")
        print("└────────────────────────────────────────────────────────┘")
        return True
    
    def _report_balance_error(self):
        """Report balance errors with helpful messages."""
        print("\n╔════════════════════════════════════════════════════════╗")
        print("║         !!! BALANCE ERROR - ÌWÀ VIOLATION !!!          ║")
        print("╚════════════════════════════════════════════════════════╝")
        print()
        print("The program ended with UNPAID DEBTS:")
        print("─" * 50)
        
        for debt in self.debt_ledger:
            print(f"  • Line {debt['line']}: '{debt['opener']}' opened")
            print(f"    └── Missing: '{debt['required']}' to close it")
        
        print()
        print("┌────────────────────────────────────────────────────────┐")
        print("│  PROVERB: \"One cannot take without giving back.\"       │")
        print("│  Every opened resource must be closed.                 │")
        print("└────────────────────────────────────────────────────────┘")
        print()
        print(">>> COMPILATION HALTED: Fix your Character first.")


# =============================================================================
# THE SMART COMPILER
# =============================================================================
class SmartIfaCompiler:
    """
    Smart Ifá Compiler with integrated Ìwà Engine.
    Pipeline: Parse → Validate (Ìwà) → Transpile
    """
    
    def __init__(self, strict_mode: bool = True):
        self.iwa_engine = IwaEngine(strict_mode)
        self.tokens: List[str] = []
        self.source_lines: List[str] = []
    
    def parse(self, source_code: str) -> List[str]:
        """Parse source code into token stream."""
        tokens = []
        self.source_lines = []
        
        lines = [l.strip() for l in source_code.split(';') if l.strip()]
        
        for line in lines:
            # Skip comments and imports
            if line.startswith("//") or line.startswith("#"):
                continue
            if line.startswith("iba") or line.startswith("ìbà"):
                continue
            if line in ('ase', 'àṣẹ'):
                tokens.append("oyeku.duro")
                continue
            
            # Extract Domain.Method pattern
            match = re.search(r'([\w\u00C0-\u017F]+)\.([\w\u00C0-\u017F]+)', line)
            if match:
                domain = match.group(1).lower()
                method = match.group(2).lower()
                token = f"{domain}.{method}"
                tokens.append(token)
                self.source_lines.append(line)
        
        self.tokens = tokens
        return tokens
    
    def validate(self) -> bool:
        """Run Ìwà Engine validation."""
        return self.iwa_engine.check(self.tokens, self.source_lines)
    
    def compile(self, source_code: str) -> Optional[str]:
        """
        Full compilation pipeline:
        1. Parse
        2. Validate (Ìwà Engine)
        3. Transpile (if valid)
        """
        print("\n" + "=" * 60)
        print("SMART IFÁ COMPILER - Compilation Started")
        print("=" * 60)
        
        # Step 1: Parse
        print("\n[1/3] Parsing source code...")
        tokens = self.parse(source_code)
        print(f"      Found {len(tokens)} tokens")
        
        # Step 2: Validate
        print("\n[2/3] Running Ìwà Engine (Balance Check)...")
        is_balanced = self.validate()
        
        if not is_balanced:
            return None
        
        # Step 3: Transpile
        print("\n[3/3] Proceeding to Rust Transpilation...")
        rust_code = self._transpile()
        
        print("\n" + "=" * 60)
        print("COMPILATION SUCCESSFUL")
        print("=" * 60)
        
        return rust_code
    
    def _transpile(self) -> str:
        """Generate Rust code from validated tokens."""
        # Import the Rust transpiler
        try:
            from ifa_rust import IfaRustTranspiler
            transpiler = IfaRustTranspiler()
            return transpiler.transpile(";".join(self.source_lines))
        except ImportError:
            return "// Rust code would be generated here"


# =============================================================================
# DEMO AND TESTS
# =============================================================================
if __name__ == "__main__":
    compiler = SmartIfaCompiler()
    
    print("""
╔══════════════════════════════════════════════════════════════╗
║           SMART COMPILER DEMO - THE ÌWÀ ENGINE               ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    # TEST A: Unbalanced Code (Bad)
    bad_code = """
ìbà Odi;
Òdí.ṣí("secret.txt");
Ìrosù.sọ("Writing...");
# Forgot to close the file!
ase;
"""
    
    print("\n" + "─" * 60)
    print("TEST A: THE GREEDY PROGRAM (Unbalanced)")
    print("─" * 60)
    print(bad_code)
    result = compiler.compile(bad_code)
    print(f"\nResult: {'REJECTED' if result is None else 'ACCEPTED'}")
    
    # TEST B: Balanced Code (Good)
    good_code = """
ìbà Odi;
Òdí.ṣí("secret.txt");
Ìrosù.sọ("Writing...");
Òdí.pa();
ase;
"""
    
    print("\n" + "─" * 60)
    print("TEST B: THE VIRTUOUS PROGRAM (Balanced)")
    print("─" * 60)
    print(good_code)
    result = compiler.compile(good_code)
    print(f"\nResult: {'REJECTED' if result is None else 'ACCEPTED'}")
    
    # TEST C: Network Code
    network_code = """
ìbà Otura;
Òtúrá.dè(8080);
Òtúrá.rán("Hello");
Òtúrá.pa();
ase;
"""
    
    print("\n" + "─" * 60)
    print("TEST C: NETWORK PROGRAM (Balanced)")
    print("─" * 60)
    print(network_code)
    result = compiler.compile(network_code)
    print(f"\nResult: {'REJECTED' if result is None else 'ACCEPTED'}")
    
    # TEST D: Memory Leak
    leak_code = """
ìbà Ogunda;
Ògúndá.gé("buffer", 256);
Ìrosù.sọ("Using memory...");
# Forgot to free!
ase;
"""
    
    print("\n" + "─" * 60)
    print("TEST D: MEMORY LEAK (Unbalanced)")
    print("─" * 60)
    print(leak_code)
    result = compiler.compile(leak_code)
    print(f"\nResult: {'REJECTED' if result is None else 'ACCEPTED'}")
