# -*- coding: utf-8 -*-
"""
IFA-LANG API & FFI - CROSS-LANGUAGE BRIDGE
"The Gates Between Worlds"

API:  Expose Ifa functions to be called from Python/Rust/C
FFI:  Call external libraries (C, Rust, Python) from Ifa-Lang

The 16 Odu as API Endpoints:
  - Each Odu domain becomes a REST/RPC-like module
  - Methods expose functionality to foreign code
"""

import ctypes
import json
import struct
import os
import sys
from typing import Any, Callable, Dict, List, Optional, Union
from functools import wraps


# =============================================================================
# IFA FFI TYPE SYSTEM
# =============================================================================
class IfaType:
    """Base type for FFI marshalling."""
    U8 = "u8"       # Unsigned 8-bit (0-255)
    I32 = "i32"     # Signed 32-bit
    F64 = "f64"     # 64-bit float
    STR = "str"     # UTF-8 string
    PTR = "ptr"     # Raw pointer
    ARR = "arr"     # Array
    OBJ = "obj"     # Object/Dict
    VOID = "void"   # No return

# C Type mapping
CTYPE_MAP = {
    IfaType.U8: ctypes.c_uint8,
    IfaType.I32: ctypes.c_int32,
    IfaType.F64: ctypes.c_double,
    IfaType.STR: ctypes.c_char_p,
    IfaType.PTR: ctypes.c_void_p,
    IfaType.VOID: None,
}


# =============================================================================
# THE FFI BRIDGE (Calling External Code from Ifá)
# =============================================================================
class IfaFFI:
    """
    Foreign Function Interface - Call C/Rust libraries from Ifá-Lang.
    Maps to Ọ̀wọ́nrín (The Exchanger) domain.
    """
    
    def __init__(self):
        self.loaded_libs: Dict[str, ctypes.CDLL] = {}
        self.registered_funcs: Dict[str, Callable] = {}
    
    def load_library(self, name: str, path: str = None) -> bool:
        """
        Load a shared library (.dll, .so, .dylib).
        Ifá syntax: Ọ̀wọ́nrín.gbe("mylib")
        """
        try:
            if path:
                lib = ctypes.CDLL(path)
            else:
                # Try common patterns
                if sys.platform == 'win32':
                    lib = ctypes.CDLL(f"{name}.dll")
                elif sys.platform == 'darwin':
                    lib = ctypes.CDLL(f"lib{name}.dylib")
                else:
                    lib = ctypes.CDLL(f"lib{name}.so")
            
            self.loaded_libs[name] = lib
            print(f"[FFI] Loaded library: {name}")
            return True
        except OSError as e:
            print(f"[FFI] Failed to load {name}: {e}")
            return False
    
    def bind_function(self, lib_name: str, func_name: str, 
                      arg_types: List[str], ret_type: str) -> Callable:
        """
        Bind a C function with type information.
        Ifá syntax: Ọ̀wọ́nrín.so("libc", "strlen", ["str"], "i32")
        """
        if lib_name not in self.loaded_libs:
            raise RuntimeError(f"Library not loaded: {lib_name}")
        
        lib = self.loaded_libs[lib_name]
        func = getattr(lib, func_name)
        
        # Set argument types
        func.argtypes = [CTYPE_MAP.get(t, ctypes.c_void_p) for t in arg_types]
        
        # Set return type
        func.restype = CTYPE_MAP.get(ret_type, ctypes.c_int)
        
        # Create wrapper
        def wrapper(*args):
            # Convert args to C types
            c_args = []
            for arg, atype in zip(args, arg_types):
                if atype == IfaType.STR:
                    c_args.append(arg.encode('utf-8') if isinstance(arg, str) else arg)
                else:
                    c_args.append(arg)
            return func(*c_args)
        
        self.registered_funcs[f"{lib_name}.{func_name}"] = wrapper
        return wrapper
    
    def call(self, lib_name: str, func_name: str, *args) -> Any:
        """
        Call a bound FFI function.
        Ifá syntax: Ọ̀wọ́nrín.pe("libc", "strlen", "hello")
        """
        key = f"{lib_name}.{func_name}"
        if key in self.registered_funcs:
            return self.registered_funcs[key](*args)
        raise RuntimeError(f"Function not bound: {key}")
    
    def call_python(self, module_name: str, func_name: str, *args) -> Any:
        """
        Call a Python function from Ifá.
        Ifá syntax: Ọ̀wọ́nrín.pe_py("math", "sqrt", 16)
        """
        import importlib
        mod = importlib.import_module(module_name)
        func = getattr(mod, func_name)
        return func(*args)


# =============================================================================
# THE API LAYER (Exposing Ifá to External Code)
# =============================================================================
class IfaAPI:
    """
    API Layer - Expose Ifá functions to be called from other languages.
    Creates callable endpoints from Odù domains.
    """
    
    def __init__(self):
        self.endpoints: Dict[str, Callable] = {}
        self.middleware: List[Callable] = []
    
    def expose(self, name: str, handler: Callable, 
               arg_types: List[str] = None, ret_type: str = None):
        """
        Expose a function as an API endpoint.
        """
        @wraps(handler)
        def wrapper(*args, **kwargs):
            # Run middleware
            for mw in self.middleware:
                args, kwargs = mw(name, args, kwargs)
            return handler(*args, **kwargs)
        
        self.endpoints[name] = {
            "handler": wrapper,
            "arg_types": arg_types or [],
            "ret_type": ret_type or "void"
        }
        return wrapper
    
    def call(self, name: str, *args, **kwargs) -> Any:
        """
        Call an exposed API endpoint.
        """
        if name not in self.endpoints:
            raise RuntimeError(f"Unknown endpoint: {name}")
        return self.endpoints[name]["handler"](*args, **kwargs)
    
    def add_middleware(self, func: Callable):
        """Add middleware for request processing."""
        self.middleware.append(func)
    
    def to_json_schema(self) -> Dict:
        """Generate JSON schema for all endpoints."""
        schema = {}
        for name, info in self.endpoints.items():
            schema[name] = {
                "args": info["arg_types"],
                "returns": info["ret_type"]
            }
        return schema


# =============================================================================
# C HEADER GENERATOR (For calling Ifá from C/Rust)
# =============================================================================
class IfaCHeaderGenerator:
    """
    Generates C header files for FFI binding.
    Allows C/Rust programs to call Ifá functions.
    """
    
    @staticmethod
    def generate_header(api: IfaAPI, filename: str = "ifa_api.h"):
        """Generate a C header file."""
        type_map = {
            "u8": "uint8_t",
            "i32": "int32_t",
            "f64": "double",
            "str": "const char*",
            "ptr": "void*",
            "void": "void",
        }
        
        lines = [
            "/* IFA-LANG C API - Auto-generated */",
            "#ifndef IFA_API_H",
            "#define IFA_API_H",
            "",
            "#include <stdint.h>",
            "",
            "#ifdef __cplusplus",
            'extern "C" {',
            "#endif",
            "",
        ]
        
        for name, info in api.endpoints.items():
            ret = type_map.get(info["ret_type"], "void")
            args = ", ".join(
                f"{type_map.get(t, 'void*')} arg{i}" 
                for i, t in enumerate(info["arg_types"])
            ) or "void"
            
            c_name = name.replace(".", "_")
            lines.append(f"{ret} ifa_{c_name}({args});")
        
        lines.extend([
            "",
            "#ifdef __cplusplus",
            "}",
            "#endif",
            "",
            "#endif /* IFA_API_H */",
        ])
        
        with open(filename, 'w') as f:
            f.write("\n".join(lines))
        
        print(f"[API] Generated C header: {filename}")
        return filename


# =============================================================================
# RUST FFI GENERATOR
# =============================================================================
class IfaRustFFIGenerator:
    """Generate Rust FFI bindings."""
    
    @staticmethod
    def generate_bindings(api: IfaAPI, filename: str = "ifa_bindings.rs"):
        """Generate Rust FFI bindings."""
        type_map = {
            "u8": "u8",
            "i32": "i32",
            "f64": "f64",
            "str": "*const c_char",
            "ptr": "*mut c_void",
            "void": "()",
        }
        
        lines = [
            "// IFA-LANG Rust Bindings - Auto-generated",
            "use std::os::raw::{c_char, c_void};",
            "",
            "extern \"C\" {",
        ]
        
        for name, info in api.endpoints.items():
            ret = type_map.get(info["ret_type"], "()")
            args = ", ".join(
                f"arg{i}: {type_map.get(t, '*mut c_void')}"
                for i, t in enumerate(info["arg_types"])
            )
            
            rust_name = name.replace(".", "_")
            lines.append(f"    pub fn ifa_{rust_name}({args}) -> {ret};")
        
        lines.extend([
            "}",
            "",
        ])
        
        with open(filename, 'w') as f:
            f.write("\n".join(lines))
        
        print(f"[API] Generated Rust bindings: {filename}")
        return filename


# =============================================================================
# IFA RPC SERVER (JSON-RPC Style)
# =============================================================================
class IfaRPCServer:
    """
    RPC Server for remote Ifá function calls.
    Uses JSON-RPC-like protocol.
    """
    
    def __init__(self, api: IfaAPI):
        self.api = api
    
    def handle_request(self, request_json: str) -> str:
        """Handle a JSON-RPC request."""
        try:
            req = json.loads(request_json)
            method = req.get("method", "")
            params = req.get("params", [])
            req_id = req.get("id", 1)
            
            result = self.api.call(method, *params)
            
            return json.dumps({
                "jsonrpc": "2.0",
                "result": result,
                "id": req_id
            })
        except Exception as e:
            return json.dumps({
                "jsonrpc": "2.0",
                "error": {"code": -1, "message": str(e)},
                "id": req.get("id", 1) if 'req' in dir() else 1
            })
    
    def start_http_server(self, port: int = 8080):
        """Start a simple HTTP server for RPC."""
        from http.server import HTTPServer, BaseHTTPRequestHandler
        
        api = self.api
        handle = self.handle_request
        
        class RPCHandler(BaseHTTPRequestHandler):
            def do_POST(self):
                length = int(self.headers['Content-Length'])
                body = self.rfile.read(length).decode('utf-8')
                
                result = handle(body)
                
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.end_headers()
                self.wfile.write(result.encode('utf-8'))
        
        server = HTTPServer(('', port), RPCHandler)
        print(f"[RPC] Server started on port {port}")
        server.serve_forever()


# =============================================================================
# IFA STDLIB AS API (Exposing all 16 Odù)
# =============================================================================
def create_stdlib_api():
    """
    Create an API from the 16-Odù stdlib.
    Each Odù becomes an API module.
    """
    from ifa_stdlib import IfaStdLib
    
    api = IfaAPI()
    std = IfaStdLib()
    
    # Expose Ogbè (System)
    api.expose("ogbe.version", std.Ogbe.version, [], "str")
    api.expose("ogbe.cwd", std.Ogbe.cwd, [], "str")
    api.expose("ogbe.gba", std.Ogbe.gba, [], "arr")
    
    # Expose Ọ̀bàrà (Math)
    api.expose("obara.fikun", std.Obara.fikun, ["i32", "i32"], "i32")
    api.expose("obara.isodipupo", std.Obara.isodipupo, ["i32", "i32"], "i32")
    api.expose("obara.agbara", std.Obara.agbara, ["i32", "i32"], "i32")
    api.expose("obara.gbongbo", std.Obara.gbongbo, ["f64"], "f64")
    
    # Expose Òtúúrúpọ̀n (Math)
    api.expose("oturupon.din", std.Oturupon.din, ["i32", "i32"], "i32")
    api.expose("oturupon.pin", std.Oturupon.pin, ["i32", "i32"], "f64")
    api.expose("oturupon.ku", std.Oturupon.ku, ["i32", "i32"], "i32")
    
    # Expose Ìká (Strings)
    api.expose("ika.so", std.Ika.so, ["str", "str"], "str")
    api.expose("ika.gigun", std.Ika.gigun, ["str"], "i32")
    api.expose("ika.hash", std.Ika.hash_, ["str"], "str")
    
    # Expose Ọ̀wọ́nrín (Random)
    api.expose("owonrin.afesona", std.Owonrin.afesona, ["i32", "i32"], "i32")
    api.expose("owonrin.uuid", std.Owonrin.uuid, [], "str")
    
    # Expose Ìwòrì (Time)
    api.expose("iwori.akoko", std.Iwori.akoko, [], "str")
    api.expose("iwori.epoch", std.Iwori.epoch, [], "i32")
    
    return api


# =============================================================================
# 2026 CEN MODEL - SECURE FFI (Sandboxed Imports)
# =============================================================================

class SecurityError(Exception):
    """Raised when a security violation is detected."""
    pass


class SecureFFI:
    """
    Secure Foreign Function Interface with module whitelisting.
    Blocks dangerous operations while allowing safe Python imports.
    """
    
    DEFAULT_WHITELIST = {'math', 'json', 'datetime', 'collections', 'itertools', 
                         'functools', 'statistics', 'random', 'string', 're'}
    FORBIDDEN_ATTRS = {'system', 'popen', 'spawn', 'exec', 'eval', 'compile',
                       'remove', 'rmdir', 'unlink', 'rmtree', '__import__', 
                       '__builtins__', 'subprocess', 'Popen'}
    
    def __init__(self, whitelist: set = None, strict_mode: bool = True):
        self.whitelist = whitelist or self.DEFAULT_WHITELIST.copy()
        self.strict_mode = strict_mode
        self.imported_modules = {}
    
    def import_module(self, name: str, alias: str = None):
        """Safely import a Python module."""
        if name not in self.whitelist:
            raise SecurityError(f"Module '{name}' is not in the whitelist. "
                              f"Allowed: {sorted(self.whitelist)}")
        
        import importlib
        mod = importlib.import_module(name)
        key = alias or name
        self.imported_modules[key] = mod
        return mod
    
    def add_to_whitelist(self, name: str):
        """Add a module to the whitelist."""
        self.whitelist.add(name)
    
    def __repr__(self):
        return f"SecureFFI(whitelist={sorted(self.whitelist)}, strict={self.strict_mode})"


# =============================================================================
# DEMO
# =============================================================================
if __name__ == "__main__":
    print("""
=== IFA-LANG API & FFI DEMO ===
""")
    
    # === FFI Demo: Call Python's math module ===
    print("=== FFI Demo: Calling Python's math module ===")
    ffi = IfaFFI()
    result = ffi.call_python("math", "sqrt", 256)
    print(f"  math.sqrt(256) = {result}")
    
    result = ffi.call_python("math", "factorial", 5)
    print(f"  math.factorial(5) = {result}")
    
    # === API Demo: Expose Ifá functions ===
    print("\n=== API Demo: Exposing Ifá functions ===")
    
    api = IfaAPI()
    
    # Expose some functions
    @api.expose("ifa.add", arg_types=["i32", "i32"], ret_type="i32")
    def ifa_add(a, b):
        return a + b
    
    @api.expose("ifa.greet", arg_types=["str"], ret_type="str")
    def ifa_greet(name):
        return f"Ẹ káàbọ̀, {name}!"
    
    # Call via API
    print(f"  api.call('ifa.add', 10, 20) = {api.call('ifa.add', 10, 20)}")
    print(f"  api.call('ifa.greet', 'World') = {api.call('ifa.greet', 'World')}")
    
    # Generate JSON schema
    print("\n=== API Schema (JSON) ===")
    print(json.dumps(api.to_json_schema(), indent=2))
    
    # === Generate C Header ===
    print("\n=== Generating C Header ===")
    IfaCHeaderGenerator.generate_header(api, "ifa_api.h")
    
    # === Generate Rust Bindings ===
    print("\n=== Generating Rust Bindings ===")
    IfaRustFFIGenerator.generate_bindings(api, "ifa_bindings.rs")
    
    # === RPC Demo ===
    print("\n=== RPC Demo: JSON-RPC Request ===")
    rpc = IfaRPCServer(api)
    
    request = json.dumps({
        "jsonrpc": "2.0",
        "method": "ifa.add",
        "params": [100, 50],
        "id": 1
    })
    print(f"  Request: {request}")
    response = rpc.handle_request(request)
    print(f"  Response: {response}")
    
    # === Stdlib API ===
    print("\n=== Stdlib API Demo ===")
    try:
        stdlib_api = create_stdlib_api()
        print(f"  stdlib_api.call('obara.fikun', 5, 3) = {stdlib_api.call('obara.fikun', 5, 3)}")
        print(f"  stdlib_api.call('ika.gigun', 'hello') = {stdlib_api.call('ika.gigun', 'hello')}")
        print(f"  stdlib_api.call('iwori.akoko') = {stdlib_api.call('iwori.akoko')}")
    except ImportError:
        print("  (ifa_stdlib not found, skipping)")
    
    print("\n[Done] API and FFI implementation complete!")
