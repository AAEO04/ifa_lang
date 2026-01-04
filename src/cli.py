# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                          IFÁ-LANG CLI                                        ║
║                    "The Oracle's Command Interface"                          ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  ifa run <file.ifa>      - Interpret and execute                             ║
║  ifa build <file.ifa>    - Transpile to Rust and compile to binary           ║
║  ifa debug <file.ifa>    - Run with Babalawo debugger                        ║
║  ifa check <file.ifa>    - Verify Ìwà-Pẹ̀lẹ́ (balance)                         ║
║  ifa matrix              - Show 256 instruction matrix                       ║
║  ifa library             - Show standard library                             ║
║  ifa repl                - Interactive REPL                                  ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import sys
import os
import subprocess
import argparse

# Add project root to path for imports
current_dir = os.path.dirname(os.path.abspath(__file__))
project_root = os.path.dirname(current_dir)
sys.path.insert(0, project_root)

# =============================================================================
# FALLBACK IMPORTS
# =============================================================================

# Parser/Compiler
try:
    from src.parser import EseCompiler, EseRuntime
except ImportError:
    try:
        from ese_parser import EseCompiler, EseRuntime
    except ImportError:
        EseCompiler = None
        EseRuntime = None

# Validator (Smart Compiler with Ìwà Engine)
try:
    from src.validator import SmartIfaCompiler, IwaEngine
except ImportError:
    try:
        from smart_compiler import SmartIfaCompiler, IwaEngine
    except ImportError:
        SmartIfaCompiler = None
        IwaEngine = None

# Transpiler (unified - Lark AST with regex fallback)
try:
    from src.transpiler import IfaRustTranspiler
except ImportError:
    try:
        from ifa_rust import IfaRustTranspiler
    except ImportError:
        IfaRustTranspiler = None

# Debugger
try:
    from src.vm import OponVM, BabalawoDebugger
except ImportError:
    try:
        from babalawo_debugger import OponVM, BabalawoDebugger
    except ImportError:
        OponVM = None
        BabalawoDebugger = None

# GPC Debugger (Grandparent-Parent-Child call stack)
try:
    from src.gpc import GPCStack, GPCDebugger, gpc_debugger
except ImportError:
    GPCStack = None
    GPCDebugger = None
    gpc_debugger = None

# ISA Matrix
try:
    from src.isa import IfaISA
except ImportError:
    try:
        from ifa_amulu import IfaISA
    except ImportError:
        IfaISA = None

# Standard Library / Memory
try:
    from src.memory import IfaStandardLibrary, Calabash4K
except ImportError:
    try:
        from ifa_12bit import IfaStandardLibrary, Calabash4K
    except ImportError:
        IfaStandardLibrary = None
        Calabash4K = None

# Documentation Generator
try:
    from src.docgen import generate_docs
except ImportError:
    generate_docs = None

# Error System (Babalawo)
try:
    from src.errors import babalawo, speak as babalawo_speak
except ImportError:
    babalawo = None
    babalawo_speak = None

# Interpreter (for instant execution like Python/JS)
try:
    from src.interpreter import IfaInterpreter, SimpleParser, run_file
except ImportError:
    IfaInterpreter = None
    SimpleParser = None
    run_file = None

# Bytecode compiler (for .ifab format)
try:
    from src.bytecode import (
        BytecodeCompiler, BytecodeSerializer, BytecodeVM, 
        BytecodeChunk, disassemble as disasm_bytecode
    )
    from src.lark_parser import IfaLarkParser
except ImportError:
    BytecodeCompiler = None
    BytecodeSerializer = None
    BytecodeVM = None
    BytecodeChunk = None
    disasm_bytecode = None
    IfaLarkParser = None

# Ọjà Package Manager
try:
    from src.oja import OjaMarket, oja_cli
except ImportError:
    OjaMarket = None
    oja_cli = None

# Babalawo Linter
try:
    from src.linter import lint_file, lint_cli
except ImportError:
    lint_file = None
    lint_cli = None

# Ifá Test Runner
try:
    from src.test_runner import run_cli as test_cli
except ImportError:
    test_cli = None

# Ìgbálẹ̀ Sandbox
try:
    from src.sandbox import (
        IgbaleRuntime, OgbeConfig, IgbaleContainer,
        run_sandboxed, PLATFORM
    )
    SANDBOX_AVAILABLE = True
except ImportError:
    SANDBOX_AVAILABLE = False
    PLATFORM = "unknown"

def cmd_oja_add(args):
    """
    Handle 'ifa oja add <url>'
    1. Check for ifa.toml
    2. Add dependency to .build/Cargo.toml
    """
    url = args.url
    print(f"📦 Adding package from {url}...")
    
    # 1. ensure ifa.toml exists
    if not os.path.exists("ifa.toml"):
        print("❌ No ifa.toml found. Is this an Ifá project?")
        return 1
        
    # extract name from url (e.g. github.com/user/lib-name.git -> lib-name)
    name = url.split("/")[-1].replace(".git", "")
    
    # 2. Update .build/Cargo.toml (The Shadow Cargo)
    cargo_path = os.path.join(".build", "Cargo.toml")
    
    if not os.path.exists(cargo_path):
        print(f"⚠️  .build/Cargo.toml not found. Run 'ifa build' first to initialize.")
        return 1
        
    try:
        with open(cargo_path, "r", encoding="utf-8") as f:
            lines = f.readlines()
            
        with open(cargo_path, "w", encoding="utf-8") as f:
            in_deps = False
            added = False
            for line in lines:
                f.write(line)
                if line.strip() == "[dependencies]":
                    in_deps = True
                    # Check if already exists? For now just append
                    f.write(f'{name} = {{ git = "{url}" }}\n')
                    added = True
            
            if not in_deps:
                # If no [dependencies] section, add it
                f.write(f'\n[dependencies]\n{name} = {{ git = "{url}" }}\n')
                
        print(f"✅ Added dependency '{name}'!")
        print(f"🔗 Linked to: {url}")
        return 0
        
    except Exception as e:
        print(f"❌ Failed to update manifest: {e}")
        return 1

# Ifá Language Server
try:
    from src.lsp import run_server as lsp_server
except ImportError:
    lsp_server = None

# Ifá Debug Adapter
try:
    from src.debug_adapter import run_dap as dap_server
except ImportError:
    dap_server = None


# =============================================================================
# CONSTANTS
# =============================================================================

BANNER = """
╔══════════════════════════════════════════════════════════════════════════════╗
║                              IFÁ-LANG v1.0                                   ║
║             A Yoruba-Inspired Esoteric Programming Language                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  8-bit Amúlù Architecture (256 Instructions)                                 ║
║  12-bit Memory Addressing (4KB Calabash)                                     ║
║  High-Level Ese Syntax (Domain.Method())                                     ║
║  Babalawo Debugger (Proverb-Based Errors)                                    ║
║  Rust Transpiler (Production Binaries)                                       ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

VERSION_INFO = """Ifá-Lang v1.0.0 (Amúlù Edition)
The Yoruba Programming Language
8-bit Architecture | 16 Odù Domains | Ìwà-Pẹ̀lẹ́ Balance
Copyright (c) 2025 Ayomide Alli (Charon)
"""


# =============================================================================
# COMMANDS
# =============================================================================

def cmd_run(args):
    """
    Run an Ifá program using the Python interpreter.
    Like Python/JavaScript - instant execution, no compilation.
    """
    filepath = args.file
    if not os.path.exists(filepath):
        print(f"Error: File not found: {filepath}")
        return 1
    
    print(f"\n🔮 Interpreting {filepath}...")
    print("-" * 50)
    
    # Try the new interpreter first (preferred)
    if IfaInterpreter and SimpleParser:
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                source = f.read()
            
            parser = SimpleParser()
            instructions = parser.parse(source)
            
            interpreter = IfaInterpreter(verbose=getattr(args, 'verbose', False))
            interpreter.execute(instructions)
            
            print("-" * 50)
            print(f"[Àṣẹ] Execution complete.")
            return 0
        except Exception as e:
            print(f"[Ifá] Runtime Error: {e}")
            return 1
    
    # Fallback to EseCompiler + EseRuntime
    elif EseCompiler and EseRuntime:
        with open(filepath, 'r', encoding='utf-8') as f:
            source = f.read()
        
        compiler = EseCompiler()
        runtime = EseRuntime()
        
        try:
            bytecode = compiler.compile(source)
            print(f"Compiled {len(bytecode)} instructions.\n")
            result = runtime.run(bytecode, verbose=getattr(args, 'verbose', False))
            print("-" * 50)
            print(f"[Ifá] Àṣẹ! Exit Value: {result}")
            return 0
        except Exception as e:
            print(f"[Ifá] Error: {e}")
            return 1
    
    else:
        print("Error: No interpreter available. Check imports.")
        return 1


def check_rust_installed():
    """Check if 'cargo' is in the system PATH."""
    if shutil.which("cargo") is None:
        print("❌ Error: The 'Rust' compiler is missing.")
        print("Ifá-Lang needs Rust to forge the final iron (binary).")
        print("\n👉 Please install it easily here: https://rustup.rs/")
        print("   Command: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
        sys.exit(1)

def cmd_build(args):
    """Transpile to Rust and compile to binary using Cargo."""
    import shutil
    import subprocess
    import tempfile
    from pathlib import Path
    
    # 1. Check for Rust prerequisites
    check_rust_installed()

    filepath = args.file
    
    # Security: Validate input file path
    try:
        safe_path = Path(filepath).resolve()
        if not safe_path.exists():
            print(f"Error: File not found: {filepath}")
            return 1
        if safe_path.suffix != '.ifa':
            print("Error: Only .ifa files can be compiled")
            return 1
        # Prevent path traversal
        if '..' in filepath:
            print("Error: Invalid file path")
            return 1
    except Exception:
        print(f"Error: Invalid file path: {filepath}")
        return 1
    
    if not IfaRustTranspiler:
        print("Error: IfaRustTranspiler not available. Check imports.")
        return 1
    
    print(f"\n=== Ifá Production Build: {filepath} ===")
    
    # 1. Transpile to Rust Code
    transpiler = IfaRustTranspiler()
    with open(filepath, 'r', encoding='utf-8') as f:
        source = f.read()
    
    # Check for native modules (alejo)
    # Simple regex scan for now: "iba alejo.name"
    import re
    native_modules = re.findall(r'iba\s+(?:alejo|àlejò)\.(\w+)', source, re.IGNORECASE)
    
    rust_code = transpiler.transpile(source)
    
    # 2. Setup Temporary Cargo Project
    build_dir = tempfile.mkdtemp(prefix="ifa_build_")
    src_dir = os.path.join(build_dir, "src")
    os.makedirs(src_dir)
    
    print(f"📦 Creating build environment in {build_dir}...")
    
    # 3. Create Cargo.toml
    # Use the one from lib/Cargo.toml but update name
    project_name = os.path.splitext(os.path.basename(filepath))[0]
    
    # Determine lib paths
    cli_dir = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.dirname(cli_dir)
    lib_cargo = os.path.join(root_dir, "lib", "Cargo.toml")
    lib_core = os.path.join(root_dir, "lib", "core.rs")
    
    if os.path.exists(lib_cargo):
        with open(lib_cargo, 'r', encoding='utf-8') as f:
            cargo_content = f.read()
            # Replace [lib] section with [bin]
            cargo_content = cargo_content.replace('name = "ifa-runtime"', f'name = "{project_name}"')
            cargo_content = cargo_content.replace('[lib]', '[[bin]]')
            cargo_content = cargo_content.replace('path = "core.rs"', 'path = "src/main.rs"')
            
        with open(os.path.join(build_dir, "Cargo.toml"), 'w', encoding='utf-8') as f:
            f.write(cargo_content)
    else:
        print("Warning: lib/Cargo.toml not found. Using minimal config.")
        with open(os.path.join(build_dir, "Cargo.toml"), 'w') as f:
            f.write(f'[package]\nname = "{project_name}"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n')
            
    # 4. Copy Runtime (core.rs) and inject user code
    # We can't just copy core.rs as main.rs because main.rs needs the `main` function from transpiler output.
    # Instead, we'll append the user code (which has main()) to a modified core.rs
    
    if os.path.exists(lib_core):
        with open(lib_core, 'r', encoding='utf-8') as f:
            core_code = f.read()
    else:
        # Fallback stub
        core_code = "pub struct Opon { }" 
        
    final_code = core_code + "\n\n" + rust_code
    
    with open(os.path.join(src_dir, "main.rs"), 'w', encoding='utf-8') as f:
        f.write(final_code)
        
    # 5. Handle Native Modules (Àlejò)
    project_dir = os.path.dirname(os.path.abspath(filepath))
    for mod in native_modules:
        rs_file = os.path.join(project_dir, f"{mod}.rs")
        if os.path.exists(rs_file):
            print(f"🔗 Linking native module: {mod}.rs")
            shutil.copy(rs_file, os.path.join(src_dir, f"{mod}.rs"))
        else:
            print(f"⚠️  Warning: Native module '{mod}.rs' not found in {project_dir}")

    # 6. Build with Cargo
    print("🚀 Compiling with Cargo (Release Mode)...")
    try:
        # Run cargo build
        proc = subprocess.run(
            ['cargo', 'build', '--release'],
            cwd=build_dir,
            capture_output=True,
            text=True
        )
        
        if proc.returncode != 0:
            print("❌ Build Failed:")
            print(proc.stderr)
            return 1
            
        # 7. Copy Binary to Output
        output_name = args.output or project_name
        if sys.platform == 'win32':
            bin_name = f"{project_name}.exe"
            target_bin = os.path.join(build_dir, "target", "release", bin_name)
            final_output = output_name if output_name.endswith('.exe') else f"{output_name}.exe"
        else:
            bin_name = project_name
            target_bin = os.path.join(build_dir, "target", "release", bin_name)
            final_output = output_name
            
        if os.path.exists(target_bin):
            shutil.copy(target_bin, final_output)
            print(f"\n✅ Build Successful! Output: {final_output}")
            
            # Clean up
            try:
                # shutil.rmtree(build_dir) # Optional: keep for debugging
                pass
            except:
                pass
            return 0
        else:
            print("❌ Binary not found in target dir.")
            return 1
            
    except FileNotFoundError:
        print("❌ Error: 'cargo' not found in PATH. Please install Rust & Cargo.")
        return 1
    except Exception as e:
        print(f"❌ Error during build: {e}")
        return 1


def cmd_debug(args):
    """Run with Babalawo debugger attached."""
    filepath = args.file
    if not os.path.exists(filepath):
        print(f"Error: File not found: {filepath}")
        return 1
    
    if not BabalawoDebugger:
        print("Error: BabalawoDebugger not available. Check imports.")
        return 1
    
    # Check for GPC mode
    use_gpc = getattr(args, 'gpc', False)
    if use_gpc and not gpc_debugger:
        print("Warning: GPC debugger not available. Running without GPC tracing.")
        use_gpc = False
    
    with open(filepath, 'r', encoding='utf-8') as f:
        source = f.read()
    
    print(f"\n=== Ifá Debug: {filepath} ===")
    if use_gpc:
        print("[GPC] Grandparent-Parent-Child tracing ENABLED")
    
    compiler = EseCompiler()
    debugger = BabalawoDebugger()
    runtime = EseRuntime()
    
    try:
        bytecode = compiler.compile(source)
        print(f"Compiled {len(bytecode)} instructions.")
        print("Running with debugger attached...\n")
        
        # Push main frame to GPC stack if enabled
        if use_gpc:
            gpc_debugger.stack.push("main", line=1, file=filepath)
        
        # Execute with debug hooks
        for i, (opcode, value, desc) in enumerate(bytecode):
            print(f"[{i:03d}] {desc}")
            
            # Push function calls to GPC stack
            if use_gpc and "CALL" in desc:
                func_name = desc.split()[-1] if desc.split() else f"func_{i}"
                gpc_debugger.stack.push(func_name, line=i, file=filepath)
            
            # Check for division by zero
            if "DIV" in desc and value == 0:
                context = {
                    'binary': opcode,
                    'registers': runtime.registers.copy(),
                    'reason': 'Division by Zero'
                }
                # Show GPC traceback on error
                if use_gpc and len(gpc_debugger.stack) > 0:
                    print("\n╔════ Call Stack (GPC) ════╗")
                    print(gpc_debugger.stack.traceback())
                debugger.diagnose(context)
                return 1
            
            # Pop function returns from GPC stack
            if use_gpc and "RET" in desc and len(gpc_debugger.stack) > 1:
                gpc_debugger.stack.pop()
            
            if opcode == "00000000":
                print("\n[Ọ̀yẹ̀kú] Clean Exit.")
                if use_gpc:
                    gpc_debugger.stack.clear()
                return 0
        
        if use_gpc:
            gpc_debugger.stack.clear()
        return 0
    except Exception as e:
        context = {
            'binary': '00000000',
            'registers': {'OKE': 0, 'ISALE': 0, 'OTUN': 0, 'OSI': 0},
            'reason': str(e)
        }
        # Show GPC traceback on exception
        if use_gpc and gpc_debugger and len(gpc_debugger.stack) > 0:
            print("\n╔════ Call Stack (GPC) ════╗")
            print(gpc_debugger.stack.traceback())
        debugger.diagnose(context)
        return 1


def cmd_check(args):
    """
    Check code balance (Ìwà-Pẹ̀lẹ́).
    With --ebo flag, also validates Ẹbọ sacrifice block lifecycles.
    """
    filepath = args.file
    if not os.path.exists(filepath):
        print(f"Error: File not found: {filepath}")
        return 1
    
    with open(filepath, 'r', encoding='utf-8') as f:
        source = f.read()
    
    print(f"\n=== Ifá Check: {filepath} ===")
    
    # Check for Ebo mode
    use_ebo = getattr(args, 'ebo', False)
    if use_ebo:
        print("[Ẹbọ] Sacrifice block lifecycle validation ENABLED")
    
    # Use SmartIfaCompiler if available, otherwise basic analysis
    if SmartIfaCompiler:
        print("[Ìwà] Using Smart Compiler for balance check...")
        compiler = SmartIfaCompiler(strict_mode=True)
        compiler.parse(source)
        
        is_valid = compiler.validate()
        
        # Additional Ẹbọ lifecycle validation
        if use_ebo:
            print("\n[Ẹbọ] Checking sacrifice block lifecycles...")
            ebo_errors = []
            
            # Check for ebo.begin without ebo.sacrifice
            ebo_begins = source.lower().count('ebo.begin')
            ebo_sacrifices = source.lower().count('ebo.sacrifice')
            if ebo_begins != ebo_sacrifices:
                ebo_errors.append(f"  • ebo.begin: {ebo_begins}, ebo.sacrifice: {ebo_sacrifices} (mismatch)")
            
            # Check for ase.begin without ase.end
            ase_begins = source.lower().count('ase.begin')
            ase_ends = source.lower().count('ase.end')
            if ase_begins != ase_ends:
                ebo_errors.append(f"  • ase.begin: {ase_begins}, ase.end: {ase_ends} (mismatch)")
            
            if ebo_errors:
                print("[Ẹbọ] ✗ Sacrifice block violations found:")
                for err in ebo_errors:
                    print(err)
                is_valid = False
            else:
                print("[Ẹbọ] ✓ All sacrifice blocks properly paired.")
        
        if is_valid:
            print("[Ìwà] ✓ All resources balanced. The code has good character.")
            return 0
        else:
            print("[Ìwà] ✗ Balance violations found!")
            return 1
    else:
        # Fallback: basic bit balance analysis
        if not EseCompiler:
            print("Error: No compiler available for analysis.")
            return 1
        
        compiler = EseCompiler()
        bytecode = compiler.compile(source)
        
        ire_count = 0
        ibi_count = 0
        
        for opcode, _, _ in bytecode:
            for char in opcode:
                if char == '1':
                    ire_count += 1
                elif char == '0':
                    ibi_count += 1
        
        balance = abs(ire_count - ibi_count)
        total = ire_count + ibi_count
        ratio = balance / total if total > 0 else 0
        
        print(f"\nÌwà-Pẹ̀lẹ́ Analysis:")
        print(f"  Iré (Light/1):    {ire_count}")
        print(f"  Ibi (Dark/0):     {ibi_count}")
        print(f"  Net Imbalance:    {balance}")
        print(f"  Balance Ratio:    {ratio:.2%}")
        
        if ratio > 0.3:
            print("\n>> WARNING: Code imbalance too high!")
            print(">> Consider Ẹbọ (refactoring) to restore harmony.")
            return 1
        else:
            print("\n>> SUCCESS: Code possesses Ìwà-Pẹ̀lẹ́ (good character).")
            return 0


def cmd_matrix(args):
    """Show the 256 instruction matrix."""
    print("\n=== The Amúlù Matrix (256 Instructions) ===")
    
    if IfaISA:
        isa = IfaISA()
        isa.print_matrix()
    else:
        # Fallback: Generate basic matrix
        print("\n16x16 Odù × Ese Matrix:\n")
        print("    ", end="")
        for i in range(16):
            print(f"{i:3X}", end=" ")
        print()
        print("    " + "-" * 64)
        
        for row in range(16):
            print(f"{row:2X} |", end=" ")
            for col in range(16):
                opcode = (row << 4) | col
                print(f"{opcode:02X} ", end=" ")
            print()


def cmd_library(args):
    """Browse the standard library."""
    print("\n=== The Great Library (Standard Library) ===")
    
    if IfaStandardLibrary:
        lib = IfaStandardLibrary()
        lib.print_library()
    else:
        # Fallback: List modular stdlib
        print("\n16 Odù Domains:\n")
        domains = [
            ("Ogbè", "1111", "System Initialization"),
            ("Ọ̀yẹ̀kú", "0000", "Process Termination"),
            ("Ìwòrì", "0110", "Time & Iteration"),
            ("Òdí", "1001", "File I/O"),
            ("Ìrosù", "1100", "Console I/O"),
            ("Ọ̀wọ́nrín", "0011", "Random & Chaos"),
            ("Ọ̀bàrà", "1000", "Math Addition"),
            ("Ọ̀kànràn", "0001", "Error Handling"),
            ("Ògúndá", "1110", "Arrays & Process"),
            ("Ọ̀sá", "0111", "Control Flow"),
            ("Ìká", "0100", "String Operations"),
            ("Òtúúrúpọ̀n", "0010", "Math Subtraction"),
            ("Òtúrá", "1011", "Network Operations"),
            ("Ìrẹtẹ̀", "1101", "Memory Management"),
            ("Ọ̀ṣẹ́", "1010", "Graphics Display"),
            ("Òfún", "0101", "Object Creation"),
        ]
        
        for name, binary, desc in domains:
            print(f"  [{binary}] {name:12} - {desc}")


def cmd_repl(args):
    """Start interactive REPL - like Python's interactive mode."""
    print(BANNER)
    print("🔮 Ifá Interactive Shell (REPL)")
    print("Type 'exit' to quit, 'help' for commands.\n")
    
    # Use the new interpreter if available
    if IfaInterpreter and SimpleParser:
        interpreter = IfaInterpreter(verbose=False)
        parser = SimpleParser()
        
        while True:
            try:
                line = input("ifá> ")
                cmd = line.strip().lower()
                
                if cmd == 'exit' or cmd == 'quit':
                    print("Ó dàbọ̀! (Goodbye!)")
                    break
                elif cmd == 'help':
                    print("\nCommands:")
                    print("  exit, quit  - Exit the REPL")
                    print("  clear       - Clear interpreter state")
                    print("  memory      - Show memory state")
                    print("  help        - Show this message")
                    print("\nExample code:")
                    print('  Irosu.fo("Hello World");')
                    print('  Ogbe.bi(50);')
                    print('  Obara.ro(10);')
                    continue
                elif cmd == 'clear':
                    interpreter = IfaInterpreter(verbose=False)
                    print("[Clear] Interpreter state reset.")
                    continue
                elif cmd == 'memory':
                    print(f"  Accumulator: {interpreter.accumulator}")
                    print(f"  Last Result: {interpreter.last_result}")
                    print(f"  Variables: {interpreter.memory}")
                    continue
                elif not line.strip():
                    continue
                
                # Parse and execute
                instructions = parser.parse(line)
                interpreter.execute(instructions)
                
            except KeyboardInterrupt:
                print("\nÓ dàbọ̀!")
                break
            except Exception as e:
                print(f"⚠️ Error: {e}")
    
    # Fallback to old compiler/runtime
    elif EseCompiler and EseRuntime:
        compiler = EseCompiler()
        runtime = EseRuntime()
        
        while True:
            try:
                line = input("ifá> ")
                cmd = line.strip().lower()
                
                if cmd == 'exit' or cmd == 'quit':
                    print("Ó dàbọ̀! (Goodbye!)")
                    break
                elif cmd == 'help':
                    print("Commands: exit, help, clear")
                    continue
                elif cmd == 'clear':
                    runtime = EseRuntime()
                    print("Runtime cleared.")
                    continue
                elif not line.strip():
                    continue
                
                bytecode = compiler.compile(line)
                runtime.run(bytecode, verbose=True)
            except KeyboardInterrupt:
                print("\nÓ dàbọ̀!")
                break
            except Exception as e:
                print(f"Error: {e}")
    else:
        print("Error: No interpreter available for REPL.")
        return 1
    
    return 0


def cmd_bytecode(args):
    """
    Compile to .ifab bytecode format.
    Creates compact binary for IoT/Smart Dust deployment.
    """
    filepath = args.file
    if not os.path.exists(filepath):
        print(f"Error: File not found: {filepath}")
        return 1
    
    if not BytecodeCompiler or not IfaLarkParser:
        print("Error: Bytecode compiler not available.")
        print("  Install Lark: pip install lark")
        return 1
    
    print(f"\n📦 Compiling {filepath} to bytecode...")
    
    try:
        # Parse with Lark
        parser = IfaLarkParser()
        ast = parser.parse_file(filepath)
        
        # Compile to bytecode
        compiler = BytecodeCompiler()
        chunk = compiler.compile(ast)
        
        # Determine output path
        if args.output:
            output = args.output
        else:
            output = filepath.replace('.ifa', '.ifab')
        
        # Serialize
        BytecodeSerializer.save(chunk, output)
        
        # Stats
        with open(filepath, 'r', encoding='utf-8') as f:
            source_size = len(f.read())
        with open(output, 'rb') as f:
            binary_size = len(f.read())
        
        ratio = (1 - binary_size / source_size) * 100 if source_size > 0 else 0
        
        print(f"\n[SUCCESS] Bytecode created: {output}")
        print(f"  Source:   {source_size:,} bytes")
        print(f"  Binary:   {binary_size:,} bytes")
        print(f"  Savings:  {ratio:.1f}%")
        
        # Show disassembly if verbose
        if getattr(args, 'verbose', False):
            print(f"\n{disasm_bytecode(chunk)}")
        
        return 0
        
    except Exception as e:
        print(f"[ERROR] Compilation failed: {e}")
        import traceback
        traceback.print_exc()
        return 1


def cmd_run_bytecode(args):
    """Run a .ifab bytecode file."""
    filepath = args.file
    if not os.path.exists(filepath):
        print(f"Error: File not found: {filepath}")
        return 1
    
    if not BytecodeVM:
        print("Error: Bytecode VM not available.")
        return 1
    
    print(f"\n🚀 Running {filepath}...")
    print("-" * 50)
    
    try:
        chunk = BytecodeSerializer.load(filepath)
        vm = BytecodeVM()
        result = vm.run(chunk, verbose=getattr(args, 'verbose', False))
        print("-" * 50)
        print(f"[Àṣẹ] Exit value: {result}")
        return 0
    except Exception as e:
        print(f"[ERROR] Execution failed: {e}")
        return 1


def cmd_disasm(args):
    """Disassemble a .ifab file."""
    filepath = args.file
    if not os.path.exists(filepath):
        print(f"Error: File not found: {filepath}")
        return 1
    
    if not disasm_bytecode:
        print("Error: Disassembler not available.")
        return 1
    
    chunk = BytecodeSerializer.load(filepath)
    print(disasm_bytecode(chunk))
    return 0


def cmd_doc(args):
    """Generate Ifá Corpus documentation."""
    if not generate_docs:
        print("Error: Documentation generator not available.")
        return 1
    
    print(f"\n=== Ifá Documentation Generator ===")
    output_dir = generate_docs(args.directory, args.output)
    print(f"\n[Ifá Doc] Documentation generated successfully!")
    print(f"[Ifá Doc] Open: {output_dir}/index.html")
    return 0


def cmd_oja(args):
    """Ọjà Package Manager - The Market."""
    if not oja_cli:
        print("Error: Ọjà package manager not available.")
        return 1
    
    oja_cli(args.oja_args)
    return 0


def cmd_test(args):
    """Run tests."""
    if not test_cli:
        print("Error: Test runner not available. Check src/test_runner.py")
        return 1
    
    return test_cli(args)



def cmd_lsp(args):
    """Run Language Server Protocol (LSP) server."""
    if not lsp_server:
        print("Error: LSP server not available. Check src/lsp.py")
        return 1
    
    lsp_server()
    return 0


def cmd_dap(args):
    """Run Debug Adapter Protocol (DAP) server."""
    if not dap_server:
        print("Error: DAP server not available. Check src/debug_adapter.py")
        return 1
    
    dap_server()
    return 0


def cmd_version(args):
    """Show version info."""
    print(VERSION_INFO)
    return 0


def cmd_lint(args):
    """Babalawo Linter - Static analysis."""
    if not lint_cli:
        print("Error: Linter not available. Check src/linter.py")
        return 1
    
    lint_cli(args.lint_args)
    return 0


def cmd_sandbox(args):
    """Sandbox command - run code in isolated container."""
    if not SANDBOX_AVAILABLE:
        print("Error: Sandbox not available. Check src/sandbox.py")
        return 1
    
    subcmd = getattr(args, 'sandbox_cmd', 'run')
    
    if subcmd == 'run':
        # Run file in sandbox
        if not hasattr(args, 'file') or not args.file:
            print("Error: No file specified")
            return 1
        
        timeout = getattr(args, 'timeout', 30.0)
        
        try:
            with open(args.file, 'r', encoding='utf-8') as f:
                code = f.read()
        except Exception as e:
            print(f"Error reading file: {e}")
            return 1
        
        print(f"╔══════════════════════════════════════════╗")
        print(f"║  ÌGBÁLẸ̀ SANDBOX ({PLATFORM})            ║")
        print(f"╚══════════════════════════════════════════╝")
        print(f"Running: {args.file}")
        print(f"Timeout: {timeout}s")
        print("-" * 44)
        
        result = run_sandboxed(code, timeout=timeout)
        
        if result['stdout']:
            print(result['stdout'])
        if result['stderr']:
            print(f"[stderr] {result['stderr']}")
        
        print("-" * 44)
        print(f"Exit: {result['exit_code']} | Duration: {result['duration']:.2f}s")
        
        if result['violations']:
            print(f"Security violations: {len(result['violations'])}")
        
        return 0 if result['success'] else 1
    
    elif subcmd == 'demo':
        # Run demo
        print(f"""
╔══════════════════════════════════════════════════════════════╗
║              ÌGBÁLẸ̀ - THE SACRED GROUND                     ║
║           Ifá-Lang Sandbox ({PLATFORM})                      ║
╚══════════════════════════════════════════════════════════════╝

Sandbox Features:
  ✓ Filesystem Isolation (Òdí)
  ✓ Time/Resource Limits (Ìwòrì)
  ✓ Process Control (Ògúndá)
  ✓ Memory Limits (Òtúúrúpọ̀n)
  ✓ Network Isolation (Ọ̀sá)
  ✓ Security Monitoring (Ọ̀kànràn)

Usage:
  ifa sandbox run script.ifa --timeout 30
  ifa sandbox demo

Docker-Compatible:
  ✓ Container lifecycle (create/start/stop/destroy)
  ✓ Resource quotas (CPU, memory, processes)
  ✓ Virtual filesystem with quotas
  ✓ I/O capture and logging
""")
        
        # Quick demo
        runtime = IgbaleRuntime()
        config = OgbeConfig(name="demo-container")
        container = runtime.create(config)
        container.start()
        
        container.odi_fs.write_file("workspace/hello.txt", "Àṣẹ from Ìgbálẹ̀!")
        content = container.odi_fs.read_file("workspace/hello.txt")
        print(f"Virtual FS test: {content}")
        
        info = container.inspect()
        print(f"Container: {info['Name']} (ID: {info['ID'][:8]}...)")
        print(f"State: {info['State']}")
        print(f"RootFS: {info['RootFS']}")
        
        container.stop()
        runtime.gc()
        
        print("\n[Ìgbálẹ̀] Demo complete!")
        return 0
    
    elif subcmd == 'list':
        runtime = IgbaleRuntime()
        containers = runtime.list()
        if not containers:
            print("No containers running")
        else:
            for c in containers:
                print(f"  {c['Name']}: {c['State']}")
        return 0
    
    return 0


# =============================================================================
# MAIN
# =============================================================================

def main():
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        prog='ifa',
        description='Ifá-Lang - The Yoruba Programming Language',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  ifa run examples/hello.ifa
  ifa build demo.ifa -o myapp
  ifa debug crash_test.ifa
  ifa check balance_test.ifa
  ifa matrix
  ifa library
  ifa repl
"""
    )
    parser.add_argument('--version', '-v', action='store_true', help='Show version')
    
    subparsers = parser.add_subparsers(dest='command')
    
    # run command
    run_parser = subparsers.add_parser('run', help='Run an Ifá program')
    run_parser.add_argument('file', help='Path to .ifa file')
    run_parser.add_argument('--verbose', '-V', action='store_true', help='Verbose output')
    run_parser.set_defaults(func=cmd_run)
    
    # build command
    build_parser = subparsers.add_parser('build', help='Transpile to Rust and compile')
    build_parser.add_argument('file', help='Path to .ifa file')
    build_parser.add_argument('--output', '-o', help='Output binary name')
    build_parser.set_defaults(func=cmd_build)
    
    # debug command
    debug_parser = subparsers.add_parser('debug', help='Run with Babalawo debugger')
    debug_parser.add_argument('file', help='Path to .ifa file')
    debug_parser.add_argument('--gpc', action='store_true', 
                             help='Enable GPC (Grandparent-Parent-Child) call stack tracing')
    debug_parser.set_defaults(func=cmd_debug)
    
    # check command
    check_parser = subparsers.add_parser('check', help='Check code balance (Ìwà)')
    check_parser.add_argument('file', help='Path to .ifa file')
    check_parser.add_argument('--ebo', action='store_true',
                             help='Enable Ẹbọ (sacrifice block) lifecycle validation')
    check_parser.set_defaults(func=cmd_check)
    
    # matrix command
    matrix_parser = subparsers.add_parser('matrix', help='Show 256 instruction matrix')
    matrix_parser.set_defaults(func=cmd_matrix)
    
    # library command
    library_parser = subparsers.add_parser('library', help='Browse standard library')
    library_parser.set_defaults(func=cmd_library)
    
    # repl command
    repl_parser = subparsers.add_parser('repl', help='Interactive REPL')
    repl_parser.set_defaults(func=cmd_repl)
    
    # compile command (alias for build, for backward compat)
    compile_parser = subparsers.add_parser('compile', help='Compile to Rust (alias for build)')
    compile_parser.add_argument('file', help='Path to .ifa file')
    compile_parser.add_argument('--output', '-o', help='Output file path')
    compile_parser.set_defaults(func=cmd_build)
    
    # bytecode command (compile to .ifab)
    bytecode_parser = subparsers.add_parser('bytecode', help='Compile to .ifab bytecode')
    bytecode_parser.add_argument('file', help='Path to .ifa file')
    bytecode_parser.add_argument('--output', '-o', help='Output .ifab file')
    bytecode_parser.add_argument('--verbose', '-V', action='store_true', help='Show disassembly')
    bytecode_parser.set_defaults(func=cmd_bytecode)
    
    # runb command (run bytecode)
    runb_parser = subparsers.add_parser('runb', help='Run .ifab bytecode file')
    runb_parser.add_argument('file', help='Path to .ifab file')
    runb_parser.add_argument('--verbose', '-V', action='store_true', help='Verbose output')
    runb_parser.set_defaults(func=cmd_run_bytecode)
    
    # disasm command
    disasm_parser = subparsers.add_parser('disasm', help='Disassemble .ifab file')
    disasm_parser.add_argument('file', help='Path to .ifab file')
    disasm_parser.set_defaults(func=cmd_disasm)
    
    # doc command
    doc_parser = subparsers.add_parser('doc', help='Generate Ifá Corpus documentation')
    doc_parser.add_argument('directory', nargs='?', default='.', help='Source directory')
    doc_parser.add_argument('--output', '-o', default='docs', help='Output directory')
    doc_parser.set_defaults(func=cmd_doc)
    
    # oja command (package manager)
    oja_parser = subparsers.add_parser('oja', help='Ọjà Package Manager')
    oja_parser.add_argument('oja_args', nargs='*', help='Ọjà subcommand and args')
    oja_parser.set_defaults(func=cmd_oja)
    
    # lint command (static analysis)
    lint_parser = subparsers.add_parser('lint', help='Babalawo Linter - static analysis')
    lint_parser.add_argument('lint_args', nargs='*', help='Files/dirs to lint')
    lint_parser.set_defaults(func=cmd_lint)
    
    # test command (unit tests)
    test_parser = subparsers.add_parser('test', help='Run Tests (Idanwo)')
    test_parser.add_argument('files', nargs='*', help='Test files or directories')
    test_parser.set_defaults(func=cmd_test)
    
    # lsp command (language server)
    lsp_parser = subparsers.add_parser('lsp', help='Run Language Server (JSON-RPC)')
    lsp_parser.add_argument('--stdio', action='store_true', help='Use stdio transport (default)')
    lsp_parser.set_defaults(func=cmd_lsp)

    # dap command (debug adapter)
    dap_parser = subparsers.add_parser('dap', help='Run Debug Adapter (DAP)')
    dap_parser.set_defaults(func=cmd_dap)
    
    # sandbox command (Ìgbálẹ̀ container)
    sandbox_parser = subparsers.add_parser('sandbox', 
        help='Run in Ìgbálẹ̀ sandbox (isolated container)')
    sandbox_subparsers = sandbox_parser.add_subparsers(dest='sandbox_cmd')
    
    # sandbox run
    sandbox_run = sandbox_subparsers.add_parser('run', help='Run file in sandbox')
    sandbox_run.add_argument('file', help='Ifá file to run')
    sandbox_run.add_argument('--timeout', '-t', type=float, default=30.0,
                            help='Max runtime in seconds (default: 30)')
    sandbox_run.set_defaults(func=cmd_sandbox, sandbox_cmd='run')
    
    # sandbox demo
    sandbox_demo = sandbox_subparsers.add_parser('demo', help='Run sandbox demo')
    sandbox_demo.set_defaults(func=cmd_sandbox, sandbox_cmd='demo')
    
    # sandbox list
    sandbox_list = sandbox_subparsers.add_parser('list', help='List containers')
    sandbox_list.set_defaults(func=cmd_sandbox, sandbox_cmd='list')
    
    sandbox_parser.set_defaults(func=cmd_sandbox, sandbox_cmd='demo')
    
    args = parser.parse_args()
    
    if args.version:
        return cmd_version(args)
    
    if not args.command:
        print(BANNER)
        parser.print_help()
        return 0
    
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
