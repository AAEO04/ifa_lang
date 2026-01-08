# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ-LANG COMPILER PACKAGE                                 ║
║                    The Soul, Voice, Conscience, and Hand                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Modules:                                                                    ║
║    • lexer.py      - Tokenizer with Yoruba Unicode support                   ║
║    • parser.py     - Ese Parser (high-level syntax to bytecode)              ║
║    • validator.py  - Ìwà Engine (balance/resource checker)                   ║
║    • transpiler.py - Rust code generator                                     ║
║    • vm.py         - OponVM with Babalawo debugger                           ║
║    • ffi.py        - Foreign Function Interface                              ║
║    • isa.py        - Amúlù 8-bit Instruction Set (256 ops)                   ║
║    • memory.py     - 12-bit addressing, 4KB Calabash memory                  ║
║    • cli.py        - Command-line interface                                  ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import sys

# Add project root to path
_project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, _project_root)


# =============================================================================
# VERSION
# =============================================================================
__version__ = "1.0.0"


# =============================================================================
# LAZY IMPORTS WITH FALLBACKS
# =============================================================================

def _import_with_fallback(src_module, src_class, fallback_module, fallback_class=None):
    """Import from src/ first, fall back to project root."""
    fallback_class = fallback_class or src_class
    try:
        mod = __import__(f'src.{src_module}', fromlist=[src_class])
        return getattr(mod, src_class)
    except (ImportError, AttributeError):
        try:
            mod = __import__(fallback_module, fromlist=[fallback_class])
            return getattr(mod, fallback_class)
        except (ImportError, AttributeError):
            return None


# Parser/Compiler
EseCompiler = _import_with_fallback('parser', 'EseCompiler', 'ese_parser')
EseRuntime = _import_with_fallback('parser', 'EseRuntime', 'ese_parser')

# Validator
SmartIfaCompiler = _import_with_fallback('validator', 'SmartIfaCompiler', 'smart_compiler')
IwaEngine = _import_with_fallback('validator', 'IwaEngine', 'smart_compiler')

# Transpiler
IfaRustTranspiler = _import_with_fallback('transpiler', 'IfaRustTranspiler', 'ifa_rust')

# VM/Debugger
OponVM = _import_with_fallback('vm', 'OponVM', 'babalawo_debugger')
BabalawoDebugger = _import_with_fallback('vm', 'BabalawoDebugger', 'babalawo_debugger')

# FFI
IfaFFI = _import_with_fallback('ffi', 'IfaFFI', 'ifa_ffi')
IfaAPI = _import_with_fallback('ffi', 'IfaAPI', 'ifa_ffi')

# ISA (from new isa.py or fallback to ifa_amulu.py)
IfaISA = _import_with_fallback('isa', 'IfaISA', 'ifa_amulu')

# Memory (from new memory.py or fallback to ifa_12bit.py)
Calabash4K = _import_with_fallback('memory', 'Calabash4K', 'ifa_12bit')
Odu12Bit = _import_with_fallback('memory', 'Odu12Bit', 'ifa_12bit')
IfaStandardLibrary = _import_with_fallback('memory', 'IfaStandardLibrary', 'ifa_12bit')

# Documentation Generator
try:
    from src.docgen import generate_docs, IfaDocParser, IfaDocGenerator
except ImportError:
    generate_docs = None
    IfaDocParser = None
    IfaDocGenerator = None

# Error System (Babalawo)
try:
    from src.errors import Babalawo, babalawo, speak, IfaError
except ImportError:
    Babalawo = None
    babalawo = None
    speak = None
    IfaError = None

# Interpreter (instant execution like Python/JS)
try:
    from src.interpreter import IfaInterpreter, SimpleParser, run_file, run_code
except ImportError:
    IfaInterpreter = None
    SimpleParser = None
    run_file = None
    run_code = None

# Lark Parser (formal EBNF grammar)
try:
    from src.lark_parser import IfaLarkParser, Program, OduCall, VarDecl
except ImportError:
    IfaLarkParser = None
    Program = None
    OduCall = None
    VarDecl = None


# =============================================================================
# EXPORTS
# =============================================================================
__all__ = [
    # Version
    '__version__',
    
    # Parser
    'EseCompiler',
    'EseRuntime',
    
    # Validator
    'SmartIfaCompiler',
    'IwaEngine',
    
    # Transpiler
    'IfaRustTranspiler',
    
    # VM
    'OponVM',
    'BabalawoDebugger',
    
    # FFI
    'IfaFFI',
    'IfaAPI',
    
    # ISA
    'IfaISA',
    
    # Memory
    'Calabash4K',
    'Odu12Bit',
    'IfaStandardLibrary',
    
    # Documentation
    'generate_docs',
    'IfaDocParser',
    'IfaDocGenerator',
    
    # Errors
    'Babalawo',
    'babalawo',
    'speak',
    'IfaError',
    
    # Interpreter
    'IfaInterpreter',
    'SimpleParser',
    'run_file',
    'run_code',
    
    # Lark Parser (formal grammar)
    'IfaLarkParser',
    'Program',
    'OduCall',
    'VarDecl',
]
