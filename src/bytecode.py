# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ BYTECODE FORMAT (.ifab)                               ║
║                    Binary Format for Fast Loading                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Instead of interpreting text, compile to binary IFAB format.                ║
║  A 2KB text file becomes a 200-byte .ifab file.                              ║
║  Critical for "Smart Dust" / IoT embedded devices.                           ║
╚══════════════════════════════════════════════════════════════════════════════╝

File Format:
┌─────────────────────────────────────────────────────────────────┐
│ Magic Number (4 bytes): "IFA1"                                  │
│ Version (1 byte): 0x01                                          │
│ Constants Count (2 bytes): Little-endian                        │
│ Code Size (4 bytes): Little-endian                              │
├─────────────────────────────────────────────────────────────────┤
│ Constants Pool:                                                 │
│   Type (1 byte) + Length (2 bytes) + Data                       │
├─────────────────────────────────────────────────────────────────┤
│ Bytecode:                                                       │
│   OpCode (1 byte) + Operands (variable)                         │
└─────────────────────────────────────────────────────────────────┘
"""

import struct
import os
from typing import Any, Dict, List, Tuple, Optional, BinaryIO
from dataclasses import dataclass, field
from enum import IntEnum


# =============================================================================
# OPCODE DEFINITIONS - 8-bit Instruction Set Architecture
# =============================================================================

class OpCode(IntEnum):
    """
    8-bit opcodes based on Odù logic.
    The high nibble indicates the category, low nibble the operation.
    """
    # === 0x0X: Stack Operations ===
    NOP         = 0x00  # No operation
    LOAD_CONST  = 0x01  # Push constant onto stack
    LOAD_VAR    = 0x02  # Push variable value onto stack
    STORE_VAR   = 0x03  # Pop and store to variable
    DUP         = 0x04  # Duplicate top of stack
    POP         = 0x05  # Pop and discard
    SWAP        = 0x06  # Swap top two stack items
    
    # === 0x1X: Call Operations ===
    CALL_ODU    = 0x10  # Call standard library Odù
    CALL_ESE    = 0x11  # Call method on Odù
    CALL_FUNC   = 0x12  # Call user-defined function
    RETURN      = 0x13  # Return from function
    
    # === 0x2X: Control Flow (Dídá) ===
    JUMP        = 0x20  # Unconditional jump
    JUMP_IF     = 0x21  # Jump if true
    JUMP_NOT    = 0x22  # Jump if false
    LOOP        = 0x23  # Loop back
    
    # === 0x3X: Arithmetic (Ọ̀bàrà / Òtúúrúpọ̀n) ===
    ADD         = 0x30  # Addition
    SUB         = 0x31  # Subtraction
    MUL         = 0x32  # Multiplication
    DIV         = 0x33  # Division
    MOD         = 0x34  # Modulo
    NEG         = 0x35  # Negate
    
    # === 0x4X: Comparison ===
    EQ          = 0x40  # Equal
    NE          = 0x41  # Not equal
    LT          = 0x42  # Less than
    LE          = 0x43  # Less or equal
    GT          = 0x44  # Greater than
    GE          = 0x45  # Greater or equal
    
    # === 0x5X: Logic ===
    AND         = 0x50  # Logical AND
    OR          = 0x51  # Logical OR
    NOT         = 0x52  # Logical NOT
    
    # === 0x6X: String (Ìká) ===
    CONCAT      = 0x60  # Concatenate strings
    STRLEN      = 0x61  # String length
    SUBSTR      = 0x62  # Substring
    
    # === 0x7X: Array (Ògúndá) ===
    NEW_ARRAY   = 0x70  # Create array
    ARRAY_GET   = 0x71  # Get array element
    ARRAY_SET   = 0x72  # Set array element
    ARRAY_LEN   = 0x73  # Array length
    ARRAY_PUSH  = 0x74  # Push to array
    
    # === 0x8X: Object (Òfún) ===
    NEW_OBJ     = 0x80  # Create object
    GET_FIELD   = 0x81  # Get object field
    SET_FIELD   = 0x82  # Set object field
    
    # === 0xFX: System ===
    HALT        = 0xFF  # Stop execution (Ọ̀yẹ̀kú)
    DEBUG       = 0xFE  # Debug breakpoint
    PRINT       = 0xFD  # Print top of stack


# =============================================================================
# CONSTANT TYPES
# =============================================================================

class ConstType(IntEnum):
    """Type tags for constants pool."""
    NULL    = 0x00
    INT     = 0x01  # 4-byte signed integer
    FLOAT   = 0x02  # 8-byte double
    STRING  = 0x03  # Length-prefixed UTF-8 string
    BOOL    = 0x04  # 1 byte (0 or 1)
    NAME    = 0x05  # Symbol/identifier name


# =============================================================================
# ODÙ INDEX - Maps Odù names to indices
# =============================================================================

ODU_INDEX = {
    "ogbe": 0, "oyeku": 1, "iwori": 2, "odi": 3,
    "irosu": 4, "owonrin": 5, "obara": 6, "okanran": 7,
    "ogunda": 8, "osa": 9, "ika": 10, "oturupon": 11,
    "otura": 12, "irete": 13, "ose": 14, "ofun": 15,
}


# =============================================================================
# BYTECODE CHUNK - Compiled unit
# =============================================================================

@dataclass
class BytecodeChunk:
    """A chunk of compiled bytecode."""
    magic: bytes = b"IFA1"
    version: int = 1
    constants: List[Tuple[int, Any]] = field(default_factory=list)
    code: bytearray = field(default_factory=bytearray)
    line_info: List[int] = field(default_factory=list)  # Maps bytecode offset to source line
    
    def add_constant(self, value: Any) -> int:
        """Add a constant and return its index."""
        # Check if already exists
        for i, (t, v) in enumerate(self.constants):
            if v == value:
                return i
        
        # Determine type
        if value is None:
            const_type = ConstType.NULL
        elif isinstance(value, bool):
            const_type = ConstType.BOOL
        elif isinstance(value, int):
            const_type = ConstType.INT
        elif isinstance(value, float):
            const_type = ConstType.FLOAT
        elif isinstance(value, str):
            const_type = ConstType.STRING
        else:
            const_type = ConstType.NAME
            value = str(value)
        
        idx = len(self.constants)
        self.constants.append((const_type, value))
        return idx
    
    def emit(self, opcode: int, *operands: int, line: int = 0):
        """Emit an instruction."""
        self.code.append(opcode & 0xFF)
        self.line_info.append(line)
        for op in operands:
            self.code.append(op & 0xFF)
            self.line_info.append(line)
    
    def emit_u16(self, opcode: int, value: int, line: int = 0):
        """Emit instruction with 16-bit operand."""
        self.code.append(opcode & 0xFF)
        self.code.append(value & 0xFF)
        self.code.append((value >> 8) & 0xFF)
        for _ in range(3):
            self.line_info.append(line)


# =============================================================================
# BYTECODE COMPILER
# =============================================================================

class BytecodeCompiler:
    """
    Compiles Ifá AST to bytecode.
    """
    
    def __init__(self):
        self.chunk = BytecodeChunk()
        self.variables: Dict[str, int] = {}
        self.functions: Dict[str, int] = {}
    
    def compile(self, ast) -> BytecodeChunk:
        """Compile AST to bytecode chunk."""
        self.chunk = BytecodeChunk()
        self._compile_node(ast)
        self.chunk.emit(OpCode.HALT)
        return self.chunk
    
    def _compile_node(self, node):
        """Compile a single AST node."""
        node_type = type(node).__name__
        method = getattr(self, f'_compile_{node_type.lower()}', None)
        if method:
            method(node)
        elif hasattr(node, 'statements'):
            for stmt in node.statements:
                self._compile_node(stmt)
    
    def _compile_program(self, node):
        for stmt in node.statements:
            self._compile_node(stmt)
    
    def _compile_vardecl(self, node):
        """Compile variable declaration."""
        # Compile the value
        self._compile_node(node.value)
        # Store to variable
        var_idx = self.chunk.add_constant(node.name)
        self.chunk.emit(OpCode.STORE_VAR, var_idx)
        self.variables[node.name] = var_idx
    
    def _compile_instruction(self, node):
        """Compile an instruction (odu call statement)."""
        self._compile_node(node.call)
        self.chunk.emit(OpCode.POP)  # Discard result
    
    def _compile_oducall(self, node):
        """Compile Odù method call."""
        # Push arguments in reverse order
        for arg in reversed(node.args):
            self._compile_node(arg)
        
        # Push arg count
        self.chunk.emit(OpCode.LOAD_CONST, self.chunk.add_constant(len(node.args)))
        
        # Call the Odù
        odu_idx = ODU_INDEX.get(node.odu.lower(), 0)
        ese_idx = self.chunk.add_constant(node.ese)
        
        self.chunk.emit(OpCode.CALL_ODU, odu_idx)
        self.chunk.emit(OpCode.CALL_ESE, ese_idx)
    
    def _compile_literal(self, node):
        """Compile a literal value."""
        idx = self.chunk.add_constant(node.value)
        self.chunk.emit(OpCode.LOAD_CONST, idx)
    
    def _compile_identifier(self, node):
        """Compile variable reference."""
        var_idx = self.chunk.add_constant(node.name)
        self.chunk.emit(OpCode.LOAD_VAR, var_idx)
    
    def _compile_binaryop(self, node):
        """Compile binary operation."""
        self._compile_node(node.left)
        self._compile_node(node.right)
        
        op_map = {
            '+': OpCode.ADD, '-': OpCode.SUB,
            '*': OpCode.MUL, '/': OpCode.DIV, '%': OpCode.MOD,
            '==': OpCode.EQ, '!=': OpCode.NE,
            '<': OpCode.LT, '<=': OpCode.LE,
            '>': OpCode.GT, '>=': OpCode.GE,
            '&&': OpCode.AND, '||': OpCode.OR,
        }
        opcode = op_map.get(node.op, OpCode.NOP)
        self.chunk.emit(opcode)
    
    def _compile_unaryop(self, node):
        """Compile unary operation."""
        self._compile_node(node.operand)
        
        if node.op == '-':
            self.chunk.emit(OpCode.NEG)
        elif node.op == '!':
            self.chunk.emit(OpCode.NOT)
    
    def _compile_ifstmt(self, node):
        """Compile if statement."""
        # Compile condition
        self._compile_node(node.condition)
        
        # Jump to else if false
        jump_else = len(self.chunk.code)
        self.chunk.emit_u16(OpCode.JUMP_NOT, 0)  # Placeholder
        
        # Compile then branch
        for stmt in node.then_body:
            self._compile_node(stmt)
        
        # Jump over else
        jump_end = len(self.chunk.code)
        self.chunk.emit_u16(OpCode.JUMP, 0)  # Placeholder
        
        # Patch else jump
        else_addr = len(self.chunk.code)
        self.chunk.code[jump_else + 1] = else_addr & 0xFF
        self.chunk.code[jump_else + 2] = (else_addr >> 8) & 0xFF
        
        # Compile else branch
        for stmt in node.else_body:
            self._compile_node(stmt)
        
        # Patch end jump
        end_addr = len(self.chunk.code)
        self.chunk.code[jump_end + 1] = end_addr & 0xFF
        self.chunk.code[jump_end + 2] = (end_addr >> 8) & 0xFF
    
    def _compile_whilestmt(self, node):
        """Compile while loop."""
        loop_start = len(self.chunk.code)
        
        # Compile condition
        self._compile_node(node.condition)
        
        # Jump to end if false
        jump_end = len(self.chunk.code)
        self.chunk.emit_u16(OpCode.JUMP_NOT, 0)  # Placeholder
        
        # Compile body
        for stmt in node.body:
            self._compile_node(stmt)
        
        # Jump back to loop
        self.chunk.emit_u16(OpCode.JUMP, loop_start)
        
        # Patch end jump
        end_addr = len(self.chunk.code)
        self.chunk.code[jump_end + 1] = end_addr & 0xFF
        self.chunk.code[jump_end + 2] = (end_addr >> 8) & 0xFF
    
    def _compile_returnstmt(self, node):
        """Compile return statement."""
        if node.value:
            self._compile_node(node.value)
        else:
            self.chunk.emit(OpCode.LOAD_CONST, self.chunk.add_constant(None))
        self.chunk.emit(OpCode.RETURN)
    
    def _compile_endstmt(self, node):
        """Compile end statement."""
        self.chunk.emit(OpCode.HALT)


# =============================================================================
# BYTECODE SERIALIZER
# =============================================================================

class BytecodeSerializer:
    """
    Serializes/deserializes bytecode to/from .ifab files.
    """
    
    @staticmethod
    def serialize(chunk: BytecodeChunk) -> bytes:
        """Serialize bytecode chunk to bytes."""
        output = bytearray()
        
        # Header
        output.extend(chunk.magic)              # 4 bytes: "IFA1"
        output.append(chunk.version)            # 1 byte: version
        
        # Constants count (2 bytes, little-endian)
        output.extend(struct.pack('<H', len(chunk.constants)))
        
        # Code size (4 bytes, little-endian)
        output.extend(struct.pack('<I', len(chunk.code)))
        
        # Constants pool
        for const_type, value in chunk.constants:
            output.append(const_type)
            
            if const_type == ConstType.NULL:
                pass  # No data
            elif const_type == ConstType.BOOL:
                output.append(1 if value else 0)
            elif const_type == ConstType.INT:
                output.extend(struct.pack('<i', value))
            elif const_type == ConstType.FLOAT:
                output.extend(struct.pack('<d', value))
            elif const_type in (ConstType.STRING, ConstType.NAME):
                encoded = value.encode('utf-8')
                output.extend(struct.pack('<H', len(encoded)))
                output.extend(encoded)
        
        # Bytecode
        output.extend(chunk.code)
        
        return bytes(output)
    
    @staticmethod
    def deserialize(data: bytes) -> BytecodeChunk:
        """Deserialize bytes to bytecode chunk."""
        chunk = BytecodeChunk()
        pos = 0
        
        # Header
        chunk.magic = data[pos:pos+4]
        pos += 4
        
        if chunk.magic != b"IFA1":
            raise ValueError(f"Invalid magic number: {chunk.magic}")
        
        chunk.version = data[pos]
        pos += 1
        
        # Constants count
        const_count = struct.unpack('<H', data[pos:pos+2])[0]
        pos += 2
        
        # Code size
        code_size = struct.unpack('<I', data[pos:pos+4])[0]
        pos += 4
        
        # Constants pool
        for _ in range(const_count):
            const_type = data[pos]
            pos += 1
            
            if const_type == ConstType.NULL:
                value = None
            elif const_type == ConstType.BOOL:
                value = data[pos] != 0
                pos += 1
            elif const_type == ConstType.INT:
                value = struct.unpack('<i', data[pos:pos+4])[0]
                pos += 4
            elif const_type == ConstType.FLOAT:
                value = struct.unpack('<d', data[pos:pos+8])[0]
                pos += 8
            elif const_type in (ConstType.STRING, ConstType.NAME):
                length = struct.unpack('<H', data[pos:pos+2])[0]
                pos += 2
                value = data[pos:pos+length].decode('utf-8')
                pos += length
            else:
                raise ValueError(f"Unknown constant type: {const_type}")
            
            chunk.constants.append((const_type, value))
        
        # Bytecode
        chunk.code = bytearray(data[pos:pos+code_size])
        
        return chunk
    
    @staticmethod
    def save(chunk: BytecodeChunk, filepath: str):
        """Save bytecode to .ifab file."""
        with open(filepath, 'wb') as f:
            f.write(BytecodeSerializer.serialize(chunk))
    
    @staticmethod
    def load(filepath: str) -> BytecodeChunk:
        """Load bytecode from .ifab file."""
        with open(filepath, 'rb') as f:
            return BytecodeSerializer.deserialize(f.read())


# =============================================================================
# BYTECODE VM
# =============================================================================

class BytecodeVM:
    """
    Stack-based virtual machine for .ifab bytecode.
    """
    
    def __init__(self):
        self.stack: List[Any] = []
        self.variables: Dict[str, Any] = {}
        self.pc: int = 0  # Program counter
        self.running: bool = False
        self.chunk: BytecodeChunk = None
    
    def run(self, chunk: BytecodeChunk, verbose: bool = False) -> Any:
        """Execute bytecode chunk."""
        self.chunk = chunk
        self.stack = []
        self.pc = 0
        self.running = True
        
        while self.running and self.pc < len(chunk.code):
            opcode = chunk.code[self.pc]
            self.pc += 1
            
            if verbose:
                self._debug_instruction(opcode)
            
            self._execute(opcode)
        
        return self.stack[-1] if self.stack else None
    
    def _read_byte(self) -> int:
        """Read next byte from code."""
        val = self.chunk.code[self.pc]
        self.pc += 1
        return val
    
    def _read_u16(self) -> int:
        """Read next 16-bit value."""
        lo = self.chunk.code[self.pc]
        hi = self.chunk.code[self.pc + 1]
        self.pc += 2
        return (hi << 8) | lo
    
    def _get_constant(self, idx: int) -> Any:
        """Get constant by index."""
        if idx < len(self.chunk.constants):
            return self.chunk.constants[idx][1]
        return None
    
    def _execute(self, opcode: int):
        """Execute a single opcode."""
        
        # === Stack Operations ===
        if opcode == OpCode.NOP:
            pass
        
        elif opcode == OpCode.LOAD_CONST:
            idx = self._read_byte()
            self.stack.append(self._get_constant(idx))
        
        elif opcode == OpCode.LOAD_VAR:
            idx = self._read_byte()
            name = self._get_constant(idx)
            self.stack.append(self.variables.get(name, None))
        
        elif opcode == OpCode.STORE_VAR:
            idx = self._read_byte()
            name = self._get_constant(idx)
            self.variables[name] = self.stack.pop() if self.stack else None
        
        elif opcode == OpCode.DUP:
            if self.stack:
                self.stack.append(self.stack[-1])
        
        elif opcode == OpCode.POP:
            if self.stack:
                self.stack.pop()
        
        elif opcode == OpCode.SWAP:
            if len(self.stack) >= 2:
                self.stack[-1], self.stack[-2] = self.stack[-2], self.stack[-1]
        
        # === Call Operations ===
        elif opcode == OpCode.CALL_ODU:
            odu_idx = self._read_byte()
            # Store for next CALL_ESE
            self._current_odu = odu_idx
        
        elif opcode == OpCode.CALL_ESE:
            ese_idx = self._read_byte()
            ese_name = self._get_constant(ese_idx)
            # Pop arg count
            arg_count = self.stack.pop() if self.stack else 0
            # Pop args
            args = [self.stack.pop() for _ in range(int(arg_count))][::-1]
            # Execute the call
            result = self._call_stdlib(self._current_odu, ese_name, args)
            self.stack.append(result)
        
        elif opcode == OpCode.RETURN:
            self.running = False  # For now, just halt
        
        # === Control Flow ===
        elif opcode == OpCode.JUMP:
            self.pc = self._read_u16()
        
        elif opcode == OpCode.JUMP_IF:
            addr = self._read_u16()
            if self.stack and self.stack.pop():
                self.pc = addr
        
        elif opcode == OpCode.JUMP_NOT:
            addr = self._read_u16()
            if not (self.stack and self.stack.pop()):
                self.pc = addr
        
        # === Arithmetic ===
        elif opcode == OpCode.ADD:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a + b)
        
        elif opcode == OpCode.SUB:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a - b)
        
        elif opcode == OpCode.MUL:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a * b)
        
        elif opcode == OpCode.DIV:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a // b if b != 0 else 0)
        
        elif opcode == OpCode.MOD:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a % b if b != 0 else 0)
        
        elif opcode == OpCode.NEG:
            self.stack.append(-self.stack.pop())
        
        # === Comparison ===
        elif opcode == OpCode.EQ:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a == b)
        
        elif opcode == OpCode.NE:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a != b)
        
        elif opcode == OpCode.LT:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a < b)
        
        elif opcode == OpCode.LE:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a <= b)
        
        elif opcode == OpCode.GT:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a > b)
        
        elif opcode == OpCode.GE:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a >= b)
        
        # === Logic ===
        elif opcode == OpCode.AND:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a and b)
        
        elif opcode == OpCode.OR:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a or b)
        
        elif opcode == OpCode.NOT:
            self.stack.append(not self.stack.pop())
        
        # === String ===
        elif opcode == OpCode.CONCAT:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(str(a) + str(b))
        
        elif opcode == OpCode.STRLEN:
            self.stack.append(len(str(self.stack.pop())))
        
        # === System ===
        elif opcode == OpCode.HALT:
            self.running = False
        
        elif opcode == OpCode.PRINT:
            print(f"[Out] {self.stack[-1] if self.stack else 'nil'}")
        
        elif opcode == OpCode.DEBUG:
            print(f"[Debug] PC={self.pc} Stack={self.stack}")
    
    def _call_stdlib(self, odu_idx: int, ese_name: str, args: List[Any]) -> Any:
        """Call standard library function."""
        odu_names = list(ODU_INDEX.keys())
        odu_name = odu_names[odu_idx] if odu_idx < len(odu_names) else "unknown"
        
        # Ìrosù (Output)
        if odu_name == "irosu":
            if ese_name in ("fo", "fọ̀"):
                print(f"[Ìrosù] {args[0] if args else ''}")
                return None
        
        # Ogbè (Init)
        if odu_name == "ogbe":
            if ese_name in ("bi", "bí"):
                return args[0] if args else 0
        
        # Ọ̀bàrà (Math)
        if odu_name == "obara":
            if ese_name in ("ro", "rò"):
                return sum(args)
        
        # Simulate other calls
        print(f"[{odu_name}.{ese_name}] Args: {args}")
        return None
    
    def _debug_instruction(self, opcode: int):
        """Print debug info for instruction."""
        name = OpCode(opcode).name if opcode in OpCode._value2member_map_ else f"0x{opcode:02X}"
        print(f"  [{self.pc-1:04d}] {name}")


# =============================================================================
# DISASSEMBLER
# =============================================================================

def disassemble(chunk: BytecodeChunk) -> str:
    """Disassemble bytecode to human-readable format."""
    lines = []
    lines.append("=== IFAB Disassembly ===")
    lines.append(f"Magic: {chunk.magic.decode()}")
    lines.append(f"Version: {chunk.version}")
    lines.append(f"Constants: {len(chunk.constants)}")
    lines.append(f"Code Size: {len(chunk.code)} bytes")
    lines.append("")
    
    lines.append("=== Constants ===")
    for i, (t, v) in enumerate(chunk.constants):
        type_name = ConstType(t).name
        lines.append(f"  [{i:03d}] {type_name}: {repr(v)}")
    lines.append("")
    
    lines.append("=== Code ===")
    pc = 0
    while pc < len(chunk.code):
        opcode = chunk.code[pc]
        name = OpCode(opcode).name if opcode in OpCode._value2member_map_ else f"0x{opcode:02X}"
        
        # Determine operand count
        if opcode in (OpCode.JUMP, OpCode.JUMP_IF, OpCode.JUMP_NOT, OpCode.LOOP):
            operand = (chunk.code[pc+2] << 8) | chunk.code[pc+1] if pc+2 < len(chunk.code) else 0
            lines.append(f"  [{pc:04d}] {name} {operand}")
            pc += 3
        elif opcode in (OpCode.LOAD_CONST, OpCode.LOAD_VAR, OpCode.STORE_VAR,
                        OpCode.CALL_ODU, OpCode.CALL_ESE):
            operand = chunk.code[pc+1] if pc+1 < len(chunk.code) else 0
            lines.append(f"  [{pc:04d}] {name} {operand}")
            pc += 2
        else:
            lines.append(f"  [{pc:04d}] {name}")
            pc += 1
    
    return "\n".join(lines)


# =============================================================================
# DEMO
# =============================================================================

if __name__ == "__main__":
    print("""
╔══════════════════════════════════════════════════════════════╗
║              IFAB BYTECODE FORMAT DEMO                       ║
╠══════════════════════════════════════════════════════════════╣
║  Compile to compact binary for IoT/Smart Dust                ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    # Create a simple bytecode chunk manually
    chunk = BytecodeChunk()
    
    # Push "Hello World"
    hello_idx = chunk.add_constant("Hello World")
    chunk.emit(OpCode.LOAD_CONST, hello_idx)
    
    # Push arg count (1)
    count_idx = chunk.add_constant(1)
    chunk.emit(OpCode.LOAD_CONST, count_idx)
    
    # Call Ìrosù.fọ̀
    chunk.emit(OpCode.CALL_ODU, ODU_INDEX["irosu"])
    fo_idx = chunk.add_constant("fo")
    chunk.emit(OpCode.CALL_ESE, fo_idx)
    
    # Halt
    chunk.emit(OpCode.HALT)
    
    # Show disassembly
    print(disassemble(chunk))
    
    # Serialize
    data = BytecodeSerializer.serialize(chunk)
    print(f"\nSerialized size: {len(data)} bytes")
    print(f"Hex: {data[:32].hex()}...")
    
    # Deserialize and run
    loaded = BytecodeSerializer.deserialize(data)
    print("\n=== Execution ===")
    vm = BytecodeVM()
    vm.run(loaded, verbose=True)
