# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║            IFÁ 12-BIT ARCHITECTURE - THE 4,096 CALABASH                      ║
║                    Memory Map + Standard Library Index                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  256 Odù (8-bit)  = CPU Instructions (Fast/Simple)                           ║
║  4,096 Odù (12-bit) = Memory Addresses + Library Functions                   ║
║                                                                              ║
║  The Three-Legged Architecture:                                              ║
║    Leg 1 (Right): OpCode (The Action)                                        ║
║    Leg 2 (Middle): Target (The Object)                                       ║
║    Leg 3 (Left): Chapter (The Detail/Parameter)                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""


# =============================================================================
# THE 16 PRINCIPAL ODÙ - The Foundation
# =============================================================================
ODU_NAMES = [
    "OGBE",      # 0  - 0000
    "OYEKU",     # 1  - 0001  
    "IWORI",     # 2  - 0010
    "ODI",       # 3  - 0011
    "IROSU",     # 4  - 0100
    "OWONRIN",   # 5  - 0101
    "OBARA",     # 6  - 0110
    "OKANRAN",   # 7  - 0111
    "OGUNDA",    # 8  - 1000
    "OSA",       # 9  - 1001
    "IKA",       # 10 - 1010
    "OTURUPON",  # 11 - 1011
    "OTURA",     # 12 - 1100
    "IRETE",     # 13 - 1101
    "OSE",       # 14 - 1110
    "OFUN",      # 15 - 1111
]

NAME_TO_INDEX = {name: i for i, name in enumerate(ODU_NAMES)}
INDEX_TO_NAME = {i: name for i, name in enumerate(ODU_NAMES)}


# =============================================================================
# 12-BIT ODÙ ENCODER/DECODER
# =============================================================================
class Odu12Bit:
    """
    Encodes/decodes 12-bit (3-legged) Odù addresses.
    Range: 0 to 4095 (0x000 to 0xFFF)
    """
    
    @staticmethod
    def encode(leg1, leg2, leg3) -> int:
        """
        Encode three Odù legs into a 12-bit address.
        Each leg is 4 bits (0-15).
        """
        if isinstance(leg1, str):
            leg1 = NAME_TO_INDEX.get(leg1.upper(), 0)
        if isinstance(leg2, str):
            leg2 = NAME_TO_INDEX.get(leg2.upper(), 0)
        if isinstance(leg3, str):
            leg3 = NAME_TO_INDEX.get(leg3.upper(), 0)
        
        return (leg1 << 8) | (leg2 << 4) | leg3
    
    @staticmethod
    def decode(address: int):
        """
        Decode a 12-bit address into three Odù legs.
        Returns: (leg1_name, leg2_name, leg3_name)
        """
        address = address & 0xFFF
        leg1 = (address >> 8) & 0xF
        leg2 = (address >> 4) & 0xF
        leg3 = address & 0xF
        
        return (
            INDEX_TO_NAME.get(leg1, "?"),
            INDEX_TO_NAME.get(leg2, "?"),
            INDEX_TO_NAME.get(leg3, "?")
        )
    
    @staticmethod
    def to_string(address: int) -> str:
        """Convert address to human-readable Odù string."""
        leg1, leg2, leg3 = Odu12Bit.decode(address)
        return f"{leg1}-{leg2}-{leg3}"
    
    @staticmethod
    def from_string(odu_string: str) -> int:
        """Parse 'OGBE-OYEKU-IWORI' format into address."""
        parts = odu_string.upper().replace("_", "-").split("-")
        if len(parts) != 3:
            raise ValueError(f"Expected 3 legs, got {len(parts)}")
        return Odu12Bit.encode(parts[0], parts[1], parts[2])


# =============================================================================
# THE 4KB CALABASH (MEMORY MAP)
# =============================================================================
class Calabash4K:
    """
    4KB Memory using 12-bit Odù addressing.
    Each memory location is 1 byte (0-255).
    """
    
    # Memory Regions
    REGIONS = {
        (0x000, 0x0FF): "BOOT_SECTOR",      # Ogbè-*-*: System Bootstrap
        (0x100, 0x1FF): "STACK",            # Oyeku-*-*: The Call Stack
        (0x200, 0x2FF): "HEAP",             # Iwori-*-*: Dynamic Memory
        (0x300, 0x3FF): "STATIC",           # Odi-*-*: Static Variables
        (0x400, 0x4FF): "IO_BUFFER",        # Irosu-*-*: I/O Buffers
        (0x500, 0x5FF): "NETWORK",          # Owonrin-*-*: Network Stack
        (0x600, 0x6FF): "GRAPHICS",         # Obara-*-*: Video Memory
        (0x700, 0x7FF): "AUDIO",            # Okanran-*-*: Audio Buffer
        (0x800, 0x8FF): "FILESYSTEM",       # Ogunda-*-*: File Handles
        (0x900, 0x9FF): "DATABASE",         # Osa-*-*: DB Connections
        (0xA00, 0xAFF): "CRYPTO",           # Ika-*-*: Encryption Keys
        (0xB00, 0xBFF): "MATH",             # Oturupon-*-*: Math Registers
        (0xC00, 0xCFF): "OBJECTS",          # Otura-*-*: Object Heap
        (0xD00, 0xDFF): "GARBAGE",          # Irete-*-*: GC Region
        (0xE00, 0xEFF): "DEBUG",            # Ose-*-*: Debug Info
        (0xF00, 0xFFF): "RESERVED",         # Ofun-*-*: System Reserved
    }
    
    def __init__(self):
        self.memory = [0] * 4096
        self.access_log = []
    
    def read(self, address) -> int:
        """Read byte from Odù address."""
        if isinstance(address, str):
            address = Odu12Bit.from_string(address)
        original = address
        address = address & 0xFFF
        if original != address:
            print(f"  ⚠️ Address 0x{original:X} wrapped to 0x{address:03X} (4KB limit)")
        self.access_log.append(("READ", address))
        return self.memory[address]
    
    def write(self, address, value: int, strict: bool = False):
        """Write byte to Odù address.
        
        Args:
            address: 12-bit address (0-4095)
            value: Byte value (0-255)
            strict: If True, raise exception on out-of-bounds. If False, wrap values.
        """
        if isinstance(address, str):
            address = Odu12Bit.from_string(address)
        original_addr = address
        
        # Security: Strict mode raises exceptions
        if strict:
            if not (0 <= address < 4096):
                raise IndexError(f"Address {address} out of bounds (0-4095)")
            if not (0 <= value <= 255):
                raise ValueError(f"Value {value} out of byte range (0-255)")
        else:
            address = address & 0xFFF
            if original_addr != address:
                print(f"  ⚠️ Address 0x{original_addr:X} wrapped to 0x{address:03X} (4KB limit)")
            if value < 0 or value > 255:
                print(f"  ⚠️ Value {value} truncated to {value & 0xFF} (byte range)")
        
        self.access_log.append(("WRITE", address, value))
        self.memory[address] = value & 0xFF
    
    def get_region(self, address) -> str:
        """Get the memory region name for an address."""
        if isinstance(address, str):
            address = Odu12Bit.from_string(address)
        for (start, end), name in self.REGIONS.items():
            if start <= address <= end:
                return name
        return "UNKNOWN"
    
    def dump(self, start: int = 0, length: int = 16):
        """Dump memory with Odù addresses."""
        print(f"\n{'ADDRESS':<20} {'ODÙ':<25} {'HEX':<6} {'DEC':<4} {'REGION'}")
        print("-" * 70)
        for i in range(start, min(start + length, 4096)):
            odu = Odu12Bit.to_string(i)
            region = self.get_region(i)
            val = self.memory[i]
            print(f"0x{i:03X}               {odu:<25} 0x{val:02X}   {val:<4} {region}")
    
    def clear(self):
        """Clear all memory."""
        self.memory = [0] * 4096
        self.access_log.clear()


# =============================================================================
# THE GREAT LIBRARY (STANDARD LIBRARY INDEX)
# =============================================================================
class IfaStandardLibrary:
    """
    4,096 library functions indexed by 12-bit Odù.
    Maps 3-legged Odù to function definitions.
    """
    
    def __init__(self):
        self.functions = {}
        self._register_stdlib()
    
    def _register_stdlib(self):
        """Register the standard library functions."""
        
        # === SYSTEM (OGBE-*-*) ===
        self.register("OGBE-OGBE-OGBE", "sys.boot", "Total system reset")
        self.register("OGBE-OGBE-OYEKU", "sys.halt", "Graceful shutdown")
        self.register("OGBE-OYEKU-OGBE", "sys.sleep", "Pause execution")
        self.register("OGBE-IWORI-OGBE", "sys.time", "Get current time")
        self.register("OGBE-ODI-OGBE", "sys.version", "Get version info")
        
        # === STACK (OYEKU-*-*) ===
        self.register("OYEKU-OGBE-OGBE", "stack.push", "Push to stack")
        self.register("OYEKU-OYEKU-OGBE", "stack.pop", "Pop from stack")
        self.register("OYEKU-IWORI-OGBE", "stack.peek", "Peek top of stack")
        self.register("OYEKU-ODI-OGBE", "stack.clear", "Clear stack")
        
        # === MEMORY (IWORI-*-*) ===
        self.register("IWORI-OGBE-OGBE", "mem.alloc", "Allocate memory")
        self.register("IWORI-OYEKU-OGBE", "mem.free", "Free memory")
        self.register("IWORI-IWORI-OGBE", "mem.copy", "Copy memory block")
        self.register("IWORI-ODI-OGBE", "mem.fill", "Fill memory region")
        
        # === FILE I/O (ODI-*-*) ===
        self.register("ODI-OGBE-OGBE", "file.open", "Open file")
        self.register("ODI-OGBE-IROSU", "file.create", "Create new file")
        self.register("ODI-OYEKU-OGBE", "file.close", "Close file")
        self.register("ODI-IWORI-OGBE", "file.read", "Read from file")
        self.register("ODI-ODI-OGBE", "file.write", "Write to file")
        self.register("ODI-IROSU-OGBE", "file.seek", "Seek position")
        self.register("ODI-OWONRIN-OGBE", "file.delete", "Delete file")
        
        # === CONSOLE (IROSU-*-*) ===
        self.register("IROSU-OGBE-OGBE", "io.print", "Print to console")
        self.register("IROSU-OYEKU-OGBE", "io.input", "Read from console")
        self.register("IROSU-IWORI-OGBE", "io.clear", "Clear screen")
        self.register("IROSU-ODI-OGBE", "io.color", "Set text color")
        
        # === NETWORK (OTURA-*-*) ===
        self.register("OTURA-OGBE-OGBE", "net.connect", "Establish connection")
        self.register("OTURA-OYEKU-OGBE", "net.disconnect", "Close connection")
        self.register("OTURA-IWORI-OGBE", "net.send", "Send data")
        self.register("OTURA-ODI-OGBE", "net.recv", "Receive data")
        self.register("OTURA-IKA-OFUN", "net.encrypt", "Encrypt packet")
        
        # === DATABASE (OSA-*-*) ===
        self.register("OSA-OGBE-OGBE", "db.connect", "Connect to database")
        self.register("OSA-OYEKU-OGBE", "db.disconnect", "Close connection")
        self.register("OSA-IWORI-OGBE", "db.query", "Execute query")
        self.register("OSA-ODI-OGBE", "db.insert", "Insert record")
        
        # === GRAPHICS (OSE-*-*) ===
        self.register("OSE-OGBE-OGBE", "gfx.init", "Initialize graphics")
        self.register("OSE-OYEKU-OGBE", "gfx.clear", "Clear screen")
        self.register("OSE-IWORI-OGBE", "gfx.draw", "Draw primitive")
        self.register("OSE-ODI-OGBE", "gfx.render", "Render frame")
        self.register("OSE-IROSU-OGBE", "gfx.text", "Draw text")
        
        # === CRYPTO (IKA-*-*) ===
        self.register("IKA-OGBE-OGBE", "crypto.hash", "Hash data")
        self.register("IKA-OYEKU-OGBE", "crypto.encrypt", "Encrypt data")
        self.register("IKA-IWORI-OGBE", "crypto.decrypt", "Decrypt data")
        self.register("IKA-ODI-OGBE", "crypto.sign", "Sign message")
        
        # === MATH (OTURUPON-*-*) ===
        self.register("OTURUPON-OGBE-OGBE", "math.add", "Addition")
        self.register("OTURUPON-OYEKU-OGBE", "math.sub", "Subtraction")
        self.register("OTURUPON-IWORI-OGBE", "math.mul", "Multiplication")
        self.register("OTURUPON-ODI-OGBE", "math.div", "Division")
        self.register("OTURUPON-IROSU-OGBE", "math.mod", "Modulo")
        self.register("OTURUPON-OWONRIN-OGBE", "math.pow", "Power")
        self.register("OTURUPON-OBARA-OGBE", "math.sqrt", "Square root")
        
        # === OBJECTS (OFUN-*-*) ===
        self.register("OFUN-OGBE-OGBE", "obj.new", "Create object")
        self.register("OFUN-OYEKU-OGBE", "obj.delete", "Delete object")
        self.register("OFUN-IWORI-OGBE", "obj.clone", "Clone object")
        self.register("OFUN-ODI-OGBE", "obj.get", "Get property")
        self.register("OFUN-IROSU-OGBE", "obj.set", "Set property")
    
    def register(self, odu_key: str, func_name: str, description: str):
        """Register a library function."""
        address = Odu12Bit.from_string(odu_key)
        self.functions[address] = {
            "odu": odu_key,
            "name": func_name,
            "desc": description,
            "address": address
        }
    
    def lookup(self, key):
        """Look up a function by Odù or address."""
        if isinstance(key, str):
            if "-" in key:
                key = Odu12Bit.from_string(key)
            else:
                for addr, func in self.functions.items():
                    if func["name"] == key:
                        return func
                return None
        return self.functions.get(key)
    
    def list_by_category(self, prefix: str):
        """List all functions in a category (first leg)."""
        results = []
        prefix_idx = NAME_TO_INDEX.get(prefix.upper(), -1)
        for addr, func in self.functions.items():
            if (addr >> 8) == prefix_idx:
                results.append(func)
        return results
    
    def print_library(self):
        """Print the entire library."""
        print("\n" + "=" * 70)
        print("THE GREAT LIBRARY OF IFÁ (STANDARD LIBRARY)")
        print("=" * 70)
        print(f"{'ADDRESS':<8} {'ODÙ':<25} {'FUNCTION':<20} DESCRIPTION")
        print("-" * 70)
        
        for addr in sorted(self.functions.keys()):
            func = self.functions[addr]
            print(f"0x{addr:03X}    {func['odu']:<25} {func['name']:<20} {func['desc']}")



# =============================================================================
# 2026 CEN MODEL - DYNAMIC ỌPỌN (SCALABLE MEMORY)
# =============================================================================

from enum import Enum

class OponSize(Enum):
    """Memory size options for Ọpọ́n workspace."""
    KEKERE = 4 * 1024       # 4KB - IoT/Embedded
    GIDI = 16 * 1024        # 16KB - Standard
    NLA = 64 * 1024         # 64KB - Large datasets
    MEGA = 1024 * 1024      # 1MB - Maximum


class OponCalabash:
    """
    Dynamic Ọpọ́n (Calabash) - Scalable memory workspace.
    Sizes range from 4KB to 1MB based on application needs.
    """
    
    def __init__(self, size: OponSize = OponSize.GIDI):
        self.size = size
        self.total_bytes = size.value
        self.memory = bytearray(self.total_bytes)
        self._regions = {}
    
    def write(self, address: int, value: int):
        """Write a byte to memory."""
        if 0 <= address < self.total_bytes:
            self.memory[address] = value & 0xFF
        else:
            raise IndexError(f"Address {address} out of bounds (max: {self.total_bytes-1})")
    
    def read(self, address: int) -> int:
        """Read a byte from memory."""
        if 0 <= address < self.total_bytes:
            return self.memory[address]
        raise IndexError(f"Address {address} out of bounds")
    
    def allocate_region(self, name: str, size: int) -> int:
        """Allocate a named region. Returns start address."""
        used = sum(r['size'] for r in self._regions.values())
        if used + size > self.total_bytes:
            raise MemoryError(f"Cannot allocate {size} bytes, only {self.total_bytes - used} free")
        start = used
        self._regions[name] = {'start': start, 'size': size}
        return start
    
    def get_region(self, name: str) -> tuple:
        """Get region (start, size)."""
        r = self._regions.get(name)
        return (r['start'], r['size']) if r else None
    
    def clear(self):
        """Zero all memory."""
        self.memory = bytearray(self.total_bytes)
        self._regions = {}
    
    def __repr__(self):
        size_name = self.size.name.title()
        return f"OponCalabash(Ọpọ́n {size_name}, {self.total_bytes:,} bytes)"


# =============================================================================
# EXPORTS
# =============================================================================
__all__ = [
    'ODU_NAMES',
    'NAME_TO_INDEX',
    'INDEX_TO_NAME',
    'Odu12Bit',
    'Calabash4K',
    'IfaStandardLibrary',
    'OponSize',
    'OponCalabash',
]

