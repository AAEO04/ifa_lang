# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           IFÁ ERROR SYSTEM - THE BABALAWO                                    ║
║                    Proverb-Based Error Messages                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Instead of cold, unhelpful error messages like:                             ║
║    "Error: NullPointer at line 40"                                           ║
║                                                                              ║
║  The Babalawo speaks wisdom:                                                 ║
║    "The path is blocked at Line 40. Ògúndá (The Cutter) attempted to         ║
║     clear a path that does not exist. Check if your variable was             ║
║     initialized with Ogbè."                                                  ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

from dataclasses import dataclass
from typing import Dict, List, Optional, Any
import random


# =============================================================================
# ODÙ WISDOM - Proverbs and meanings for each domain
# =============================================================================

ODU_WISDOM = {
    "OGBE": {
        "name": "Ogbè",
        "title": "The Light",
        "meaning": "Beginnings, initialization, birth",
        "proverbs": [
            "A journey of a thousand miles begins with a single step.",
            "The dawn breaks for those who are prepared.",
            "Light enters where there is an opening.",
        ],
        "advice": "Check your initialization. Ogbè teaches that all things must have a proper beginning.",
    },
    "OYEKU": {
        "name": "Ọ̀yẹ̀kú",
        "title": "The Darkness",
        "meaning": "Endings, termination, completion",
        "proverbs": [
            "All rivers flow to the sea.",
            "Even the longest night ends with dawn.",
            "The path that begins must also end.",
        ],
        "advice": "Ensure proper termination. Ọ̀yẹ̀kú reminds us that endings must be honored.",
    },
    "IWORI": {
        "name": "Ìwòrì",
        "title": "The Mirror",
        "meaning": "Reflection, iteration, loops",
        "proverbs": [
            "The river does not flow backwards.",
            "What you seek is seeking you.",
            "The mirror shows truth to those who look.",
        ],
        "advice": "Check your loop conditions. Ìwòrì teaches that cycles must have purpose.",
    },
    "ODI": {
        "name": "Òdí",
        "title": "The Vessel",
        "meaning": "Storage, files, containment",
        "proverbs": [
            "The calabash can only hold what it is given.",
            "An empty vessel makes the most noise.",
            "Guard well what you store.",
        ],
        "advice": "Verify your file operations. Òdí teaches that vessels must be opened before use and closed after.",
    },
    "IROSU": {
        "name": "Ìrosù",
        "title": "The Speaker",
        "meaning": "Communication, output, expression",
        "proverbs": [
            "Words once spoken cannot be recalled.",
            "The wise speak with purpose.",
            "Let your speech be seasoned with wisdom.",
        ],
        "advice": "Check your output format. Ìrosù teaches that communication must be clear.",
    },
    "OWONRIN": {
        "name": "Ọ̀wọ́nrín",
        "title": "The Chaotic",
        "meaning": "Randomness, chance, unpredictability",
        "proverbs": [
            "The wind blows where it wills.",
            "Chaos contains the seed of order.",
            "Expect the unexpected.",
        ],
        "advice": "Account for randomness. Ọ̀wọ́nrín teaches that chaos must be embraced, not feared.",
    },
    "OBARA": {
        "name": "Ọ̀bàrà",
        "title": "The King",
        "meaning": "Expansion, addition, growth",
        "proverbs": [
            "The tree grows from within.",
            "Small drops fill the ocean.",
            "Growth requires patience and consistency.",
        ],
        "advice": "Check your arithmetic. Ọ̀bàrà teaches that expansion must respect boundaries.",
    },
    "OKANRAN": {
        "name": "Ọ̀kànràn",
        "title": "The Troublemaker",
        "meaning": "Errors, exceptions, warnings",
        "proverbs": [
            "The squeaking wheel gets the oil.",
            "Problems are opportunities in disguise.",
            "Face your troubles head-on.",
        ],
        "advice": "Handle your exceptions. Ọ̀kànràn teaches that errors are teachers.",
    },
    "OGUNDA": {
        "name": "Ògúndá",
        "title": "The Cutter",
        "meaning": "Arrays, process control, separation",
        "proverbs": [
            "The machete cuts the path.",
            "Not all that is separated is lost.",
            "To divide is also to organize.",
        ],
        "advice": "Check your array bounds. Ògúndá teaches that cutting must be precise.",
    },
    "OSA": {
        "name": "Ọ̀sá",
        "title": "The Wind",
        "meaning": "Control flow, jumps, conditionals",
        "proverbs": [
            "The wind changes direction without announcement.",
            "Flexibility is strength.",
            "Many paths lead to the same destination.",
        ],
        "advice": "Verify your conditionals. Ọ̀sá teaches that flow must have logic.",
    },
    "IKA": {
        "name": "Ìká",
        "title": "The Constrictor",
        "meaning": "Strings, compression, binding",
        "proverbs": [
            "The rope that binds can also free.",
            "Words are the threads that bind meaning.",
            "What is bound together must also be released.",
        ],
        "advice": "Check your string operations. Ìká teaches that binding must be intentional.",
    },
    "OTURUPON": {
        "name": "Òtúúrúpọ̀n",
        "title": "The Bearer",
        "meaning": "Reduction, subtraction, division",
        "proverbs": [
            "Sharing lightens the load.",
            "Less can be more.",
            "Division creates new wholes.",
        ],
        "advice": "Watch for division by zero. Òtúúrúpọ̀n teaches that subtraction requires substance.",
    },
    "OTURA": {
        "name": "Òtúrá",
        "title": "The Messenger",
        "meaning": "Network, communication, sending",
        "proverbs": [
            "The messenger is not the message.",
            "Bridges connect distant shores.",
            "News travels faster than the wind.",
        ],
        "advice": "Check your network connections. Òtúrá teaches that messages need receivers.",
    },
    "IRETE": {
        "name": "Ìrẹtẹ̀",
        "title": "The Crusher",
        "meaning": "Memory management, garbage collection",
        "proverbs": [
            "Make space for the new by releasing the old.",
            "The granary must be emptied before the harvest.",
            "What is no longer needed becomes burden.",
        ],
        "advice": "Free your memory. Ìrẹtẹ̀ teaches that release creates space for growth.",
    },
    "OSE": {
        "name": "Ọ̀ṣẹ́",
        "title": "The Beautifier",
        "meaning": "Graphics, display, aesthetics",
        "proverbs": [
            "Beauty speaks without words.",
            "The canvas awaits the artist.",
            "Form follows function.",
        ],
        "advice": "Check your display coordinates. Ọ̀ṣẹ́ teaches that beauty requires precision.",
    },
    "OFUN": {
        "name": "Òfún",
        "title": "The Creator",
        "meaning": "Object creation, inheritance",
        "proverbs": [
            "From nothing, something emerges.",
            "The child inherits from the parent.",
            "Creation is the highest art.",
        ],
        "advice": "Verify your object creation. Òfún teaches that creation requires intention.",
    },
}


# =============================================================================
# ERROR TYPES
# =============================================================================

@dataclass
class IfaError:
    """An Ifá error with Babalawo-style messaging."""
    code: str
    odu: str
    line: int
    column: int = 0
    message: str = ""
    technical: str = ""
    context: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.context is None:
            self.context = {}


# =============================================================================
# THE BABALAWO - Error Diagnosis System
# =============================================================================

class Babalawo:
    """
    The Babalawo (Priest) - Diagnoses errors and provides wisdom.
    
    Instead of cold error messages, the Babalawo speaks with proverbs
    and practical advice rooted in Ifá philosophy.
    """
    
    # Error code mappings to Odù domains
    ERROR_TO_ODU = {
        # Initialization errors → Ogbè
        "UNINITIALIZED": "OGBE",
        "NULL_REFERENCE": "OGBE",
        "UNDEFINED_VARIABLE": "OGBE",
        
        # Termination errors → Oyeku
        "UNCLOSED_RESOURCE": "OYEKU",
        "ORPHAN_PROCESS": "OYEKU",
        "INCOMPLETE_SHUTDOWN": "OYEKU",
        
        # Loop errors → Iwori
        "INFINITE_LOOP": "IWORI",
        "ITERATOR_EXHAUSTED": "IWORI",
        "LOOP_INVARIANT_VIOLATED": "IWORI",
        
        # File errors → Odi
        "FILE_NOT_FOUND": "ODI",
        "FILE_ALREADY_OPEN": "ODI",
        "FILE_NOT_CLOSED": "ODI",
        "PERMISSION_DENIED": "ODI",
        
        # Output errors → Irosu
        "FORMAT_ERROR": "IROSU",
        "OUTPUT_OVERFLOW": "IROSU",
        
        # Random errors → Owonrin
        "SEED_ERROR": "OWONRIN",
        
        # Math errors → Obara/Oturupon
        "OVERFLOW": "OBARA",
        "UNDERFLOW": "OTURUPON",
        "DIVISION_BY_ZERO": "OTURUPON",
        "ARITHMETIC_ERROR": "OBARA",
        
        # Exception errors → Okanran
        "UNHANDLED_EXCEPTION": "OKANRAN",
        "ASSERTION_FAILED": "OKANRAN",
        
        # Array errors → Ogunda
        "INDEX_OUT_OF_BOUNDS": "OGUNDA",
        "ARRAY_EMPTY": "OGUNDA",
        
        # Control flow errors → Osa
        "UNREACHABLE_CODE": "OSA",
        "INVALID_JUMP": "OSA",
        "MISSING_RETURN": "OSA",
        
        # String errors → Ika
        "INVALID_ENCODING": "IKA",
        "STRING_OVERFLOW": "IKA",
        
        # Network errors → Otura
        "CONNECTION_REFUSED": "OTURA",
        "TIMEOUT": "OTURA",
        "NETWORK_UNREACHABLE": "OTURA",
        
        # Memory errors → Irete
        "MEMORY_LEAK": "IRETE",
        "DOUBLE_FREE": "IRETE",
        "OUT_OF_MEMORY": "IRETE",
        
        # Graphics errors → Ose
        "INVALID_COORDINATES": "OSE",
        "BUFFER_OVERFLOW": "OSE",
        
        # Object errors → Ofun
        "TYPE_ERROR": "OFUN",
        "INHERITANCE_ERROR": "OFUN",
        "OBJECT_NOT_FOUND": "OFUN",
    }
    
    def __init__(self):
        self.history: List[IfaError] = []
    
    def diagnose(self, error: IfaError) -> str:
        """
        Diagnose an error and return Babalawo-style message.
        """
        self.history.append(error)
        
        # Get the Odù wisdom for this error
        odu_key = self.ERROR_TO_ODU.get(error.code, error.odu.upper())
        wisdom = ODU_WISDOM.get(odu_key, ODU_WISDOM["OKANRAN"])
        
        # Select a random proverb
        proverb = random.choice(wisdom["proverbs"])
        
        # Build the message
        message = f"""
╔══════════════════════════════════════════════════════════════════════════════╗
║  🔮 THE BABALAWO SPEAKS                                                      ║
╚══════════════════════════════════════════════════════════════════════════════╝

  The path is blocked at Line {error.line}.
  
  📖 {wisdom['name']} ({wisdom['title']}) says:
     "{proverb}"
  
  ⚠️  What happened:
     {self._format_error_message(error)}
  
  💡 Wisdom:
     {wisdom['advice']}
  
  🔧 Technical:
     Error Code: {error.code}
     Domain: {wisdom['name']} ({wisdom['meaning']})
     Location: Line {error.line}, Column {error.column}
"""
        
        if error.technical:
            message += f"     Details: {error.technical}\n"
        
        message += "\n  Àṣẹ.\n"
        
        return message
    
    def _format_error_message(self, error: IfaError) -> str:
        """Format the error message with context."""
        messages = {
            "UNINITIALIZED": f"You tried to use a variable that was never given life with Ogbè (INIT).",
            "NULL_REFERENCE": f"You reached into the void and found nothing. The path does not exist.",
            "UNDEFINED_VARIABLE": f"The name '{error.context.get('name', 'unknown')}' echoes in emptiness. It was never spoken into being.",
            
            "FILE_NOT_FOUND": f"The vessel '{error.context.get('path', 'unknown')}' cannot be found. Perhaps it was never carved.",
            "FILE_NOT_CLOSED": f"A vessel remains open. Òdí teaches: what is opened must be closed.",
            "FILE_ALREADY_OPEN": f"You cannot open what is already open. The vessel awaits filling, not opening.",
            
            "DIVISION_BY_ZERO": f"You tried to divide by the void. Òtúúrúpọ̀n cannot bear what does not exist.",
            "OVERFLOW": f"The number grew beyond its container. Even the king's treasury has limits.",
            
            "INDEX_OUT_OF_BOUNDS": f"You reached beyond the array's edge at position {error.context.get('index', '?')}. The array holds only {error.context.get('length', '?')} elements.",
            
            "CONNECTION_REFUSED": f"The distant Opon refused your greeting. Check if they are listening.",
            "TIMEOUT": f"The messenger waited too long. The spirits did not respond in time.",
            
            "MEMORY_LEAK": f"Memory was allocated but never freed. The granary overflows with old grain.",
            "DOUBLE_FREE": f"You tried to free what was already released. The burden was already lifted.",
            
            "TYPE_ERROR": f"Expected {error.context.get('expected', 'one thing')}, received {error.context.get('received', 'another')}. The offering was incorrect.",
            
            "UNHANDLED_EXCEPTION": f"An unexpected spirit emerged at Line {error.line}. It must be acknowledged.",
        }
        
        return messages.get(error.code, error.message or f"An error occurred: {error.code}")
    
    def quick_diagnose(self, code: str, line: int, **context) -> str:
        """Quick diagnosis for common errors."""
        error = IfaError(
            code=code,
            odu=self.ERROR_TO_ODU.get(code, "OKANRAN"),
            line=line,
            context=context
        )
        return self.diagnose(error)
    
    def format_simple(self, error: IfaError) -> str:
        """Simple one-line format for inline errors."""
        wisdom = ODU_WISDOM.get(self.ERROR_TO_ODU.get(error.code, "OKANRAN"))
        return f"[{wisdom['name']}] Line {error.line}: {self._format_error_message(error)}"


# =============================================================================
# GLOBAL INSTANCE
# =============================================================================

babalawo = Babalawo()


# =============================================================================
# CONVENIENCE FUNCTIONS
# =============================================================================

def speak(code: str, line: int, **context) -> str:
    """Let the Babalawo speak about an error."""
    return babalawo.quick_diagnose(code, line, **context)


def warn(message: str, line: int = 0):
    """Print a warning in Babalawo style."""
    print(f"⚠️  [Ọ̀kànràn] Warning at Line {line}: {message}")


def hint(message: str):
    """Print a helpful hint."""
    print(f"💡 [Wisdom]: {message}")


# =============================================================================
# DEMO
# =============================================================================

if __name__ == "__main__":
    # Demo: Show different error messages
    print("\n=== Babalawo Error System Demo ===\n")
    
    # Null reference
    print(speak("NULL_REFERENCE", 40))
    
    # Division by zero
    print(speak("DIVISION_BY_ZERO", 25))
    
    # Index out of bounds
    print(speak("INDEX_OUT_OF_BOUNDS", 100, index=15, length=10))
    
    # File not found
    print(speak("FILE_NOT_FOUND", 12, path="data.ifa"))
