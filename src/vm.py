# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    THE BABALAWO DEBUGGER                                     ║
║              Spiritual Error Interpretation for Ifá-Lang                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Transforms cold crashes into spiritual consultations with Ifá proverbs.     ║
║  A crash is not a bug—it's a message from the Orisha.                        ║
║                                                                              ║
║  Integrated with src/errors.py for unified error handling.                   ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import time
import sys

# Try to import the unified error system
try:
    from src.errors import Babalawo as BabalawoErrors, IfaError, ODU_WISDOM
except ImportError:
    try:
        from errors import Babalawo as BabalawoErrors, IfaError, ODU_WISDOM
    except ImportError:
        BabalawoErrors = None
        IfaError = None
        ODU_WISDOM = None


class BabalawoDebugger:
    """
    The Diviner - Interprets VM crashes as spiritual crises.
    Performs Dídá (Divination) on dead processes.
    """
    
    def __init__(self):
        # The Proverb Dictionary - Maps Crash Odù to Wisdom
        self.wisdom = {
            # === MATH ERRORS ===
            "OTURUPON-OYEKU": (
                "One cannot carry a load that does not exist.",
                "Division by Zero"
            ),
            "OBARA-OYEKU": (
                "The King attempted to expand into the Void. The kingdom collapsed.",
                "Null Pointer Exception"
            ),
            "SUB-VOID": (
                "You cannot take from that which has never been given.",
                "Underflow Error"
            ),
            
            # === MEMORY ERRORS ===
            "OGUNDA-OYEKU": (
                "The Iron Cutter struck empty air. There was nothing to cut.",
                "Allocation Error / Zero Size"
            ),
            "IRETE-OGBE": (
                "You tried to crush the Source of Life. One cannot delete the Global Scope.",
                "Permission Denied"
            ),
            "ALLOC-VOID": (
                "One does not build a house in the sky. Memory must have a foundation.",
                "Stack Overflow"
            ),
            "FREE-GLOBAL": (
                "The root of the tree cannot be pulled while it still bears fruit.",
                "Cannot Free Global Memory"
            ),
            
            # === I/O ERRORS ===
            "ODI-OYEKU": (
                "You tried to seal a Calabash that was not there.",
                "File Not Found / Invalid Handle"
            ),
            "OTURA-OYEKU": (
                "The Messenger shouted into the abyss, but no one answered.",
                "Network Timeout / Connection Refused"
            ),
            "SAVE-VOID": (
                "One cannot pour water into a vessel that does not exist.",
                "Write to Null Stream"
            ),
            "EMIT-GARBAGE": (
                "The mouth cannot speak what the heart has already forgotten.",
                "Dangling Pointer / Use After Free"
            ),
            
            # === FLOW CONTROL ERRORS ===
            "LOOP-OYEKU": (
                "A river cannot flow through a dam of nothingness. The cycle is broken.",
                "Infinite Loop Without Exit"
            ),
            "JUMP-VOID": (
                "The bird tried to fly to a branch that does not exist.",
                "Invalid Jump Target"
            ),
            "IWORI-OYEKU": (
                "One cannot reflect upon emptiness. Even mirrors need light.",
                "Recursion Depth Exceeded"
            ),
            
            # === TYPE ERRORS ===
            "OFUN-OYEKU": (
                "You tried to birth a child from the womb of nothingness.",
                "Cannot Instantiate Null Type"
            ),
            "PACK-VOID": (
                "The bag cannot hold what was never placed inside.",
                "Compression of Empty Buffer"
            ),
            
            # === GRAPHICS ERRORS ===
            "OSE-OYEKU": (
                "The artist's brush touched the canvas, but the canvas was a shadow.",
                "Render Target Missing"
            ),
            "DRAW-GARBAGE": (
                "Beauty cannot be drawn from corruption.",
                "Invalid Framebuffer"
            ),
        }
        
        # The 16 Principal Roots
        self.roots = {
            "1111": "OGBE",    "0000": "OYEKU",   "0110": "IWORI",   "1001": "ODI",
            "1100": "IROSU",   "0011": "OWONRIN", "1000": "OBARA",   "0001": "OKANRAN",
            "1110": "OGUNDA",  "0111": "OSA",     "0100": "IKA",     "0010": "OTURUPON",
            "1011": "OTURA",   "1101": "IRETE",   "1010": "OSE",     "0101": "OFUN"
        }
        
        # Verb meanings for fallback interpretation
        self.verb_meanings = {
            "OGBE": "The Light tried to shine",
            "OYEKU": "The Darkness spread",
            "IWORI": "The Reflection sought itself",
            "ODI": "The Womb tried to contain",
            "IROSU": "The Voice attempted to speak",
            "OWONRIN": "The Reversal was demanded",
            "OBARA": "The King sought expansion",
            "OKANRAN": "The Trouble manifested",
            "OGUNDA": "The Iron attempted to cut",
            "OSA": "The Wind tried to escape",
            "IKA": "The Constrictor squeezed",
            "OTURUPON": "The Bearer tried to carry",
            "OTURA": "The Messenger traveled",
            "IRETE": "The Crusher descended",
            "OSE": "The Beauty was sought",
            "OFUN": "The Creator attempted birth"
        }
        
        self.noun_meanings = {
            "OGBE": "upon the Source of All",
            "OYEKU": "into the Void of Nothingness",
            "IWORI": "within the Mirror of History",
            "ODI": "into the Sealed Vessel",
            "IROSU": "toward the Speaking Console",
            "OWONRIN": "across the Reversed Path",
            "OBARA": "within the Accumulator's Heart",
            "OKANRAN": "into the Error Stream",
            "OGUNDA": "upon the Heap of Memory",
            "OSA": "toward the Boolean Flag",
            "IKA": "within the Packed Array",
            "OTURUPON": "with the Constant Value",
            "OTURA": "through the Network Socket",
            "IRETE": "upon the Garbage of the Past",
            "OSE": "onto the Visual Screen",
            "OFUN": "with the New Object"
        }

    def interpret_marks(self, binary_str):
        """Draws the Odù pattern visually."""
        binary_str = binary_str.replace(" ", "")
        if len(binary_str) != 8:
            return
            
        right = binary_str[0:4]
        left = binary_str[4:8]
        
        print("\n┌─────────────────────────┐")
        print("│   THE SIGN OF DEATH     │")
        print("├─────────────────────────┤")
        for i in range(4):
            r_mark = "I " if right[i] == '1' else "II"
            l_mark = "I " if left[i] == '1' else "II"
            print(f"│       {l_mark}    {r_mark}        │")
        print("└─────────────────────────┘")

    def diagnose(self, crash_context):
        """
        Analyzes the VM state at the moment of death.
        
        crash_context = {
            'binary': '10010000',
            'registers': {'OKE': 0, 'ISALE': 0, ...},
            'reason': 'ZeroDivisionError',
            'line': 5
        }
        """
        binary = crash_context.get('binary', '00000000').replace(" ", "")
        registers = crash_context.get('registers', {})
        reason = crash_context.get('reason', 'Unknown Error')
        
        # Decode the legs
        right_leg = binary[0:4]
        left_leg = binary[4:8]
        
        verb = self.roots.get(right_leg, "UNKNOWN")
        noun = self.roots.get(left_leg, "UNKNOWN")
        
        odu_name = f"{verb}-{noun}"
        
        # Print the divination
        print("\n" + "=" * 50)
        print(">>> THE BABALAWO SPEAKS <<<")
        print("=" * 50)
        
        self.interpret_marks(binary)
        
        print(f"\n  The Odù Revealed: {odu_name}")
        print(f"  Binary Pattern:   {binary[:4]} {binary[4:]}")
        
        # Consult the Wisdom Dictionary
        wisdom_entry = self.wisdom.get(odu_name)
        
        # Also check the unified ODU_WISDOM from errors.py
        odu_wisdom_entry = None
        proverb_from_errors = None
        advice_from_errors = None
        if ODU_WISDOM and verb in ODU_WISDOM:
            odu_wisdom_entry = ODU_WISDOM[verb]
            proverb_from_errors = odu_wisdom_entry.get("proverbs", [""])[0]
            advice_from_errors = odu_wisdom_entry.get("advice", "")
        
        # Generate dynamic proverb if not in dictionary
        if wisdom_entry:
            proverb, tech_meaning = wisdom_entry
        elif proverb_from_errors:
            proverb = proverb_from_errors
            tech_meaning = advice_from_errors or "See wisdom advice"
        else:
            verb_story = self.verb_meanings.get(verb, f"The force of {verb} acted")
            noun_story = self.noun_meanings.get(noun, f"upon {noun}")
            proverb = f"{verb_story} {noun_story}. But the universe rejected this union."
            tech_meaning = "Unknown Conflict"
        
        # SPIRITUAL INTERPRETATION
        print("\n┌─────────────────────────────────────────────────────────┐")
        print("│            SPIRITUAL INTERPRETATION                     │")
        print("├─────────────────────────────────────────────────────────┤")
        print(f"│ Proverb:")
        print(f"│   \"{proverb}\"")
        print("└─────────────────────────────────────────────────────────┘")
        
        # TECHNICAL INTERPRETATION  
        print("\n┌─────────────────────────────────────────────────────────┐")
        print("│            TECHNICAL INTERPRETATION                     │")
        print("├─────────────────────────────────────────────────────────┤")
        print(f"│ Error Type:  {tech_meaning}")
        print(f"│ Raw Error:   {reason}")
        print(f"│ Instruction: {odu_name} (0x{int(binary, 2):02X})")
        print("└─────────────────────────────────────────────────────────┘")
        
        # Register state
        print("\n[Register State at Death]")
        print(f"  Ìsàlẹ̀ (Accumulator): {registers.get('ISALE', '?')}")
        print(f"  Òkè (Instruction Ptr): {registers.get('OKE', '?')}")
        print(f"  Ọ̀tún (X Register): {registers.get('OTUN', '?')}")
        print(f"  Òsì (Y Register): {registers.get('OSI', '?')}")
        
        # Prescription
        print("\n┌─────────────────────────────────────────────────┐")
        print("│              PRESCRIPTION (ẸBỌ)                 │")
        print("├─────────────────────────────────────────────────┤")
        
        prescriptions = self._get_prescription(verb, noun)
        for rx in prescriptions:
            print(f"│ • {rx}")
        
        print("└─────────────────────────────────────────────────┘")

    def _get_prescription(self, verb, noun):
        """Generate remediation suggestions based on the conflict."""
        prescriptions = []
        
        if noun == "OYEKU":
            prescriptions.append("Check for null/zero values before operating.")
            prescriptions.append("Initialize all variables before use.")
        
        if verb == "OTURUPON":
            prescriptions.append("Validate divisor is non-zero.")
        
        if verb == "OTURA":
            prescriptions.append("Verify network connection before sending.")
            prescriptions.append("Add timeout handling.")
        
        if verb == "ODI":
            prescriptions.append("Confirm file exists before writing.")
            prescriptions.append("Use try-catch around file operations.")
        
        if verb == "OGUNDA":
            prescriptions.append("Check available memory before allocation.")
        
        if not prescriptions:
            prescriptions.append("Review the code logic at this instruction.")
            prescriptions.append("Offer Ẹbọ (Refactoring) to restore balance.")
        
        return prescriptions


# =============================================================================
# THE VM WITH INTEGRATED DEBUGGER
# =============================================================================
class OponVM:
    """OponVM with Babalawo Debugger integration."""
    
    def __init__(self):
        self.registers = {"OKE": 0, "ISALE": 0, "OTUN": 0, "OSI": 0}
        self.memory = [0] * 256
        self.debugger = BabalawoDebugger()
        
        self.roots = {
            "1111": "OGBE",    "0000": "OYEKU",   "0110": "IWORI",   "1001": "ODI",
            "1100": "IROSU",   "0011": "OWONRIN", "1000": "OBARA",   "0001": "OKANRAN",
            "1110": "OGUNDA",  "0111": "OSA",     "0100": "IKA",     "0010": "OTURUPON",
            "1011": "OTURA",   "1101": "IRETE",   "1010": "OSE",     "0101": "OFUN"
        }

    def execute(self, program, verbose=True):
        """Execute with crash protection."""
        if verbose:
            print("--- Ìbà: System Booting ---")
        self.registers["OKE"] = 0
        
        try:
            while self.registers["OKE"] < len(program):
                binary_op, val = program[self.registers["OKE"]]
                binary_clean = binary_op.replace(" ", "")
                
                # Decode instruction
                right_leg = binary_clean[0:4]
                left_leg = binary_clean[4:8]
                verb = self.roots.get(right_leg, "UNKNOWN")
                noun = self.roots.get(left_leg, "UNKNOWN")
                
                if verbose:
                    print(f"[{self.registers['OKE']:03d}] {verb}-{noun} ({val})")
                
                # Execute based on verb
                self._execute_verb(verb, noun, val, binary_clean)
                
                self.registers["OKE"] += 1
                
        except Exception as e:
            # Summon the Babalawo
            context = {
                'binary': binary_clean,
                'registers': self.registers.copy(),
                'reason': str(e)
            }
            self.debugger.diagnose(context)

    def _execute_verb(self, verb, noun, val, binary):
        """Execute a single instruction."""
        
        # ADD
        if verb == "OBARA":
            self.registers["ISALE"] = (self.registers["ISALE"] + val) % 256
        
        # SUBTRACT / DIVIDE (can crash on zero)
        elif verb == "OTURUPON":
            if val == 0:
                raise ZeroDivisionError("Division by the Void")
            self.registers["ISALE"] //= val
        
        # OUTPUT
        elif verb == "IROSU":
            print(f"Output: {self.registers['ISALE']}")
        
        # SAVE (can crash on invalid handle)
        elif verb == "ODI":
            if val == -1:
                raise FileNotFoundError("Vessel not found")
            # Simulated save
        
        # NETWORK (can crash on no connection)
        elif verb == "OTURA":
            if noun == "OYEKU":
                raise ConnectionRefusedError("The abyss does not answer")
        
        # HALT
        elif verb == "OYEKU":
            raise SystemExit(0)


# =============================================================================
# DEMO
# =============================================================================
if __name__ == "__main__":
    print("""
╔══════════════════════════════════════════════════════════════╗
║              THE BABALAWO DEBUGGER DEMO                      ║
╠══════════════════════════════════════════════════════════════╣
║  Watch as crashes become spiritual consultations...          ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    # Test Program: Intentionally crash with division by zero
    broken_program = [
        ("10001000", 10),   # ADD 10 (Obara-Obara)
        ("10001000", 5),    # ADD 5  (Obara-Obara) -> ISALE = 15
        ("00100000", 0),    # DIV 0  (Oturupon-Oyeku) -> CRASH!
    ]
    
    print("Executing program that will crash...")
    print("-" * 50)
    
    vm = OponVM()
    vm.execute(broken_program)
