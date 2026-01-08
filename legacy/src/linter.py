# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                        IFÁ-LANG LINTER (BABALAWO)                            ║
║                   "The Wise One Who Finds Problems"                          ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Static analysis tool that checks for:                                       ║
║  - Undefined variables                                                       ║
║  - Unused imports                                                            ║
║  - Type hint violations (Orí system)                                         ║
║  - Taboo constraint violations (èèwọ̀)                                        ║
║  - Code style suggestions                                                    ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import re
from dataclasses import dataclass, field
from typing import List, Set, Dict, Optional, Any
from enum import Enum

# Try to import Lark parser
try:
    from src.lark_parser import IfaLarkParser, Program, VarDecl, ImportStmt, OduCall, Instruction
    from src.lark_parser import IfStmt, WhileStmt, ForStmt, TryStmt, EseDef, OduDef
    LARK_AVAILABLE = True
except ImportError:
    LARK_AVAILABLE = False


# =============================================================================
# LINT MESSAGE TYPES
# =============================================================================

class Severity(Enum):
    ERROR = "error"      # Must fix
    WARNING = "warning"  # Should fix
    INFO = "info"        # Suggestion
    STYLE = "style"      # Style nitpick


@dataclass
class LintMessage:
    """A single lint warning/error."""
    severity: Severity
    code: str           # e.g., "E001", "W001"
    message: str
    line: int = 0
    column: int = 0
    suggestion: str = ""
    
    def __str__(self):
        icon = {
            Severity.ERROR: "❌",
            Severity.WARNING: "⚠️",
            Severity.INFO: "💡",
            Severity.STYLE: "🎨",
        }.get(self.severity, "•")
        
        loc = f":{self.line}" if self.line else ""
        sug = f"\n      💡 {self.suggestion}" if self.suggestion else ""
        return f"   {icon} [{self.code}] {self.message}{loc}{sug}"


# =============================================================================
# LINTER RULES
# =============================================================================

@dataclass
class LintContext:
    """Context for linting - tracks state as we walk the AST."""
    defined_vars: Set[str] = field(default_factory=set)
    used_vars: Set[str] = field(default_factory=set)
    imported_modules: Set[str] = field(default_factory=set)
    used_modules: Set[str] = field(default_factory=set)
    defined_functions: Set[str] = field(default_factory=set)
    called_functions: Set[str] = field(default_factory=set)
    taboos: List[tuple] = field(default_factory=list)  # (source, target) pairs
    var_types: Dict[str, str] = field(default_factory=dict)  # var -> type hint
    messages: List[LintMessage] = field(default_factory=list)
    
    def add_error(self, code: str, msg: str, line: int = 0, suggestion: str = ""):
        self.messages.append(LintMessage(Severity.ERROR, code, msg, line, suggestion=suggestion))
    
    def add_warning(self, code: str, msg: str, line: int = 0, suggestion: str = ""):
        self.messages.append(LintMessage(Severity.WARNING, code, msg, line, suggestion=suggestion))
    
    def add_info(self, code: str, msg: str, line: int = 0, suggestion: str = ""):
        self.messages.append(LintMessage(Severity.INFO, code, msg, line, suggestion=suggestion))
    
    def add_style(self, code: str, msg: str, line: int = 0, suggestion: str = ""):
        self.messages.append(LintMessage(Severity.STYLE, code, msg, line, suggestion=suggestion))


# =============================================================================
# LINTER CLASS
# =============================================================================

class IfaLinter:
    """
    The Babalawo (Linter) - Wise one who finds problems.
    
    Error Codes:
        E001-E099: Syntax errors
        E100-E199: Variable errors
        E200-E299: Type errors
        E300-E399: Import errors
        W001-W099: General warnings
        W100-W199: Unused code warnings
        W200-W299: Style warnings
        S001-S099: Style suggestions
    """
    
    # Known Odù domains (for validation)
    VALID_DOMAINS = {
        # Yoruba
        "ogbe", "oyeku", "iwori", "odi", "irosu", "owonrin", "obara", "okanran",
        "ogunda", "osa", "ika", "oturupon", "otura", "irete", "ose", "ofun",
        # English aliases
        "init", "start", "exit", "end", "time", "clock", "file", "memory",
        "log", "print", "rand", "random", "add", "math", "error", "except",
        "array", "list", "proc", "system", "text", "string", "sub", "subtract",
        "net", "network", "logic", "bool", "draw", "graphics", "meta", "reflect",
        "async", "thread", "crypto", "hash", "root",
    }
    
    # Type hints for Orí system
    VALID_TYPES = {"Int", "Float", "Str", "Bool", "List", "Map", "Any"}
    
    def __init__(self, source: str, filename: str = "<stdin>"):
        self.source = source
        self.filename = filename
        self.lines = source.split('\n')
        self.ctx = LintContext()
        self.ast = None
    
    def lint(self) -> List[LintMessage]:
        """Run all lint checks and return messages."""
        
        # Security: Limit input size to prevent DoS
        MAX_SOURCE_SIZE = 1024 * 1024  # 1MB max
        if len(self.source) > MAX_SOURCE_SIZE:
            self.ctx.add_error("E098", f"File too large ({len(self.source)} bytes > {MAX_SOURCE_SIZE})")
            return self.ctx.messages
        
        # Security: Limit line count
        MAX_LINES = 50000
        if len(self.lines) > MAX_LINES:
            self.ctx.add_error("E099", f"Too many lines ({len(self.lines)} > {MAX_LINES})")
            return self.ctx.messages
        
        # Phase 1: Line-based checks (before parsing)
        self._check_line_issues()
        
        # Phase 2: Parse and AST-based checks
        if LARK_AVAILABLE:
            try:
                parser = IfaLarkParser()
                self.ast = parser.parse(self.source)
                self._walk_ast(self.ast)
                self._check_unused()
                self._check_taboos()
            except Exception as e:
                self.ctx.add_error("E001", f"Parse error: {e}")
        else:
            # Fallback: regex-based checks
            self._check_regex_patterns()
        
        return sorted(self.ctx.messages, key=lambda m: (m.line, m.severity.value))
    
    # =========================================================================
    # LINE-BASED CHECKS
    # =========================================================================
    
    def _check_line_issues(self):
        """Check for line-level issues."""
        for i, line in enumerate(self.lines, 1):
            # Check for trailing whitespace
            if line != line.rstrip():
                self.ctx.add_style("S001", "Trailing whitespace", i)
            
            # Check for very long lines
            if len(line) > 120:
                self.ctx.add_style("S002", f"Line too long ({len(line)} > 120 chars)", i)
            
            # Check for tabs (prefer spaces)
            if '\t' in line:
                self.ctx.add_style("S003", "Use spaces instead of tabs", i)
            
            # Check for TODO/FIXME comments
            if "TODO" in line.upper() or "FIXME" in line.upper():
                self.ctx.add_info("I001", "TODO/FIXME found", i)
    
    # =========================================================================
    # AST-BASED CHECKS
    # =========================================================================
    
    def _walk_ast(self, node, depth: int = 0):
        """Walk the AST and collect information."""
        if node is None:
            return
        
        node_type = type(node).__name__
        
        # Handle different node types
        if node_type == "Program":
            for stmt in getattr(node, 'statements', []):
                self._walk_ast(stmt, depth)
        
        elif node_type == "ImportStmt":
            path = ".".join(getattr(node, 'path', []))
            self.ctx.imported_modules.add(path)
        
        elif node_type == "VarDecl":
            name = getattr(node, 'name', '')
            self.ctx.defined_vars.add(name)
            
            # Check type hint
            type_hint = getattr(node, 'type_hint', None)
            if type_hint:
                self.ctx.var_types[name] = type_hint
                # Validate type name
                if type_hint not in self.VALID_TYPES:
                    self.ctx.add_error("E200", f"Unknown type '{type_hint}'",
                        suggestion=f"Valid types: {', '.join(self.VALID_TYPES)}")
            
            # Walk the value
            self._walk_ast(getattr(node, 'value', None), depth)
        
        elif node_type == "Instruction":
            self._walk_ast(getattr(node, 'call', None), depth)
        
        elif node_type == "OduCall":
            odu = getattr(node, 'odu', '').lower()
            ese = getattr(node, 'ese', '')
            
            self.ctx.used_modules.add(odu)
            
            # Validate Odù name
            if odu and odu not in self.VALID_DOMAINS:
                self.ctx.add_warning("W001", f"Unknown Odù domain '{odu}'",
                    suggestion="Check spelling or use English alias")
            
            # Walk arguments
            for arg in getattr(node, 'args', []):
                self._walk_ast(arg, depth + 1)
        
        elif node_type == "Identifier":
            name = getattr(node, 'name', '')
            self.ctx.used_vars.add(name)
            
            # Check if variable is defined
            if name not in self.ctx.defined_vars:
                self.ctx.add_error("E100", f"Undefined variable '{name}'",
                    suggestion=f"Add 'ayanmo {name} = ...' or 'let {name} = ...'")
        
        elif node_type in ("IfStmt", "WhileStmt"):
            self._walk_ast(getattr(node, 'condition', None), depth)
            for s in getattr(node, 'then_body', []) or getattr(node, 'body', []):
                self._walk_ast(s, depth + 1)
            for s in getattr(node, 'else_body', []):
                self._walk_ast(s, depth + 1)
        
        elif node_type == "ForStmt":
            var_name = getattr(node, 'var_name', '')
            self.ctx.defined_vars.add(var_name)
            self._walk_ast(getattr(node, 'iterable', None), depth)
            for s in getattr(node, 'body', []):
                self._walk_ast(s, depth + 1)
        
        elif node_type == "TryStmt":
            for s in getattr(node, 'try_body', []):
                self._walk_ast(s, depth + 1)
            error_var = getattr(node, 'error_var', 'err')
            self.ctx.defined_vars.add(error_var)
            for s in getattr(node, 'catch_body', []):
                self._walk_ast(s, depth + 1)
        
        elif node_type == "EseDef":
            func_name = getattr(node, 'name', '')
            self.ctx.defined_functions.add(func_name)
            for param in getattr(node, 'params', []):
                self.ctx.defined_vars.add(param)
            for s in getattr(node, 'body', []):
                self._walk_ast(s, depth + 1)
        
        elif node_type == "OduDef":
            for method in getattr(node, 'methods', []):
                self._walk_ast(method, depth + 1)
            for var in getattr(node, 'variables', []):
                self._walk_ast(var, depth + 1)
        
        elif node_type == "TabooStmt":
            # Record taboo constraints
            source = getattr(node, 'source', None)
            target = getattr(node, 'target', None)
            if source and target:
                self.ctx.taboos.append((source, target))
        
        elif node_type == "BinaryOp":
            self._walk_ast(getattr(node, 'left', None), depth)
            self._walk_ast(getattr(node, 'right', None), depth)
        
        elif node_type == "UnaryOp":
            self._walk_ast(getattr(node, 'operand', None), depth)
        
        elif node_type == "ListLiteral":
            for elem in getattr(node, 'elements', []):
                self._walk_ast(elem, depth)
        
        elif node_type == "MapLiteral":
            for k, v in getattr(node, 'entries', {}).items():
                self._walk_ast(v, depth)
        
        elif node_type == "IndexAccess":
            self._walk_ast(getattr(node, 'target', None), depth)
            self._walk_ast(getattr(node, 'index', None), depth)
        
        elif node_type == "AssignmentStmt":
            self._walk_ast(getattr(node, 'target', None), depth)
            self._walk_ast(getattr(node, 'value', None), depth)
    
    def _check_unused(self):
        """Check for unused imports and variables."""
        # Unused imports
        for module in self.ctx.imported_modules:
            # Extract domain name
            domain = module.split('.')[-1].lower()
            if domain not in self.ctx.used_modules:
                self.ctx.add_warning("W100", f"Unused import '{module}'",
                    suggestion="Remove if not needed")
        
        # Unused variables (only warn about user-defined vars)
        unused = self.ctx.defined_vars - self.ctx.used_vars
        # Exclude common loop variables
        unused -= {'i', 'j', 'k', 'item', 'elem', 'err', '_'}
        for var in unused:
            self.ctx.add_warning("W101", f"Unused variable '{var}'",
                suggestion="Prefix with _ if intentionally unused")
    
    def _check_taboos(self):
        """Check for taboo constraint violations."""
        # This is a placeholder - real implementation would track call graph
        for source, target in self.ctx.taboos:
            self.ctx.add_info("I002", f"Taboo constraint: {source} -> {target}")
    
    # =========================================================================
    # REGEX FALLBACK CHECKS
    # =========================================================================
    
    def _check_regex_patterns(self):
        """Fallback regex-based checks when Lark unavailable."""
        
        # Strip multiline strings to avoid false matches inside string literals
        # Replace triple-quoted strings with placeholders first
        source_stripped = self.source
        source_stripped = re.sub(r'""".*?"""', '""', source_stripped, flags=re.DOTALL)
        source_stripped = re.sub(r"'''.*?'''", "''", source_stripped, flags=re.DOTALL)
        # Then single-quoted strings (non-greedy to handle multiple on same line)
        source_stripped = re.sub(r'"[^"\n]*"', '""', source_stripped)
        source_stripped = re.sub(r"'[^'\n]*'", "''", source_stripped)
        
        # Find all variable definitions
        var_pattern = r'(?:ayanmo|let|var)\s+(\w+)'
        for match in re.finditer(var_pattern, source_stripped):
            self.ctx.defined_vars.add(match.group(1))
        
        # Find all variable usages (rough approximation)
        word_pattern = r'\b([a-zA-Z_]\w*)\b'
        for match in re.finditer(word_pattern, source_stripped):
            word = match.group(1)
            if word not in {'ayanmo', 'let', 'var', 'if', 'else', 'while', 'for', 'in',
                           'true', 'false', 'otito', 'eke', 'iba', 'import'}:
                self.ctx.used_vars.add(word)


# =============================================================================
# CLI INTERFACE
# =============================================================================

def lint_file(filepath: str, verbose: bool = False) -> List[LintMessage]:
    """Lint a single file and return messages."""
    if not os.path.exists(filepath):
        return [LintMessage(Severity.ERROR, "E000", f"File not found: {filepath}")]
    
    with open(filepath, 'r', encoding='utf-8') as f:
        source = f.read()
    
    linter = IfaLinter(source, filepath)
    return linter.lint()


def lint_cli(args: list):
    """Command-line interface for linter."""
    if not args:
        print("""
╔══════════════════════════════════════════════════════════════╗
║              BABALAWO - The Ifá-Lang Linter                  ║
║             "The Wise One Who Finds Problems"                ║
╚══════════════════════════════════════════════════════════════╝

Usage:
    ifa lint <file.ifa>       Check a file for issues
    ifa lint <dir>            Check all .ifa files in directory
    ifa lint --help           Show this help

Error Codes:
    E001-E099   Syntax errors
    E100-E199   Variable errors (undefined, etc.)
    E200-E299   Type errors (Orí system)
    W001-W099   General warnings
    W100-W199   Unused code warnings
    S001-S099   Style suggestions
""")
        return
    
    target = args[0]
    
    # Handle directory
    if os.path.isdir(target):
        files = []
        for root, _, filenames in os.walk(target):
            for fn in filenames:
                if fn.endswith('.ifa'):
                    files.append(os.path.join(root, fn))
    else:
        files = [target]
    
    total_errors = 0
    total_warnings = 0
    
    for filepath in files:
        messages = lint_file(filepath)
        
        if messages:
            print(f"\n📄 {filepath}")
            print("   " + "─" * 50)
            
            for msg in messages:
                print(msg)
                if msg.severity == Severity.ERROR:
                    total_errors += 1
                elif msg.severity == Severity.WARNING:
                    total_warnings += 1
    
    # Summary
    print(f"""
════════════════════════════════════════════════════════════════
📊 Lint Summary: {total_errors} errors, {total_warnings} warnings
""")
    
    if total_errors > 0:
        print("   ❌ Fix errors before proceeding!")
    elif total_warnings > 0:
        print("   ⚠️  Consider addressing warnings.")
    else:
        print("   ✅ No issues found. Àṣẹ!")


# =============================================================================
# MAIN
# =============================================================================

if __name__ == "__main__":
    import sys
    lint_cli(sys.argv[1:])
