# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ INTERPRETER - THE LIVE VM                             ║
║                    Instant Execution Like Python/JavaScript                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Dual-Mode Execution:                                                        ║
║    ifa run <file>   → Interpreted (Instant start, Python speed)              ║
║    ifa build <file> → Compiled (Slow start, Rust speed)                      ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import sys
import os
import time
import re
from typing import Any, Dict, List, Tuple, Optional

# Try to import standard library
try:
    from lib.std import StandardLibrary
except ImportError:
    StandardLibrary = None

# Try to import error system
try:
    from src.errors import babalawo, speak
except ImportError:
    babalawo = None
    speak = None

# Try to import Àjọṣe relationship engine
try:
    from src.ajose import AjosePredicateEngine
except ImportError:
    AjosePredicateEngine = None


# =============================================================================
# OOP SUPPORT: IfaObject (dá instances)
# =============================================================================

class IfaObject:
    """
    Represents an object instance created via odù.dá().
    Stores fields as a dictionary and supports method dispatch.
    """
    def __init__(self, class_name: str, fields: Dict[str, Any] = None, methods: Dict[str, callable] = None):
        self.class_name = class_name
        self.fields = fields or {}
        self.methods = methods or {}
    
    def get_field(self, name: str) -> Any:
        return self.fields.get(name, None)
    
    def set_field(self, name: str, value: Any):
        self.fields[name] = value
    
    def call_method(self, name: str, args: List[Any] = None) -> Any:
        if name in self.methods:
            return self.methods[name](self, *(args or []))
        raise AttributeError(f"Object '{self.class_name}' has no method '{name}'")
    
    def __repr__(self):
        return f"<{self.class_name} Object with {len(self.fields)} fields>"


# =============================================================================
# THE INTERPRETER
# =============================================================================

class IfaInterpreter:
    """
    The Live VM - Executes Ifá code directly in Python.
    No Rust generation, no compilation - just instant execution.
    """
    
    def __init__(self, verbose: bool = False):
        # Memory state (The Opon/Divination Board)
        self.memory: Dict[str, Any] = {}
        self.last_result: Any = None
        self.accumulator: int = 0
        self.verbose = verbose
        
        # Standard library instances (lazily created)
        self._stdlib: Dict[str, Any] = {}
        
        # Try to import OturaDomain for proper networking
        try:
            from lib.std.otura import OturaDomain
            self._otura = OturaDomain()
        except ImportError:
            self._otura = None
        
        # File handles for Òdí operations
        self._file_handles: Dict[str, Any] = {}
        
        # Network state for Òtúrá operations
        self._network_state: Dict[str, Any] = {}
        
        # Graphics buffer for Ọ̀ṣẹ́ operations
        self._screen: List[List[str]] = [[' ' for _ in range(40)] for _ in range(20)]
        
        # OOP: Class registry (class_name -> {fields, methods})
        self._classes: Dict[str, Dict[str, Any]] = {}
        
        # Àjọṣe: Reactive relationship engine
        if AjosePredicateEngine:
            self.ajose = AjosePredicateEngine()
            
            # Define reactive relationships
            self.ajose.define("VariableChange", "Interpreter", "Variable")
            
            # Subscribe to variable changes
            @self.ajose.when("VariableChange")
            def on_var_change(source, target, context):
                if self.verbose:
                    print(f"[Àjọṣe] {context.get('name', '?')}: {context.get('old', '?')} → {context.get('new', '?')}")
        else:
            self.ajose = None
    
    def execute(self, instructions: List[Tuple[str, str, str]]):
        """
        Runs the instructions directly in Python.
        
        Each instruction is a tuple: (domain, verse, args)
        Example: ("irosu", "fo", '"Hello World"')
        """
        for domain, verse, args in instructions:
            key = f"{domain.lower()}.{verse.lower()}"
            
            try:
                result = self._dispatch(key, args)
                if result is not None:
                    self.last_result = result
            except Exception as e:
                self._handle_error(key, args, e)
    
    def _dispatch(self, key: str, args: str) -> Optional[Any]:
        """Dispatch to the appropriate handler based on key."""
        
        # ========== OOP: Constructor Call (ClassName.dá) ==========
        parts = key.split('.')
        class_name = parts[0]
        method_name = parts[1] if len(parts) > 1 else ""
        
        # Check if this is a constructor call (dá / da / new)
        if class_name in self._classes and method_name in ['dá', 'da', 'new']:
            parsed_args = self._split_args(args) if args else []
            resolved_args = [self._resolve_value(a) for a in parsed_args]
            return self.instantiate(class_name, resolved_args)
        
        # Check if this is a method call on an object variable
        if class_name in self.memory:
            obj = self.memory[class_name]
            if isinstance(obj, IfaObject):
                parsed_args = self._split_args(args) if args else []
                resolved_args = [self._resolve_value(a) for a in parsed_args]
                return self.call_method_on_object(obj, method_name, resolved_args)

        # ========== SPECIAL: CLASS DEFINITION (__odu__) ==========
        if key == "__odu__":
            # Class was already registered during parsing, just acknowledge
            if self.verbose:
                print(f"[OOP] Class defined: {args}")
            return None
        
        # ========== SPECIAL: MATCH STATEMENT (__match__) ==========
        if key == "__match__":
            # args format: "[(pattern, action), ...]" as string
            var_name = args  # The variable being matched is stored as the verse
            # Actually the key is "__match__" and args contains the arms
            return self._execute_match(class_name, args)  # class_name holds var name

        # ========== ÌROSÙ (OUTPUT/CONSOLE) ==========
        if key == "irosu.fo" or key == "ìrosù.fọ̀":
            # Print to console (like console.log or print)
            value = self._resolve_value(args)
            print(f"[Ìrosù] {value}")
            return value
            
        elif key == "irosu.gba" or key == "ìrosù.gbà":
            # Read from console (like input())
            prompt = self._resolve_value(args) if args else ""
            return input(f"[Ìrosù] {prompt}")
            
        elif key == "irosu.nu" or key == "ìrosù.nù":
            # Clear screen
            os.system('cls' if os.name == 'nt' else 'clear')
            return None
        
        # ========== OGBÈ (INIT/VARIABLES) ==========
        elif key == "ogbe.bi" or key == "ogbè.bí":
            # Initialize/birth a value
            self.last_result = self._parse_arg(args)
            if self.verbose:
                print(f"[Ogbè] Born: {self.last_result}")
            return self.last_result
            
        elif key == "ogbe.fi" or key == "ogbè.fí":
            # Store in variable: ogbe.fi("name", value)
            parts = self._split_args(args)
            if len(parts) >= 2:
                name = self._parse_arg(parts[0])
                old_val = self.memory.get(name)
                new_val = self._parse_arg(parts[1])
                
                self.memory[name] = new_val
                
                # Notify Àjọṣe of variable change
                if self.ajose:
                    self.ajose.link("VariableChange", self, name,
                                   name=name, old=old_val, new=new_val)
                
                return new_val
            return None
            
        elif key == "ogbe.wa" or key == "ogbè.wá":
            # Get variable
            name = self._parse_arg(args)
            return self.memory.get(name, None)
        
        # ========== ỌBÀRÀ (MATH - ADDITION) ==========
        elif key == "obara.ro" or key == "ọ̀bàrà.rò":
            # Add to accumulator
            val = self._parse_arg(args)
            if isinstance(val, (int, float)):
                self.accumulator += val
                self.last_result = self.accumulator
            return self.accumulator
            
        elif key == "obara.isiro" or key == "ọ̀bàrà.ìṣírò":
            # Multiply
            val = self._parse_arg(args)
            if isinstance(val, (int, float)):
                self.accumulator *= val
                self.last_result = self.accumulator
            return self.accumulator
        
        # ========== ÒTÚÚRÚPỌ̀N (MATH - SUBTRACTION/DIVISION) ==========
        elif key == "oturupon.din" or key == "òtúúrúpọ̀n.dín":
            # Subtract from accumulator
            val = self._parse_arg(args)
            if isinstance(val, (int, float)):
                self.accumulator -= val
                self.last_result = self.accumulator
            return self.accumulator
            
        elif key == "oturupon.pin" or key == "òtúúrúpọ̀n.pín":
            # Divide
            val = self._parse_arg(args)
            if isinstance(val, (int, float)) and val != 0:
                self.accumulator //= val
                self.last_result = self.accumulator
            elif val == 0:
                if speak:
                    print(speak("DIVISION_BY_ZERO", 0))
                else:
                    print("⚠️ Error: Division by zero!")
            return self.accumulator
            
        elif key == "oturupon.ku" or key == "òtúúrúpọ̀n.kù":
            # Modulo
            val = self._parse_arg(args)
            if isinstance(val, (int, float)) and val != 0:
                self.accumulator %= val
                self.last_result = self.accumulator
            return self.accumulator
        
        # ========== ÌWÒRÌ (TIME/LOOPS) ==========
        elif key == "iwori.duro" or key == "ìwòrì.dúró":
            # Sleep/wait (milliseconds)
            ms = self._parse_arg(args)
            if isinstance(ms, (int, float)):
                time.sleep(ms / 1000)
            return None
            
        elif key == "iwori.akoko" or key == "ìwòrì.àkókò":
            # Get current time
            return time.time()
        
        # ========== ÒDÍ (FILE I/O) ==========
        elif key == "odi.si" or key == "òdí.ṣí":
            # Open file
            parts = self._split_args(args)
            filename = self._parse_arg(parts[0]) if parts else "output.txt"
            mode = self._parse_arg(parts[1]) if len(parts) > 1 else "r"
            
            # Security: Validate file path (prevent directory traversal)
            if not self._validate_file_path(filename):
                print("[Security] File access denied - path outside allowed directories")
                return False
            
            try:
                self._file_handles[filename] = open(filename, mode, encoding='utf-8')
                if self.verbose:
                    print(f"[Òdí] Opened: {filename}")
                return True
            except Exception as e:
                if speak:
                    print(speak("FILE_NOT_FOUND", 0, path=filename))
                else:
                    print(f"⚠️ File error: {e}")
                return False
                
        elif key == "odi.ka" or key == "òdí.kà":
            # Read file
            filename = self._parse_arg(args)
            if filename in self._file_handles:
                return self._file_handles[filename].read()
            return None
            
        elif key == "odi.ko" or key == "òdí.kọ":
            # Write to file
            parts = self._split_args(args)
            filename = self._parse_arg(parts[0]) if parts else None
            content = self._parse_arg(parts[1]) if len(parts) > 1 else str(self.last_result)
            if filename in self._file_handles:
                self._file_handles[filename].write(str(content))
                return True
            return False
            
        elif key == "odi.pa" or key == "òdí.pà":
            # Close file
            filename = self._parse_arg(args)
            if filename in self._file_handles:
                self._file_handles[filename].close()
                del self._file_handles[filename]
                if self.verbose:
                    print(f"[Òdí] Closed: {filename}")
                return True
            return False
        
        # ========== ÒTÚRÁ (NETWORK) ==========
        elif key == "otura.ran" or key == "òtúrá.rán":
            # HTTP POST or TCP send
            parts = self._split_args(args)
            url_or_data = self._resolve_value(parts[0]) if parts else ""
            data = self._resolve_value(parts[1]) if len(parts) > 1 else None
            
            if not self._validate_network_target(url_or_data):
                print(f"[Security] Network access denied - blocked destination")
                return False
            
            # If we have an active socket, send data
            if hasattr(self, '_active_socket') and self._active_socket:
                try:
                    self._active_socket.sendall(str(url_or_data).encode('utf-8'))
                    if self.verbose:
                        print(f"[Òtúrá] Sent via socket: {len(str(url_or_data))} bytes")
                    return True
                except Exception as e:
                    print(f"[Òtúrá] Socket send error: {e}")
                    return False
            
            # Otherwise, HTTP POST
            try:
                import urllib.request
                import urllib.parse
                
                if data:
                    # POST request
                    post_data = urllib.parse.urlencode({'data': str(data)}).encode('utf-8')
                    req = urllib.request.Request(str(url_or_data), data=post_data, method='POST')
                else:
                    # GET request
                    req = urllib.request.Request(str(url_or_data))
                
                req.add_header('User-Agent', 'Ifa-Lang/1.0')
                with urllib.request.urlopen(req, timeout=10) as response:
                    result = response.read().decode('utf-8')
                    self.last_result = result
                    if self.verbose:
                        print(f"[Òtúrá] HTTP {'POST' if data else 'GET'} → {url_or_data[:50]}...")
                    return result
            except Exception as e:
                print(f"[Òtúrá] HTTP error: {e}")
                return False
            
        elif key == "otura.gba" or key == "òtúrá.gbà":
            # HTTP GET or TCP receive
            parts = self._split_args(args)
            url_or_size = self._resolve_value(parts[0]) if parts else 1024
            timeout = int(self._resolve_value(parts[1])) if len(parts) > 1 else 10
            
            # If we have an active socket, receive data
            if hasattr(self, '_active_socket') and self._active_socket:
                try:
                    size = int(url_or_size) if str(url_or_size).isdigit() else 1024
                    self._active_socket.settimeout(timeout)
                    data = self._active_socket.recv(size).decode('utf-8')
                    self.last_result = data
                    if self.verbose:
                        print(f"[Òtúrá] Received via socket: {len(data)} bytes")
                    return data
                except Exception as e:
                    print(f"[Òtúrá] Socket recv error: {e}")
                    return None
            
            # Otherwise, HTTP GET
            url = str(url_or_size)
            if not self._validate_network_target(url):
                print(f"[Security] Network access denied - blocked destination")
                return None
            
            try:
                import urllib.request
                req = urllib.request.Request(url)
                req.add_header('User-Agent', 'Ifa-Lang/1.0')
                with urllib.request.urlopen(req, timeout=timeout) as response:
                    result = response.read().decode('utf-8')
                    self.last_result = result
                    if self.verbose:
                        print(f"[Òtúrá] HTTP GET ← {url[:50]}...")
                    return result
            except Exception as e:
                print(f"[Òtúrá] HTTP error: {e}")
                return None
        
        elif key == "otura.de" or key == "òtúrá.dé":
            # TCP Connect: Otura.de(host, port)
            parts = self._split_args(args)
            host = self._resolve_value(parts[0]) if parts else "localhost"
            port = int(self._resolve_value(parts[1])) if len(parts) > 1 else 80
            
            if not self._validate_network_target(host):
                print(f"[Security] Network access denied - blocked destination")
                return False
            
            try:
                import socket
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                sock.settimeout(10)
                sock.connect((str(host), port))
                self._active_socket = sock
                self._network_state['connected'] = True
                if self.verbose:
                    print(f"[Òtúrá] Connected to {host}:{port}")
                return True
            except Exception as e:
                print(f"[Òtúrá] Connect error: {e}")
                return False
        
        elif key == "otura.tetisi" or key == "òtúrá.tẹ́tisí":
            # TCP Listen: Otura.tetisi(port)
            port = int(self._resolve_value(args)) if args else 8080
            
            try:
                import socket
                server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                server.bind(('0.0.0.0', port))
                server.listen(1)
                server.settimeout(30)
                if self.verbose:
                    print(f"[Òtúrá] Listening on port {port}...")
                conn, addr = server.accept()
                self._active_socket = conn
                self._network_state['server'] = server
                self._network_state['client_addr'] = addr
                if self.verbose:
                    print(f"[Òtúrá] Client connected from {addr}")
                return True
            except Exception as e:
                print(f"[Òtúrá] Listen error: {e}")
                return False
        
        elif key == "otura.pa" or key == "òtúrá.pà":
            # Close connection
            try:
                if hasattr(self, '_active_socket') and self._active_socket:
                    self._active_socket.close()
                    self._active_socket = None
                if self._network_state.get('server'):
                    self._network_state['server'].close()
                self._network_state.clear()
                if self.verbose:
                    print(f"[Òtúrá] Connection closed")
                return True
            except Exception as e:
                print(f"[Òtúrá] Close error: {e}")
                return False
        
        # ========== ÌKÁ (STRINGS) ==========
        elif key == "ika.sopo" or key == "ìká.ṣọpọ̀":
            # String concatenation
            val = self._parse_arg(args)
            self.last_result = str(self.last_result or "") + str(val)
            return self.last_result
        
        elif key == "ika.so" or key == "ìká.sọ":
            # Multi-arg string concatenation: Ika.so(a, b, c, ...) -> "abc..."
            parts = self._split_args(args)
            resolved = [str(self._resolve_value(p)) for p in parts]
            self.last_result = "".join(resolved)
            return self.last_result
        
        elif key == "ika.gigun" or key == "ìká.gígùn":
            # String length
            val = self._resolve_value(args)
            return len(str(val)) if val else 0
            
        elif key == "ika.gun" or key == "ìká.gùn":
            # String length (alternate)
            val = self._resolve_value(args)
            return len(str(val)) if val else 0
        
        # ========== ỌBÀRÀ (MATH - ADDITION) EXTENDED ==========
        elif key == "obara.fikun" or key == "ọ̀bàrà.fikun":
            # Add two values: Obara.fikun(a, b) -> a + b
            parts = self._split_args(args)
            if len(parts) >= 2:
                a = self._resolve_value(parts[0])
                b = self._resolve_value(parts[1])
                if isinstance(a, (int, float)) and isinstance(b, (int, float)):
                    result = a + b
                else:
                    result = str(a) + str(b)
                self.last_result = result
                return result
            return 0
        
        # ========== ÒGÚNDÁ (ARRAYS/LISTS) ==========
        elif key == "ogunda.da" or key == "ògúndá.dá":
            # Create new array: Ogunda.da() -> []
            return []
        
        elif key == "ogunda.fi" or key == "ògúndá.fí":
            # Add to array: Ogunda.fi(arr, value) -> arr with value appended
            parts = self._split_args(args)
            if len(parts) >= 2:
                arr = self._resolve_value(parts[0])
                val = self._resolve_value(parts[1])
                if isinstance(arr, list):
                    arr.append(val)
                    return arr
                else:
                    # Create new list with value
                    return [val]
            return []
        
        elif key == "ogunda.gigun" or key == "ògúndá.gígùn":
            # Array length: Ogunda.gigun(arr) -> len(arr)
            arr = self._resolve_value(args)
            if isinstance(arr, list):
                return len(arr)
            return 0
        
        elif key == "ogunda.gba" or key == "ògúndá.gbà":
            # Get from array: Ogunda.gba(arr, index) -> arr[index]
            parts = self._split_args(args)
            if len(parts) >= 2:
                arr = self._resolve_value(parts[0])
                idx = self._resolve_value(parts[1])
                if isinstance(arr, list) and isinstance(idx, int):
                    if 0 <= idx < len(arr):
                        return arr[idx]
            return None
        
        elif key == "ogunda.mu" or key == "ògúndá.mú":
            # Remove last from array: Ogunda.mu(arr) -> removed element
            arr = self._resolve_value(args)
            if isinstance(arr, list) and arr:
                return arr.pop()
            return None
        
        # ========== ỌSẸ (GRAPHICS) ==========
        elif key == "ose.ya" or key == "ọ̀ṣẹ́.yà":
            # Draw at position
            parts = self._split_args(args)
            if len(parts) >= 3:
                x, y, char = int(parts[0]), int(parts[1]), str(parts[2])
                if 0 <= x < 40 and 0 <= y < 20:
                    self._screen[y][x] = char[0] if char else '#'
            return None
            
        elif key == "ose.han" or key == "ọ̀ṣẹ́.hàn":
            # Render screen
            print("+" + "-" * 40 + "+")
            for row in self._screen:
                print("|" + "".join(row) + "|")
            print("+" + "-" * 40 + "+")
            return None
            
        elif key == "ose.nu" or key == "ọ̀ṣẹ́.nù":
            # Clear screen buffer
            self._screen = [[' ' for _ in range(40)] for _ in range(20)]
            return None
        
        # ========== ỌYẸKÚ (EXIT/HALT) ==========
        elif key == "oyeku.ku" or key == "ọ̀yẹ̀kú.kú":
            # Terminate process
            print("[Ọ̀yẹ̀kú] Process Terminated.")
            sys.exit(0)
            
        elif key == "oyeku.sun" or key == "ọ̀yẹ̀kú.sùn":
            # Sleep (longer)
            ms = self._parse_arg(args)
            if isinstance(ms, (int, float)):
                time.sleep(ms / 1000)
            return None
        
        # ========== ỌWỌNRÍN (RANDOM) ==========
        elif key == "owonrin.yan" or key == "ọ̀wọ́nrín.yàn":
            # Random number
            import random
            parts = self._split_args(args)
            if len(parts) >= 2:
                low, high = int(parts[0]), int(parts[1])
                self.last_result = random.randint(low, high)
            else:
                self.last_result = random.randint(0, 255)
            return self.last_result
        
        # ========== ÍROSÙ (CONSOLE I/O) ==========
        elif key == "irosu.fo" or key == "írosù.fọ́":
            # Print output: Irosu.fo(message)
            msg = self._resolve_value(args)
            print(msg)
            return msg
        
        elif key == "irosu.kigbe" or key == "írosù.kígbe":
            # Print error: Irosu.kigbe(message)
            msg = self._resolve_value(args)
            print(f"⚠️ {msg}", file=sys.stderr)
            return msg
        
        elif key == "irosu.gba" or key == "írosù.gbà":
            # Read input: Irosu.gba(prompt)
            prompt = self._resolve_value(args) or "> "
            result = input(str(prompt))
            self.last_result = result
            return result
        
        elif key == "irosu.so_owo" or key == "írosù.sọ_ọwọ́":
            # Format with colors (ANSI)
            parts = self._split_args(args)
            msg = self._resolve_value(parts[0]) if parts else ""
            color = self._resolve_value(parts[1]) if len(parts) > 1 else "white"
            colors = {"red": "\033[91m", "green": "\033[92m", "yellow": "\033[93m", 
                     "blue": "\033[94m", "white": "\033[97m", "reset": "\033[0m"}
            code = colors.get(str(color).lower(), "")
            print(f"{code}{msg}\033[0m")
            return msg
        
        # ========== ỌKÀNRÀN (ERROR HANDLING) ==========
        elif key == "okanran.ju" or key == "ọkànràn.jù":
            # Throw error: Okanran.ju(message)
            msg = self._resolve_value(args)
            raise RuntimeError(f"[Ọkànràn] {msg}")
        
        elif key == "okanran.mu" or key == "ọkànràn.mú":
            # Store error handler (simplified - stores in memory)
            handler = self._resolve_value(args)
            self.memory['_error_handler'] = handler
            return True
        
        elif key == "okanran.gba_asise" or key == "ọkànràn.gbà_àṣìṣe":
            # Get last error
            return self.memory.get('_last_error', None)
        
        # ========== ỌSÀ (CONCURRENCY/LOCKING) ==========
        elif key == "osa.khoa" or key == "ọ̀sà.kóà":
            # Acquire lock: Osa.khoa(name)
            import threading
            lock_name = self._resolve_value(args)
            if '_locks' not in self.memory:
                self.memory['_locks'] = {}
            if lock_name not in self.memory['_locks']:
                self.memory['_locks'][lock_name] = threading.Lock()
            self.memory['_locks'][lock_name].acquire()
            if self.verbose:
                print(f"[Ọ̀sà] Lock acquired: {lock_name}")
            return True
        
        elif key == "osa.si" or key == "ọ̀sà.sí":
            # Release lock: Osa.si(name)
            lock_name = self._resolve_value(args)
            if '_locks' in self.memory and lock_name in self.memory['_locks']:
                try:
                    self.memory['_locks'][lock_name].release()
                    if self.verbose:
                        print(f"[Ọ̀sà] Lock released: {lock_name}")
                    return True
                except RuntimeError:
                    return False
            return False
        
        # ========== ÌRẸTẸ̀ (MEMORY MANAGEMENT) ==========
        elif key == "irete.ya" or key == "ìrẹtẹ̀.yá":
            # Allocate memory region: Irete.ya(size)
            size = int(self._resolve_value(args) or 256)
            # Security: Limit allocation size
            MAX_ALLOC = 1024 * 1024  # 1MB max
            if size > MAX_ALLOC:
                print(f"[Security] Allocation too large: {size} > {MAX_ALLOC}")
                return None
            addr = len(self.memory.get('_heap', []))
            if '_heap' not in self.memory:
                self.memory['_heap'] = []
            self.memory['_heap'].extend([0] * size)
            if self.verbose:
                print(f"[Ìrẹtẹ̀] Allocated {size} bytes at {addr}")
            return addr
        
        elif key == "irete.tu" or key == "ìrẹtẹ̀.tú":
            # Free memory (mark as available - simplified GC)
            addr = int(self._resolve_value(args) or 0)
            if self.verbose:
                print(f"[Ìrẹtẹ̀] Freed address {addr}")
            return True
        
        # ========== ỌFÚN (OBJECTS) ==========
        elif key == "ofun.da" or key == "ọ̀fún.dá":
            # Create new object: Ofun.da()
            return {}
        
        elif key == "ofun.fi" or key == "ọ̀fún.fí":
            # Set property: Ofun.fi(obj, key, value)
            parts = self._split_args(args)
            if len(parts) >= 3:
                obj = self._resolve_value(parts[0])
                key_name = self._resolve_value(parts[1])
                value = self._resolve_value(parts[2])
                if isinstance(obj, dict):
                    obj[str(key_name)] = value
                    return obj
            return None
        
        elif key == "ofun.gba" or key == "ọ̀fún.gbà":
            # Get property: Ofun.gba(obj, key)
            parts = self._split_args(args)
            if len(parts) >= 2:
                obj = self._resolve_value(parts[0])
                key_name = self._resolve_value(parts[1])
                if isinstance(obj, dict):
                    return obj.get(str(key_name))
            return None
        
        # ========== EXTENDED ÒDÍ (FILE I/O) ==========
        elif key == "odi.wa" or key == "òdí.wà":
            # Check file exists: Odi.wa(path)
            path = self._resolve_value(args)
            return os.path.exists(str(path))
        
        elif key == "odi.fi" or key == "òdí.fí":
            # Append to file: Odi.fi(path, content)
            parts = self._split_args(args)
            if len(parts) >= 2:
                path = self._resolve_value(parts[0])
                content = self._resolve_value(parts[1])
                if not self._validate_file_path(str(path)):
                    print("[Security] File access denied")
                    return False
                with open(str(path), 'a', encoding='utf-8') as f:
                    f.write(str(content))
                return True
            return False
        
        elif key == "odi.akojo" or key == "òdí.àkójọ":
            # List directory: Odi.akojo(path)
            path = self._resolve_value(args) or "."
            if not self._validate_file_path(str(path)):
                print("[Security] Directory access denied")
                return []
            try:
                return os.listdir(str(path))
            except:
                return []
        
        # ========== EXTENDED ÌKÁ (STRINGS) ==========
        elif key == "ika.ge" or key == "ìká.gé":
            # Substring: Ika.ge(str, start, len)
            parts = self._split_args(args)
            if len(parts) >= 3:
                s = str(self._resolve_value(parts[0]))
                start = int(self._resolve_value(parts[1]))
                length = int(self._resolve_value(parts[2]))
                return s[start:start + length]
            return ""
        
        elif key == "ika.nla" or key == "ìká.nlá":
            # Uppercase: Ika.nla(str)
            s = self._resolve_value(args)
            return str(s).upper() if s else ""
        
        elif key == "ika.kekere" or key == "ìká.kékeré":
            # Lowercase: Ika.kekere(str)
            s = self._resolve_value(args)
            return str(s).lower() if s else ""
        
        elif key == "ika.wa" or key == "ìká.wá":
            # Find substring: Ika.wa(str, substr)
            parts = self._split_args(args)
            if len(parts) >= 2:
                s = str(self._resolve_value(parts[0]))
                sub = str(self._resolve_value(parts[1]))
                return s.find(sub)
            return -1
        
        elif key == "ika.pin" or key == "ìká.pín":
            # Split string: Ika.pin(str, delim)
            parts = self._split_args(args)
            if len(parts) >= 2:
                s = str(self._resolve_value(parts[0]))
                delim = str(self._resolve_value(parts[1]))
                return s.split(delim)
            elif parts:
                return str(self._resolve_value(parts[0])).split()
            return []
        
        # ========== UNKNOWN ==========
        else:
            print(f"⚠️ Runtime Warning: Spirit '{key}' not found in Interpreter.")
            return None
    
    def _parse_arg(self, arg: str) -> Any:
        """Parse an argument string into Python value."""
        if not arg:
            return None
        arg = arg.strip()
        
        # String literal
        if (arg.startswith('"') and arg.endswith('"')) or \
           (arg.startswith("'") and arg.endswith("'")):
            return arg[1:-1]
        
        # Boolean
        if arg.lower() == "otito" or arg.lower() == "true":
            return True
        if arg.lower() == "eke" or arg.lower() == "false":
            return False
        
        # Number
        try:
            if '.' in arg:
                return float(arg)
            return int(arg)
        except ValueError:
            pass
        
        # Variable reference
        if arg in self.memory:
            return self.memory[arg]
        
        # Special: 'last' or 'abajade' returns last result
        if arg.lower() in ('last', 'abajade', ''):
            return self.last_result
        
        return arg
    
    def _resolve_value(self, arg: str) -> Any:
        """Resolve argument, check memory for variables."""
        if not arg or arg == "":
            return self.last_result
        return self._parse_arg(arg)
    
    def _validate_file_path(self, filepath: str) -> bool:
        """Validate file path to prevent directory traversal attacks.
        
        Only allows access to files within:
        - Current working directory
        - data/ subdirectory
        - output/ subdirectory
        """
        if not filepath:
            return False
        
        # Block obvious traversal attempts
        if '..' in filepath:
            return False
        
        # Resolve to absolute path
        abs_path = os.path.abspath(filepath)
        cwd = os.getcwd()
        
        # Must be within working directory
        if not abs_path.startswith(cwd):
            return False
        
        return True
    
    def _validate_network_target(self, url: str) -> bool:
        """Validate network target for SSRF protection.
        
        Blocks:
        - Localhost (127.x.x.x)
        - Private IPs (10.x, 172.16-31.x, 192.168.x)
        - Link-local (169.254.x.x)
        - Cloud metadata (169.254.169.254)
        """
        if not url:
            return False
        
        url_lower = str(url).lower()
        
        # Block common SSRF targets
        blocked_patterns = [
            '127.0.0.1', 'localhost', '0.0.0.0',
            '169.254.169.254',  # AWS/GCP metadata
            '10.', '172.16.', '172.17.', '172.18.', '172.19.',
            '172.20.', '172.21.', '172.22.', '172.23.',
            '172.24.', '172.25.', '172.26.', '172.27.',
            '172.28.', '172.29.', '172.30.', '172.31.',
            '192.168.', '169.254.',
            'file://', 'gopher://', 'dict://',
        ]
        
        for pattern in blocked_patterns:
            if pattern in url_lower:
                return False
        
        return True
    
    def _split_args(self, args: str) -> List[str]:
        """Split comma-separated arguments, respecting quotes."""
        result = []
        current = ""
        in_string = False
        string_char = None
        
        for char in args:
            if char in ('"', "'") and not in_string:
                in_string = True
                string_char = char
                current += char
            elif char == string_char and in_string:
                in_string = False
                current += char
                string_char = None
            elif char == ',' and not in_string:
                result.append(current.strip())
                current = ""
            else:
                current += char
        
        if current.strip():
            result.append(current.strip())
        
        return result
    
    def _handle_error(self, key: str, args: str, error: Exception):
        """Handle runtime errors with Babalawo wisdom."""
        if babalawo:
            domain = key.split('.')[0].upper()
            error_type = type(error).__name__
            
            # Map Python exceptions to Ifá error codes
            error_map = {
                'ZeroDivisionError': 'DIVISION_BY_ZERO',
                'FileNotFoundError': 'FILE_NOT_FOUND',
                'IndexError': 'INDEX_OUT_OF_BOUNDS',
                'KeyError': 'UNDEFINED_VARIABLE',
                'TypeError': 'TYPE_ERROR',
                'ConnectionError': 'CONNECTION_REFUSED',
            }
            
            code = error_map.get(error_type, 'UNHANDLED_EXCEPTION')
            print(speak(code, 0))
        else:
            print(f"⚠️ Error in {key}: {error}")

    # =========================================================================
    # OOP SUPPORT: Class Registration & Instantiation
    # =========================================================================
    
    def register_class(self, class_name: str, fields: Dict[str, Any], methods: Dict[str, callable]):
        """
        Register a class (odù) definition.
        Called when parsing an odù block.
        """
        self._classes[class_name] = {
            'fields': fields.copy(),
            'methods': methods.copy()
        }
        if self.verbose:
            print(f"[OOP] Registered class: {class_name}")
    
    def instantiate(self, class_name: str, args: List[Any] = None) -> IfaObject:
        """
        Create a new instance of a class (call dá constructor).
        Example: Dog.dá("Bingo") -> IfaObject
        """
        if class_name not in self._classes:
            raise NameError(f"Class '{class_name}' is not defined")
        
        class_def = self._classes[class_name]
        
        # 1. Create object with default field values
        obj = IfaObject(
            class_name=class_name,
            fields=class_def['fields'].copy(),
            methods=class_def['methods'].copy()
        )
        
        # 2. Run constructor (dá) if it exists
        if 'dá' in class_def['methods'] or 'da' in class_def['methods'] or 'new' in class_def['methods']:
            constructor = class_def['methods'].get('dá') or class_def['methods'].get('da') or class_def['methods'].get('new')
            if constructor:
                constructor(obj, *(args or []))
        
        return obj
    
    def call_method_on_object(self, obj: IfaObject, method_name: str, args: List[Any] = None) -> Any:
        """
        Call a method on an IfaObject instance.
        Example: my_dog.speak()
        """
        if not isinstance(obj, IfaObject):
            raise TypeError(f"Cannot call method '{method_name}' on non-object")
        
        return obj.call_method(method_name, args)
    
    def _execute_match(self, var_name: str, arms_str: str) -> Any:
        """
        Execute a match statement.
        
        Args:
            var_name: Variable name to match against
            arms_str: String representation of match arms
        
        Returns:
            Result of matched arm execution
        """
        # Get the value to match
        value = self.memory.get(var_name, None)
        if value is None:
            value = self._resolve_value(var_name)
        
        # Parse arms from string representation
        try:
            import ast
            arms = ast.literal_eval(arms_str)  # Safe parsing of literals only
        except (ValueError, SyntaxError):
            arms = []
        
        # Find matching arm
        for pattern, action in arms:
            matched = False
            
            # Wildcard match
            if pattern == "_":
                matched = True
            # String comparison
            elif isinstance(pattern, str):
                if pattern.startswith('"') or pattern.startswith("'"):
                    pattern_val = pattern.strip("\"'")
                    matched = (str(value) == pattern_val)
                elif pattern.isdigit():
                    matched = (value == int(pattern))
                else:
                    matched = (str(value) == pattern)
            # Number comparison
            elif isinstance(pattern, (int, float)):
                matched = (value == pattern)
            else:
                matched = (value == pattern)
            
            if matched:
                # Execute the matched action
                return self._execute_action(action)
        
        return None
    
    def _execute_action(self, action: str) -> Any:
        """Execute a match arm action (simple statement)."""
        action = action.strip()
        
        # Check if it's a method call
        if "." in action and "(" in action:
            # Parse and dispatch
            match = re.match(r'(\w+)\.(\w+)\s*\(\s*(.*?)\s*\)', action)
            if match:
                domain, verse, args = match.groups()
                key = f"{domain.lower()}.{verse.lower()}"
                return self._dispatch(key, args)
        
        # Otherwise treat as expression
        return self._resolve_value(action)


# =============================================================================
# SIMPLE PARSER (for interpreter mode)
# =============================================================================

class SimpleParser:
    """
    Quick parser for interpreter mode.
    Parses Ifá syntax into (domain, verse, args) tuples.
    Also handles OOP (odù), Match (yàn), and Lambdas (->).
    """
    
    # Pattern: Domain.verse(args);
    PATTERN = re.compile(
        r'(\w+)\.(\w+)\s*\(\s*(.*?)\s*\)\s*;?',
        re.UNICODE | re.DOTALL
    )
    
    # Variable assignment: ayanmo name = value; OR let name = value;
    VAR_PATTERN = re.compile(
        r'(?:ayanmo|ayanmọ|let)\s+(\w+)\s*=\s*(.+?)\s*;',
        re.UNICODE
    )
    
    # OOP: odù ClassName { ... } OR class ClassName { ... }
    ODU_PATTERN = re.compile(
        r'(?:odù|odu|class)\s+(\w+)\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}',
        re.UNICODE | re.DOTALL
    )
    
    # Match: yàn (expr) { ... } OR match (expr) { ... }
    MATCH_PATTERN = re.compile(
        r'(?:yàn|yan|match)\s*\(\s*(\w+)\s*\)\s*\{([^}]+)\}',
        re.UNICODE | re.DOTALL
    )
    
    # Field inside class: ayanmo name = value;
    FIELD_PATTERN = re.compile(
        r'(?:ayanmo|ayanmọ|let)\s+(\w+)\s*=\s*(.+?)\s*;',
        re.UNICODE
    )
    
    # Method inside class: ese name(params) { ... } OR func name(params) { ... }
    METHOD_PATTERN = re.compile(
        r'(?:ese|func)\s+(\w+)\s*\(\s*(.*?)\s*\)\s*\{([^}]*)\}',
        re.UNICODE | re.DOTALL
    )
    
    def parse(self, source: str, interpreter=None) -> List[Tuple[str, str, str]]:
        """Parse source code into instructions."""
        instructions = []
        
        # Remove comments
        source = re.sub(r'#.*$', '', source, flags=re.MULTILINE)
        source = re.sub(r'//.*$', '', source, flags=re.MULTILINE)
        
        # 1. Handle OOP class definitions (odù blocks)
        for match in self.ODU_PATTERN.finditer(source):
            class_name, body = match.groups()
            fields, methods = self._parse_class_body(body)
            
            # Register class with interpreter if provided
            if interpreter:
                interpreter.register_class(class_name, fields, methods)
            
            # Add special instruction for class definition
            instructions.append(("__odu__", "define", class_name))
        
        # 2. Handle match statements (yàn blocks)
        for match in self.MATCH_PATTERN.finditer(source):
            var_name, arms_body = match.groups()
            arms = self._parse_match_arms(arms_body)
            # Store as special instruction
            instructions.append(("__match__", var_name, str(arms)))
        
        # 3. Handle variable assignments
        for match in self.VAR_PATTERN.finditer(source):
            name, value = match.groups()
            instructions.append(("ogbe", "fi", f'"{name}", {value}'))
        
        # 4. Handle method calls
        for match in self.PATTERN.finditer(source):
            domain, verse, args = match.groups()
            # Normalize domain names
            domain = self._normalize_domain(domain)
            instructions.append((domain, verse, args))
        
        return instructions
    
    def _parse_class_body(self, body: str) -> Tuple[Dict[str, Any], Dict[str, callable]]:
        """Parse fields and methods from class body."""
        fields = {}
        methods = {}
        
        # Extract fields
        for match in self.FIELD_PATTERN.finditer(body):
            name, value = match.groups()
            # Try to evaluate simple literals
            try:
                if value.startswith('"') or value.startswith("'"):
                    fields[name] = value.strip('"\'')
                elif value.isdigit():
                    fields[name] = int(value)
                elif value in ['true', 'otito']:
                    fields[name] = True
                elif value in ['false', 'eke']:
                    fields[name] = False
                else:
                    fields[name] = value
            except:
                fields[name] = value
        
        # Extract methods (as string bodies for now)
        for match in self.METHOD_PATTERN.finditer(body):
            method_name, params, method_body = match.groups()
            # Store method as a simple callable stub
            param_list = [p.strip() for p in params.split(',') if p.strip()]
            methods[method_name] = self._create_method_stub(method_name, param_list, method_body)
        
        return fields, methods
    
    def _create_method_stub(self, name: str, params: List[str], body: str):
        """Create a callable for a method that executes its body."""
        # Store reference to parser for recursive parsing
        parser_instance = self
        
        def method_impl(obj, *args):
            # 1. Set parameters as fields on the object
            for i, param in enumerate(params):
                if i < len(args):
                    obj.set_field(param, args[i])
            
            # 2. Parse and execute method body
            body_stripped = body.strip()
            if not body_stripped:
                return None
            
            # Look for return statement
            return_value = None
            
            # Split body into statements (simple split by ;)
            statements = [s.strip() for s in body_stripped.split(';') if s.strip()]
            
            for stmt in statements:
                # Handle return statement
                if stmt.startswith('padà ') or stmt.startswith('pada ') or stmt.startswith('return '):
                    expr = stmt.split(' ', 1)[1] if ' ' in stmt else ''
                    return_value = parser_instance._evaluate_expr(expr, obj)
                    return return_value
                
                # Handle field assignment: name = value
                if '=' in stmt and not stmt.startswith('ayanmo') and not stmt.startswith('let'):
                    parts = stmt.split('=', 1)
                    if len(parts) == 2:
                        field_name = parts[0].strip()
                        value_expr = parts[1].strip()
                        value = parser_instance._evaluate_expr(value_expr, obj)
                        obj.set_field(field_name, value)
                        continue
                
                # Handle method calls
                if '.' in stmt and '(' in stmt:
                    match = re.match(r'(\w+)\.(\w+)\s*\(\s*(.*?)\s*\)', stmt)
                    if match:
                        domain, verse, call_args = match.groups()
                        key = f"{domain.lower()}.{verse.lower()}"
                        # Would need interpreter instance here
                        continue
            
            return return_value
        
        return method_impl
    
    def _evaluate_expr(self, expr: str, obj=None) -> Any:
        """Evaluate a simple expression, with optional object context."""
        expr = expr.strip()
        
        # String literal
        if (expr.startswith('"') and expr.endswith('"')) or \
           (expr.startswith("'") and expr.endswith("'")):
            return expr[1:-1]
        
        # Number literal
        try:
            if '.' in expr:
                return float(expr)
            return int(expr)
        except ValueError:
            pass
        
        # Boolean
        if expr in ['true', 'otito']:
            return True
        if expr in ['false', 'eke']:
            return False
        
        # Field reference on object
        if obj and hasattr(obj, 'get_field'):
            field_val = obj.get_field(expr)
            if field_val is not None:
                return field_val
        
        # Variable reference
        return expr
    
    def _parse_match_arms(self, arms_body: str) -> List[Tuple[str, str]]:
        """Parse match arms: pattern => action;"""
        arms = []
        arm_pattern = re.compile(r'(\S+)\s*=>\s*(.+?);', re.UNICODE)
        for match in arm_pattern.finditer(arms_body):
            pattern, action = match.groups()
            arms.append((pattern.strip(), action.strip()))
        return arms
    
    def _normalize_domain(self, domain: str) -> str:
        """Normalize Yoruba domain names to ASCII."""
        mapping = {
            'ìrosù': 'irosu', 'irosu': 'irosu',
            'ogbè': 'ogbe', 'ogbe': 'ogbe',
            'ọ̀yẹ̀kú': 'oyeku', 'oyeku': 'oyeku',
            'ìwòrì': 'iwori', 'iwori': 'iwori',
            'òdí': 'odi', 'odi': 'odi',
            'ọ̀wọ́nrín': 'owonrin', 'owonrin': 'owonrin',
            'ọ̀bàrà': 'obara', 'obara': 'obara',
            'ọ̀kànràn': 'okanran', 'okanran': 'okanran',
            'ògúndá': 'ogunda', 'ogunda': 'ogunda',
            'ọ̀sá': 'osa', 'osa': 'osa',
            'ìká': 'ika', 'ika': 'ika',
            'òtúúrúpọ̀n': 'oturupon', 'oturupon': 'oturupon',
            'òtúrá': 'otura', 'otura': 'otura',
            'ìrẹtẹ̀': 'irete', 'irete': 'irete',
            'ọ̀ṣẹ́': 'ose', 'ose': 'ose',
            'òfún': 'ofun', 'ofun': 'ofun',
        }
        return mapping.get(domain.lower(), domain.lower())


# =============================================================================
# CONVENIENCE FUNCTIONS
# =============================================================================

# Try to import Lark parser (unified pipeline)
try:
    from src.lark_parser import IfaLarkParser, VarDecl, Instruction, OduCall, IfStmt, WhileStmt, ForStmt, TryStmt
    LARK_AVAILABLE = True
except ImportError:
    LARK_AVAILABLE = False
    IfaLarkParser = None


def run_file(filepath: str, verbose: bool = False, use_lark: bool = True):
    """Run an Ifá file directly.
    
    Uses Lark AST parser by default (unified pipeline).
    Falls back to SimpleParser if Lark unavailable.
    """
    with open(filepath, 'r', encoding='utf-8') as f:
        source = f.read()
    
    # UNIFIED PIPELINE: Prefer Lark AST
    if use_lark and LARK_AVAILABLE and IfaLarkParser:
        try:
            lark_parser = IfaLarkParser()
            ast = lark_parser.parse(source)
            interpreter = IfaInterpreter(verbose=verbose)
            return run_ast(interpreter, ast)
        except Exception as e:
            if verbose:
                print(f"[Lark] Parse error, falling back to SimpleParser: {e}")
    
    # FALLBACK: Simple regex parser
    parser = SimpleParser()
    instructions = parser.parse(source)
    
    interpreter = IfaInterpreter(verbose=verbose)
    interpreter.execute(instructions)
    
    return interpreter


def run_ast(interpreter: IfaInterpreter, ast) -> IfaInterpreter:
    """Execute an AST tree using the interpreter.
    
    This is the new unified execution path.
    """
    if hasattr(ast, 'statements'):
        for stmt in ast.statements:
            execute_stmt(interpreter, stmt)
    return interpreter


def execute_stmt(interpreter: IfaInterpreter, stmt):
    """Execute a single AST statement node."""
    node_type = type(stmt).__name__
    
    if node_type == "VarDecl":
        # Variable declaration
        value = evaluate_expr(interpreter, stmt.value)
        interpreter.memory[stmt.name] = value
        if interpreter.verbose:
            type_info = f" (typed: {stmt.type_hint})" if hasattr(stmt, 'type_hint') and stmt.type_hint else ""
            print(f"[Ogbè] Set {stmt.name} = {value}{type_info}")
    
    elif node_type == "Instruction":
        execute_stmt(interpreter, stmt.call)
    
    elif node_type == "OduCall":
        # Method call
        key = f"{stmt.odu.lower()}.{stmt.ese.lower()}"
        args = ", ".join(str(evaluate_expr(interpreter, a)) for a in stmt.args)
        interpreter._dispatch(key, args)
    
    elif node_type == "IfStmt":
        cond = evaluate_expr(interpreter, stmt.condition)
        if cond:
            for s in stmt.then_body:
                execute_stmt(interpreter, s)
        elif hasattr(stmt, 'else_body') and stmt.else_body:
            for s in stmt.else_body:
                execute_stmt(interpreter, s)
    
    elif node_type == "WhileStmt":
        while evaluate_expr(interpreter, stmt.condition):
            for s in stmt.body:
                execute_stmt(interpreter, s)
    
    elif node_type == "ForStmt":
        iterable = evaluate_expr(interpreter, stmt.iterable)
        if hasattr(iterable, '__iter__'):
            for item in iterable:
                interpreter.memory[stmt.var_name] = item
                for s in stmt.body:
                    execute_stmt(interpreter, s)
    
    elif node_type == "TryStmt":
        try:
            for s in stmt.try_body:
                execute_stmt(interpreter, s)
        except Exception as e:
            interpreter.memory[stmt.error_var] = str(e)
            for s in stmt.catch_body:
                execute_stmt(interpreter, s)
    
    elif node_type == "ReturnStmt":
        interpreter.last_result = evaluate_expr(interpreter, stmt.value) if stmt.value else None
    
    elif node_type == "EndStmt":
        pass  # Program end marker
    
    elif node_type == "AssignmentStmt":
        value = evaluate_expr(interpreter, stmt.value)
        if isinstance(stmt.target, str):
            interpreter.memory[stmt.target] = value
        elif hasattr(stmt.target, 'name'):
            interpreter.memory[stmt.target.name] = value


def evaluate_expr(interpreter: IfaInterpreter, expr) -> any:
    """Evaluate an expression node to a Python value."""
    if expr is None:
        return None
    
    node_type = type(expr).__name__
    
    if node_type == "Literal":
        return expr.value
    
    elif node_type == "Identifier":
        return interpreter.memory.get(expr.name, None)
    
    elif node_type == "BinaryOp":
        left = evaluate_expr(interpreter, expr.left)
        right = evaluate_expr(interpreter, expr.right)
        
        op = expr.op
        if op == '+': return left + right
        elif op == '-': return left - right
        elif op == '*': return left * right
        elif op == '/': return left / right if right != 0 else 0
        elif op == '%': return left % right if right != 0 else 0
        elif op == '==': return left == right
        elif op == '!=': return left != right
        elif op == '<': return left < right
        elif op == '>': return left > right
        elif op == '<=': return left <= right
        elif op == '>=': return left >= right
        elif op in ('and', 'ati'): return left and right
        elif op in ('or', 'tabi'): return left or right
    
    elif node_type == "UnaryOp":
        operand = evaluate_expr(interpreter, expr.operand)
        op = expr.op
        if op == '-': return -operand
        elif op in ('not', 'kii'): return not operand
    
    elif node_type == "OduCall":
        key = f"{expr.odu.lower()}.{expr.ese.lower()}"
        args = ", ".join(str(evaluate_expr(interpreter, a)) for a in expr.args)
        return interpreter._dispatch(key, args)
    
    elif node_type == "ListLiteral":
        return [evaluate_expr(interpreter, e) for e in expr.elements]
    
    elif node_type == "MapLiteral":
        return {k: evaluate_expr(interpreter, v) for k, v in expr.entries.items()}
    
    elif node_type == "IndexAccess":
        target = interpreter.memory.get(expr.target, [])
        index = evaluate_expr(interpreter, expr.index)
        if isinstance(target, list) and isinstance(index, int):
            return target[index] if 0 <= index < len(target) else None
        elif isinstance(target, dict):
            return target.get(str(index), None)
    
    # Fallback for primitives
    if isinstance(expr, (int, float, str, bool)):
        return expr
    
    return expr


def run_code(source: str, verbose: bool = False):
    """Run Ifá code string directly."""
    parser = SimpleParser()
    instructions = parser.parse(source)
    
    interpreter = IfaInterpreter(verbose=verbose)
    interpreter.execute(instructions)
    
    return interpreter


# =============================================================================
# 2026 CEN MODEL - DÍFÁ (DIVINATION BLOCK)
# =============================================================================

class DivinationBlock:
    """
    Maps numeric values to Odù patterns using formal thresholds.
    
    Usage:
        div = DivinationBlock()
        odu = div.evaluate(0.75)  # Returns "Ọ̀sá" for values 0.5-0.75
    """
    
    THRESHOLDS = [
        (0.0, 0.125, "Ogbè", "OGBE"),
        (0.125, 0.25, "Ọ̀yẹ̀kú", "OYEKU"),
        (0.25, 0.375, "Ìwòrì", "IWORI"),
        (0.375, 0.5, "Ìrẹtẹ̀", "IRETE"),
        (0.5, 0.625, "Ọ̀sá", "OSA"),
        (0.625, 0.75, "Ògúndá", "OGUNDA"),
        (0.75, 0.875, "Ìká", "IKA"),
        (0.875, 1.0, "Òfún", "OFUN"),
    ]
    
    def evaluate(self, value: float) -> str:
        """Map a float (0-1) to an Odù pattern."""
        for low, high, yoruba, _ in self.THRESHOLDS:
            if low <= value < high:
                return yoruba
        return self.THRESHOLDS[-1][2]
    
    def get_domain(self, value: float) -> str:
        """Get the domain code (uppercase ASCII)."""
        for low, high, _, domain in self.THRESHOLDS:
            if low <= value < high:
                return domain
        return self.THRESHOLDS[-1][3]
    
    def branch(self, value: float, branches: dict) -> any:
        """Execute the branch corresponding to the Odù threshold."""
        domain = self.get_domain(value).upper()
        if domain in branches:
            return branches[domain]() if callable(branches[domain]) else branches[domain]
        if "DEFAULT" in branches:
            return branches["DEFAULT"]() if callable(branches["DEFAULT"]) else branches["DEFAULT"]
        return None


# =============================================================================
# DEMO
# =============================================================================

if __name__ == "__main__":
    print("""
╔══════════════════════════════════════════════════════════════╗
║              IFÁ INTERPRETER DEMO                            ║
╠══════════════════════════════════════════════════════════════╣
║  Like Python - Instant Execution                             ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    # Test code
    test_code = '''
# Hello World in Ifá
Irosu.fo("Ẹ kú àbọ̀ sí Ifá-Lang!");

# Math operations
Ogbe.bi(50);
Obara.ro(10);
Irosu.fo("After adding 10:");

# Random number
Owonrin.yan(1, 100);
Irosu.fo("Random number generated");
'''
    
    print("Running test code...")
    print("-" * 50)
    run_code(test_code, verbose=True)
