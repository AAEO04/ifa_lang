# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║               IFA-LANG ESE PARSER (HIGH-LEVEL COMPILER)                      ║
║                   "The Stories Within the Verses"                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Transforms human-readable Odù.Ese() syntax into 8-bit bytecode.             ║
║                                                                              ║
║  Syntax:                                                                     ║
║    ìbà Òtúrá;              # Import the Network domain                       ║
║    Òtúrá.dè(1);            # Bind to port 1                                  ║
║    Òtúrá.rán("Àṣẹ");       # Send message                                    ║
║    Òbàrà.fikun(10);        # Add 10                                          ║
║    Ìrosù.sọ(x);            # Print output                                    ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import re
import sys
import unicodedata


# =============================================================================
# THE 16 ODÙ DOMAINS (Classes)
# =============================================================================
class OduDomain:
    """Base class for all Odù Domains."""
    
    def __init__(self, name, binary):
        self.name = name
        self.binary = binary
        self.ese = {}  # Method registry
    
    def register_ese(self, name, ascii_name, description, target_noun="ACCUM"):
        """Register an Ese (verse/method) for this domain."""
        self.ese[name] = {
            "name": name,
            "ascii": ascii_name,
            "desc": description,
            "target": target_noun
        }
        self.ese[ascii_name] = self.ese[name]  # ASCII alias


# =============================================================================
# THE 16 PRINCIPAL DOMAINS
# =============================================================================
DOMAINS = {}

def setup_domains():
    """Initialize all 16 Odù domains with their Ese (methods)."""
    global DOMAINS
    
    # === OGBÈ (Light/Initialize) ===
    ogbe = OduDomain("Ogbè", "1111")
    ogbe.register_ese("bẹ̀rẹ̀", "bere", "Initialize value")
    ogbe.register_ese("tún", "tun", "Reset to zero")
    ogbe.register_ese("gbé", "gbe", "Load value")
    DOMAINS["OGBE"] = DOMAINS["Ogbè"] = DOMAINS["ogbe"] = ogbe
    
    # === OYEKU (Dark/Terminate) ===
    oyeku = OduDomain("Ọ̀yẹ̀kú", "0000")
    oyeku.register_ese("dúró", "duro", "Halt execution")
    oyeku.register_ese("pa", "pa", "Kill process")
    oyeku.register_ese("sùn", "sun", "Sleep/pause")
    DOMAINS["OYEKU"] = DOMAINS["Ọ̀yẹ̀kú"] = DOMAINS["oyeku"] = oyeku
    
    # === IWORI (Loop/Reflect) ===
    iwori = OduDomain("Ìwòrì", "0110")
    iwori.register_ese("yípo", "yipo", "Loop/iterate")
    iwori.register_ese("padà", "pada", "Return/go back")
    iwori.register_ese("wò", "wo", "Inspect/peek")
    DOMAINS["IWORI"] = DOMAINS["Ìwòrì"] = DOMAINS["iwori"] = iwori
    
    # === ODI (Storage/File) ===
    odi = OduDomain("Òdí", "1001")
    odi.register_ese("ṣí", "si", "Open file")
    odi.register_ese("pa", "pa", "Close file")
    odi.register_ese("kà", "ka", "Read from file")
    odi.register_ese("kọ", "ko", "Write to file")
    odi.register_ese("pa", "paa", "Delete file")
    DOMAINS["ODI"] = DOMAINS["Òdí"] = DOMAINS["odi"] = odi
    
    # === IROSU (Output/Console) ===
    irosu = OduDomain("Ìrosù", "1100")
    irosu.register_ese("sọ", "so", "Print/speak")
    irosu.register_ese("gbọ́", "gbo", "Input/listen")
    irosu.register_ese("mọ́", "mo", "Clear screen")
    DOMAINS["IROSU"] = DOMAINS["Ìrosù"] = DOMAINS["irosu"] = irosu
    
    # === OWONRIN (Swap/Exchange) ===
    owonrin = OduDomain("Ọ̀wọ́nrín", "0011")
    owonrin.register_ese("pàárọ̀", "paaro", "Swap values")
    owonrin.register_ese("yí", "yi", "Reverse")
    DOMAINS["OWONRIN"] = DOMAINS["Ọ̀wọ́nrín"] = DOMAINS["owonrin"] = owonrin
    
    # === OBARA (Math/Expand) ===
    obara = OduDomain("Ọ̀bàrà", "1000")
    obara.register_ese("fikun", "fikun", "Add")
    obara.register_ese("àpapọ̀", "apapo", "Sum/total")
    DOMAINS["OBARA"] = DOMAINS["Ọ̀bàrà"] = DOMAINS["obara"] = obara
    
    # === OKANRAN (Error/Exception) ===
    okanran = OduDomain("Ọ̀kànràn", "0001")
    okanran.register_ese("júwe", "juwe", "Throw error")
    okanran.register_ese("gbà", "gba", "Catch error")
    DOMAINS["OKANRAN"] = DOMAINS["Ọ̀kànràn"] = DOMAINS["okanran"] = okanran
    
    # === OGUNDA (Memory/Allocate) ===
    ogunda = OduDomain("Ògúndá", "1110")
    ogunda.register_ese("gé", "ge", "Allocate/cut")
    ogunda.register_ese("dá", "da", "Create memory")
    DOMAINS["OGUNDA"] = DOMAINS["Ògúndá"] = DOMAINS["ogunda"] = ogunda
    
    # === OSA (Jump/Break) ===
    osa = OduDomain("Ọ̀sá", "0111")
    osa.register_ese("fò", "fo", "Jump")
    osa.register_ese("já", "ja", "Break out")
    osa.register_ese("bá", "ba", "If condition")
    DOMAINS["OSA"] = DOMAINS["Ọ̀sá"] = DOMAINS["osa"] = osa
    
    # === IKA (Compress/Pack) ===
    ika = OduDomain("Ìká", "0100")
    ika.register_ese("dì", "di", "Pack/compress")
    ika.register_ese("tú", "tu", "Unpack")
    ika.register_ese("bò", "bo", "Encrypt")
    ika.register_ese("lá", "la", "Decrypt")
    DOMAINS["IKA"] = DOMAINS["Ìká"] = DOMAINS["ika"] = ika
    
    # === OTURUPON (Subtract/Math) ===
    oturupon = OduDomain("Òtúúrúpọ̀n", "0010")
    oturupon.register_ese("dín", "din", "Subtract")
    oturupon.register_ese("pín", "pin", "Divide")
    oturupon.register_ese("kù", "ku", "Modulo")
    oturupon.register_ese("gbà", "gba", "Multiply")
    DOMAINS["OTURUPON"] = DOMAINS["Òtúúrúpọ̀n"] = DOMAINS["oturupon"] = oturupon
    
    # === OTURA (Network/Send) ===
    otura = OduDomain("Òtúrá", "1011")
    otura.register_ese("dè", "de", "Bind/listen")
    otura.register_ese("rán", "ran", "Send")
    otura.register_ese("gbà", "gba", "Receive")
    otura.register_ese("so", "so", "Connect")
    DOMAINS["OTURA"] = DOMAINS["Òtúrá"] = DOMAINS["otura"] = otura
    
    # === IRETE (Free/GC) ===
    irete = OduDomain("Ìrẹtẹ̀", "1101")
    irete.register_ese("tú", "tu", "Free memory")
    irete.register_ese("nù", "nu", "Garbage collect")
    DOMAINS["IRETE"] = DOMAINS["Ìrẹtẹ̀"] = DOMAINS["irete"] = irete
    
    # === OSE (Graphics/Display) ===
    ose = OduDomain("Ọ̀ṣẹ́", "1010")
    ose.register_ese("yà", "ya", "Draw")
    ose.register_ese("hàn", "han", "Show/render")
    ose.register_ese("nù", "nu", "Clear canvas")
    ose.register_ese("àwọ̀", "awo", "Set color")
    DOMAINS["OSE"] = DOMAINS["Ọ̀ṣẹ́"] = DOMAINS["ose"] = ose
    
    # === OFUN (Objects/Create) ===
    ofun = OduDomain("Òfún", "0101")
    ofun.register_ese("dá", "da", "Create object")
    ofun.register_ese("pa", "pa", "Delete object")
    ofun.register_ese("rí", "ri", "Get property")
    ofun.register_ese("fi", "fi", "Set property")
    DOMAINS["OFUN"] = DOMAINS["Òfún"] = DOMAINS["ofun"] = ofun


# Initialize domains
setup_domains()


# =============================================================================
# THE ESE COMPILER
# =============================================================================
class EseCompiler:
    """Compiles Ese (high-level) syntax to bytecode."""
    
    # Bytecode patterns
    OPCODES = {
        ("OBARA", "fikun"):    ("10001000", "ADD"),
        ("OBARA", "apapo"):    ("10001000", "ADD"),
        ("OTURUPON", "din"):   ("00100010", "SUB"),
        ("OTURUPON", "pin"):   ("00100011", "DIV"),
        ("OTURUPON", "ku"):    ("00100100", "MOD"),
        ("IROSU", "so"):       ("11001100", "OUT"),
        ("IROSU", "gbo"):      ("11001101", "IN"),
        ("ODI", "si"):         ("10011111", "F_OPEN"),
        ("ODI", "ko"):         ("10011100", "F_WRITE"),
        ("ODI", "ka"):         ("10010110", "F_READ"),
        ("ODI", "pa"):         ("10010000", "F_CLOSE"),
        ("OTURA", "de"):       ("10111011", "BIND"),
        ("OTURA", "ran"):      ("10111111", "SEND"),
        ("OTURA", "gba"):      ("10110000", "RECV"),
        ("OSE", "ya"):         ("10101010", "DRAW"),
        ("OSE", "han"):        ("10100000", "RENDER"),
        ("OSE", "nu"):         ("10101111", "G_CLR"),
        ("OYEKU", "duro"):     ("00000000", "HALT"),
        ("OGBE", "bere"):      ("11111111", "INIT"),
        ("OGBE", "gbe"):       ("11111111", "INIT"),
        ("IWORI", "yipo"):     ("01100110", "LOOP"),
        ("OSA", "fo"):         ("01110111", "JUMP"),
        ("OSA", "ja"):         ("01111111", "BREAK"),
    }
    
    def __init__(self):
        self.variables = {}
        self.imports = set()
        self.bytecode = []
    
    def normalize(self, text):
        """Normalize Unicode for consistent matching."""
        return unicodedata.normalize('NFC', text)
    
    def compile(self, source):
        """Compile Ese source to bytecode."""
        source = self.normalize(source)
        lines = source.strip().split('\n')
        
        self.bytecode = []
        self.imports = set()
        
        for line_num, line in enumerate(lines, 1):
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            
            try:
                self._compile_line(line)
            except Exception as e:
                print(f"[ERROR] Line {line_num}: {e}")
                print(f"        {line}")
        
        # Auto-add HALT if not present
        if not self.bytecode or self.bytecode[-1][0] != "00000000":
            self.bytecode.append(("00000000", 0, "HALT"))
        
        return self.bytecode
    
    def _compile_line(self, line):
        """Compile a single line."""
        line = line.rstrip(';')
        
        # Import: ìbà Domain;
        if line.startswith('ìbà ') or line.startswith('iba '):
            domain = line.split()[1]
            domain_upper = self._normalize_domain(domain)
            self.imports.add(domain_upper)
            print(f"[IMPORT] {domain} → Domain loaded")
            return
        
        # Variable assignment: let x = 50;
        if '=' in line and not '.' in line.split('=')[0]:
            parts = line.split('=')
            var_name = parts[0].replace('let', '').strip()
            value = int(parts[1].strip())
            self.variables[var_name] = value
            self.bytecode.append(("11111111", value, f"INIT {var_name}"))
            return
        
        # Method call: Domain.ese(args)
        method_match = re.match(r'(\w+)\.(\w+)\(([^)]*)\)', line)
        if method_match:
            domain = method_match.group(1)
            method = method_match.group(2)
            args = method_match.group(3).strip()
            
            self._compile_method(domain, method, args)
            return
        
        # Simple keyword (ase, etc)
        if line in ('ase', 'àṣẹ'):
            self.bytecode.append(("00000000", 0, "HALT"))
            return
        
        print(f"[WARN] Unrecognized: {line}")
    
    def _normalize_domain(self, name):
        """Get uppercase domain key."""
        mappings = {
            'ogbè': 'OGBE', 'ogbe': 'OGBE',
            'ọ̀yẹ̀kú': 'OYEKU', 'oyeku': 'OYEKU',
            'ìwòrì': 'IWORI', 'iwori': 'IWORI',
            'òdí': 'ODI', 'odi': 'ODI',
            'ìrosù': 'IROSU', 'irosu': 'IROSU',
            'ọ̀wọ́nrín': 'OWONRIN', 'owonrin': 'OWONRIN',
            'ọ̀bàrà': 'OBARA', 'obara': 'OBARA',
            'ọ̀kànràn': 'OKANRAN', 'okanran': 'OKANRAN',
            'ògúndá': 'OGUNDA', 'ogunda': 'OGUNDA',
            'ọ̀sá': 'OSA', 'osa': 'OSA',
            'ìká': 'IKA', 'ika': 'IKA',
            'òtúúrúpọ̀n': 'OTURUPON', 'oturupon': 'OTURUPON',
            'òtúrá': 'OTURA', 'otura': 'OTURA',
            'ìrẹtẹ̀': 'IRETE', 'irete': 'IRETE',
            'ọ̀ṣẹ́': 'OSE', 'ose': 'OSE',
            'òfún': 'OFUN', 'ofun': 'OFUN',
        }
        return mappings.get(self.normalize(name.lower()), name.upper())
    
    def _compile_method(self, domain, method, args):
        """Compile a domain.method(args) call."""
        domain_key = self._normalize_domain(domain)
        method_key = self.normalize(method.lower())
        
        # Parse argument
        value = 0
        if args:
            args = args.strip('"\'')
            if args in self.variables:
                value = self.variables[args]
            elif args.isdigit():
                value = int(args)
        
        # Look up opcode
        key = (domain_key, method_key)
        if key in self.OPCODES:
            opcode, instr_name = self.OPCODES[key]
            self.bytecode.append((opcode, value, f"{instr_name}({value})"))
        else:
            # Generic mapping
            domain_obj = DOMAINS.get(domain_key)
            if domain_obj:
                self.bytecode.append((domain_obj.binary + "0000", value, 
                                     f"{domain_key}.{method}({value})"))
            else:
                print(f"[WARN] Unknown: {domain}.{method}")


# =============================================================================
# THE ESE RUNTIME
# =============================================================================
class EseRuntime:
    """Executes compiled Ese bytecode."""
    
    def __init__(self):
        self.registers = {"OKE": 0, "ISALE": 0, "OTUN": 0, "OSI": 0}
        self.screen = [[' ' for _ in range(16)] for _ in range(8)]
    
    def run(self, bytecode, verbose=True):
        """Execute bytecode."""
        if verbose:
            print("\n--- Ìbà: Execution Begins ---")
        
        for i, (opcode, value, desc) in enumerate(bytecode):
            if verbose:
                print(f"[{i:03d}] {desc}")
            
            # Execute based on opcode patterns
            if opcode == "11111111":  # INIT
                self.registers["ISALE"] = value
            elif opcode == "10001000":  # ADD
                self.registers["ISALE"] = (self.registers["ISALE"] + value) % 256
            elif opcode.startswith("0010"):  # Math
                if "SUB" in desc:
                    self.registers["ISALE"] = (self.registers["ISALE"] - value) % 256
                elif "DIV" in desc and value != 0:
                    self.registers["ISALE"] //= value
                elif "MOD" in desc and value != 0:
                    self.registers["ISALE"] %= value
            elif opcode.startswith("1100"):  # Output
                print(f"Output: {self.registers['ISALE']}")
            elif opcode.startswith("1010"):  # Graphics
                if "RENDER" in desc:
                    self._render_screen()
            elif opcode == "00000000":  # HALT
                if verbose:
                    print("[Ọ̀yẹ̀kú] Execution Complete.")
                break
        
        return self.registers["ISALE"]
    
    def _render_screen(self):
        """Render the graphics buffer."""
        print("\n+" + "-" * 16 + "+")
        for row in self.screen:
            print("|" + "".join(row) + "|")
        print("+" + "-" * 16 + "+")


# =============================================================================
# THE MAIN INTERFACE
# =============================================================================
def run_ese_file(filepath):
    """Compile and run an Ese file."""
    with open(filepath, 'r', encoding='utf-8') as f:
        source = f.read()
    
    print(f"=== Compiling: {filepath} ===")
    compiler = EseCompiler()
    bytecode = compiler.compile(source)
    
    print(f"\n=== Bytecode ({len(bytecode)} instructions) ===")
    for i, (op, val, desc) in enumerate(bytecode):
        print(f"  {i}: [{op}] {desc}")
    
    print("\n=== Execution ===")
    runtime = EseRuntime()
    result = runtime.run(bytecode)
    
    print(f"\nFinal ISALE: {result}")
    return result


# =============================================================================
# CLI & DEMO
# =============================================================================
if __name__ == "__main__":
    if len(sys.argv) > 1:
        run_ese_file(sys.argv[1])
    else:
        print("""
╔══════════════════════════════════════════════════════════════╗
║              IFA-LANG ESE COMPILER                           ║
╠══════════════════════════════════════════════════════════════╣
║  Usage: python ese_parser.py <filename.ifa>                  ║
╠══════════════════════════════════════════════════════════════╣
║  SYNTAX EXAMPLES:                                            ║
║    ìbà Òtúrá;           # Import network domain              ║
║    let x = 50;          # Variable assignment                ║
║    Obara.fikun(10);     # Add 10                             ║
║    Oturupon.din(5);     # Subtract 5                         ║
║    Irosu.so(x);         # Print output                       ║
║    Ose.ya(42);          # Draw character                     ║
║    ase;                 # End program                        ║
╚══════════════════════════════════════════════════════════════╝
""")
        
        # Demo: Compile inline example
        demo_source = """
# Demo: Math operations
ìbà Obara;
ìbà Oturupon;
ìbà Irosu;

let x = 50;
Obara.fikun(10);
Oturupon.din(5);
Oturupon.ku(3);
Irosu.so(x);
ase;
"""
        print("=== Demo Program ===")
        print(demo_source)
        
        compiler = EseCompiler()
        bytecode = compiler.compile(demo_source)
        
        print("=== Compiled Bytecode ===")
        for i, (op, val, desc) in enumerate(bytecode):
            print(f"  {i}: [{op}] {desc}")
        
        print("\n=== Execution ===")
        runtime = EseRuntime()
        runtime.run(bytecode)
