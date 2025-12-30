# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFA ISA - AMÚLÙ ARCHITECTURE                              ║
║                    4096 Instruction Set (256 Compound × 16 Verbs)            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  A 12-bit micro-instruction architecture based on the 256 Compound Odù.     ║
║                                                                              ║
║  INSTRUCTION LEVELS:                                                         ║
║    • 4-bit  (16):   Principal Odù (basic domains)                           ║
║    • 8-bit  (256):  Compound Odù (Parent_Child modules)                     ║
║    • 12-bit (4096): Full ISA (Compound + Verb)                              ║
║                                                                              ║
║  12-BIT LAYOUT:                                                              ║
║    ┌────────────┬────────────┬────────────┐                                 ║
║    │   Parent   │   Child    │   Verb     │                                 ║
║    │  (4-bit)   │  (4-bit)   │  (4-bit)   │                                 ║
║    └────────────┴────────────┴────────────┘                                 ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""


# =============================================================================
# THE 16 PRINCIPAL ODÙ - The DNA of the Language
# =============================================================================
ODU_ROOTS = {
    # Binary   Name         As Verb (OpCode)       As Noun (Target)
    "1111": {"name": "OGBE",      "verb": "INIT",  "noun": "GLOBAL"},
    "0000": {"name": "OYEKU",     "verb": "HALT",  "noun": "VOID"},
    "0110": {"name": "IWORI",     "verb": "LOOP",  "noun": "STACK"},
    "1001": {"name": "ODI",       "verb": "SAVE",  "noun": "DISK"},
    "1100": {"name": "IROSU",     "verb": "EMIT",  "noun": "CONSOLE"},
    "0011": {"name": "OWONRIN",   "verb": "SWAP",  "noun": "POINTER"},
    "1000": {"name": "OBARA",     "verb": "ADD",   "noun": "ACCUM"},
    "0001": {"name": "OKANRAN",   "verb": "THROW", "noun": "ERRLOG"},
    "1110": {"name": "OGUNDA",    "verb": "ALLOC", "noun": "HEAP"},
    "0111": {"name": "OSA",       "verb": "JUMP",  "noun": "FLAG"},
    "0100": {"name": "IKA",       "verb": "PACK",  "noun": "ARRAY"},
    "0010": {"name": "OTURUPON",  "verb": "SUB",   "noun": "CONST"},
    "1011": {"name": "OTURA",     "verb": "SEND",  "noun": "SOCKET"},
    "1101": {"name": "IRETE",     "verb": "FREE",  "noun": "GARBAGE"},
    "1010": {"name": "OSE",       "verb": "DRAW",  "noun": "SCREEN"},
    "0101": {"name": "OFUN",      "verb": "NEW",   "noun": "OBJECT"},
}

# Reverse lookups
VERB_TO_BINARY = {v["verb"]: k for k, v in ODU_ROOTS.items()}
NOUN_TO_BINARY = {v["noun"]: k for k, v in ODU_ROOTS.items()}
NAME_TO_BINARY = {v["name"]: k for k, v in ODU_ROOTS.items()}


# =============================================================================
# THE ISA DECODER
# =============================================================================
class IfaISA:
    """
    The Amúlù Instruction Set Architecture Decoder.
    Decodes 8-bit instructions into VERB_NOUN commands.
    """
    
    def decode(self, binary_byte: str):
        """
        Decodes an 8-bit Odù into a command.
        Returns: (odu_name, command, verb, noun)
        """
        if len(binary_byte) != 8:
            raise ValueError(f"Expected 8-bit string, got {len(binary_byte)}")
        
        right_leg = binary_byte[0:4]  # Verb (OpCode)
        left_leg = binary_byte[4:8]   # Noun (Target)
        
        verb_data = ODU_ROOTS.get(right_leg, {"name": "?", "verb": "UNKNOWN"})
        noun_data = ODU_ROOTS.get(left_leg, {"name": "?", "noun": "UNKNOWN"})
        
        odu_name = f"{verb_data['name']}-{noun_data['name']}"
        command = f"{verb_data['verb']}_{noun_data['noun']}"
        
        return odu_name, command, verb_data['verb'], noun_data['noun']
    
    def encode(self, verb: str, noun: str) -> str:
        """
        Encodes a VERB + NOUN into an 8-bit binary string.
        """
        verb_bin = VERB_TO_BINARY.get(verb, "0000")
        noun_bin = NOUN_TO_BINARY.get(noun, "0000")
        return verb_bin + noun_bin
    
    def encode_by_name(self, right_odu: str, left_odu: str) -> str:
        """
        Encodes two Odù names into an 8-bit binary string.
        """
        right_bin = NAME_TO_BINARY.get(right_odu.upper(), "0000")
        left_bin = NAME_TO_BINARY.get(left_odu.upper(), "0000")
        return right_bin + left_bin
    
    def print_matrix(self):
        """Print the complete 16×16 instruction matrix."""
        nouns = list(ODU_ROOTS.keys())
        
        print("\n" + "=" * 100)
        print("THE AMÚLÙ MATRIX - 256 INSTRUCTIONS")
        print("=" * 100)
        print(f"{'VERB↓ NOUN→':<12}", end="")
        for n in nouns:
            print(f"{ODU_ROOTS[n]['noun'][:6]:<8}", end="")
        print()
        print("-" * 100)
        
        for v_bin in nouns:
            verb = ODU_ROOTS[v_bin]['verb']
            print(f"{verb:<12}", end="")
            for n_bin in nouns:
                noun = ODU_ROOTS[n_bin]['noun']
                cmd = f"{verb}_{noun}"[:7]
                print(f"{cmd:<8}", end="")
            print()
    
    def list_all_instructions(self):
        """Return a list of all 256 instructions."""
        instructions = []
        for right_bin, right_data in ODU_ROOTS.items():
            for left_bin, left_data in ODU_ROOTS.items():
                binary = right_bin + left_bin
                instructions.append({
                    'binary': binary,
                    'hex': hex(int(binary, 2)),
                    'odu': f"{right_data['name']}-{left_data['name']}",
                    'command': f"{right_data['verb']}_{left_data['noun']}",
                    'verb': right_data['verb'],
                    'noun': left_data['noun'],
                })
        return instructions


# =============================================================================
# HELPER FUNCTIONS
# =============================================================================
def assemble(verb: str, noun: str, value: int = 0):
    """
    Helper to create an instruction tuple from verb + noun.
    """
    isa = IfaISA()
    binary = isa.encode(verb, noun)
    return (binary, value)


def disassemble(binary: str):
    """
    Convert binary instruction to human-readable.
    """
    isa = IfaISA()
    odu_name, command, verb, noun = isa.decode(binary)
    return f"{odu_name}: {command}"


# =============================================================================
# EXPORTS
# =============================================================================
__all__ = [
    'ODU_ROOTS',
    'VERB_TO_BINARY',
    'NOUN_TO_BINARY',
    'NAME_TO_BINARY',
    'IfaISA',
    'assemble',
    'disassemble',
]
