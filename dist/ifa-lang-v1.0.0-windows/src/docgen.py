# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           IFÁ DOCUMENTATION GENERATOR - THE CORPUS WRITER                    ║
║                    Generates Ifá Corpus-style HTML Documentation             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Usage: ifa doc [directory]                                                  ║
║  Output: Static HTML website organized by Odù domains                        ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import re
from datetime import datetime
from typing import Dict, List, Optional
from dataclasses import dataclass, field


# =============================================================================
# DATA STRUCTURES
# =============================================================================

@dataclass
class Ese:
    """A single verse (method/function) in the Ifá Corpus."""
    name: str
    description: str
    params: List[str] = field(default_factory=list)
    returns: str = ""
    example: str = ""
    line_number: int = 0


@dataclass
class OduChapter:
    """A chapter (class/module) within an Odù domain."""
    name: str
    odu: str  # The Odù this belongs to
    description: str
    verses: List[Ese] = field(default_factory=list)
    source_file: str = ""


@dataclass 
class OduBook:
    """A complete Odù book with all its chapters."""
    name: str
    binary: str
    meaning: str
    chapters: List[OduChapter] = field(default_factory=list)


# =============================================================================
# ODÙ MEANINGS (for documentation)
# =============================================================================

ODU_MEANINGS = {
    "OGBE": ("1111", "The Light", "System initialization, beginnings, CLI arguments"),
    "OYEKU": ("0000", "The Darkness", "Process termination, endings, sleep"),
    "IWORI": ("0110", "The Mirror", "Reflection, time, iteration, loops"),
    "ODI": ("1001", "The Vessel", "Storage, file operations, containment"),
    "IROSU": ("1100", "The Speaker", "Communication, console I/O, expression"),
    "OWONRIN": ("0011", "The Chaotic", "Randomness, chance, unpredictability"),
    "OBARA": ("1000", "The King", "Expansion, addition, multiplication"),
    "OKANRAN": ("0001", "The Troublemaker", "Errors, exceptions, warnings"),
    "OGUNDA": ("1110", "The Cutter", "Arrays, process control, separation"),
    "OSA": ("0111", "The Wind", "Control flow, jumps, conditionals"),
    "IKA": ("0100", "The Constrictor", "Strings, compression, binding"),
    "OTURUPON": ("0010", "The Bearer", "Reduction, subtraction, division"),
    "OTURA": ("1011", "The Messenger", "Network, communication, sending"),
    "IRETE": ("1101", "The Crusher", "Memory management, garbage collection"),
    "OSE": ("1010", "The Beautifier", "Graphics, display, aesthetics"),
    "OFUN": ("0101", "The Creator", "Object creation, inheritance"),
}


# =============================================================================
# PARSER - Extract documentation from .ifa files
# =============================================================================

class IfaDocParser:
    """Parses .ifa files to extract documentation."""
    
    def __init__(self):
        self.books: Dict[str, OduBook] = {}
        self._init_books()
    
    def _init_books(self):
        """Initialize all 16 Odù books."""
        for name, (binary, meaning, desc) in ODU_MEANINGS.items():
            self.books[name] = OduBook(name=name, binary=binary, meaning=meaning, chapters=[])
    
    def parse_file(self, filepath: str) -> List[OduChapter]:
        """Parse a single .ifa file for documentation."""
        chapters = []
        
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
            lines = content.split('\n')
        
        current_chapter = None
        current_description = []
        in_doc_block = False
        
        for i, line in enumerate(lines):
            stripped = line.strip()
            
            # Doc comment block (# or """)
            if stripped.startswith('# '):
                current_description.append(stripped[2:])
                continue
            
            # Import statement - determines Odù domain
            if stripped.startswith('ìbà ') or stripped.startswith('iba '):
                domain = stripped.split()[1].rstrip(';').upper()
                if domain in self.books:
                    current_chapter = OduChapter(
                        name=os.path.basename(filepath),
                        odu=domain,
                        description='\n'.join(current_description),
                        source_file=filepath
                    )
                    chapters.append(current_chapter)
                current_description = []
            
            # Method call - Domain.method()
            method_match = re.match(r'(\w+)\.(\w+)\((.*?)\)', stripped)
            if method_match and current_chapter:
                domain, method, args = method_match.groups()
                verse = Ese(
                    name=f"{domain}.{method}",
                    description=' '.join(current_description) if current_description else f"Calls {method} on {domain}",
                    params=[a.strip() for a in args.split(',') if a.strip()],
                    line_number=i + 1
                )
                current_chapter.verses.append(verse)
                current_description = []
        
        return chapters
    
    def parse_python_file(self, filepath: str) -> List[OduChapter]:
        """Parse a Python Standard Library file for documentation."""
        chapters = []
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
            lines = content.split('\n')
            
        current_chapter = None
        
        # Regex for OduModule class
        class_pattern = re.compile(r'class\s+(\w+)\s*\(OduModule\):')
        
        # Regex for super().__init__("Name", "Binary", "Desc")
        init_pattern = re.compile(r'super\(\)\.__init__\(\s*["\']([^"\']+)["\']\s*,\s*["\']([^"\']+)["\']\s*,\s*["\']([^"\']+)["\']')
        
        # Regex for _register("name", self.func, "desc")
        register_pattern = re.compile(r'self\._register\(\s*["\']([^"\']+)["\']\s*,\s*self\.(\w+)\s*,\s*["\']([^"\']+)["\']')
        
        # Regex for function definitions: def func_name(self, param1, param2, ...) -> type:
        def_pattern = re.compile(r'def\s+(\w+)\s*\(\s*self\s*(?:,\s*(.+?))?\s*\)\s*(?:->\s*[\w\[\], ]+)?:')
        
        # First pass: Build a map of function definitions with params and line numbers
        func_info = {}  # {func_name: {'params': [...], 'line': N}}
        for i, line in enumerate(lines, 1):
            def_match = def_pattern.match(line.strip())
            if def_match:
                func_name = def_match.group(1)
                params_str = def_match.group(2) if def_match.group(2) else ""
                
                # Parse parameters, removing type hints and defaults for cleaner display
                params = []
                if params_str:
                    for param in params_str.split(','):
                        param = param.strip()
                        # Extract just the parameter name (before : or =)
                        param_name = re.split(r'[:=]', param)[0].strip()
                        if param_name and param_name != 'self':
                            # Include type hint if present for better docs
                            if ':' in param:
                                type_hint = param.split(':')[1].split('=')[0].strip()
                                params.append(f"{param_name}: {type_hint}")
                            else:
                                params.append(param_name)
                
                func_info[func_name] = {'params': params, 'line': i}
        
        domain_name = None
        domain_binary = None
        domain_desc = None
        
        # Second pass: Extract registrations and match to function definitions
        for i, line in enumerate(lines, 1):
            stripped = line.strip()
            
            # Check for module init details
            init_match = init_pattern.search(stripped)
            if init_match:
                domain_name, domain_binary, domain_desc = init_match.groups()
                for key, val in ODU_MEANINGS.items():
                    if val[0] == domain_binary:
                        current_chapter = OduChapter(
                            name=domain_name,
                            odu=key,
                            description=domain_desc,
                            source_file=filepath
                        )
                        chapters.append(current_chapter)
                        break
                continue

            # Check for function registration
            reg_match = register_pattern.search(stripped)
            if reg_match and current_chapter:
                func_name, py_func, func_desc = reg_match.groups()
                
                # Look up the actual function definition for params and line number
                info = func_info.get(py_func, {})
                params = info.get('params', [])
                line_num = info.get('line', i)  # Fall back to registration line if def not found
                
                # Handle special cases like hash_ -> hash
                if not params and py_func.endswith('_'):
                    info = func_info.get(py_func, {})
                    params = info.get('params', [])
                    line_num = info.get('line', i)
                
                verse = Ese(
                    name=func_name,
                    description=func_desc,
                    params=params if params else ["none"],
                    line_number=line_num
                )
                current_chapter.verses.append(verse)
                
        return chapters

    def parse_directory(self, directory: str) -> Dict[str, OduBook]:
        """Parse all .ifa and .py files in a directory."""
        for root, dirs, files in os.walk(directory):
            for file in files:
                filepath = os.path.join(root, file)
                if file.endswith('.ifa'):
                    chapters = self.parse_file(filepath)
                    for chapter in chapters:
                        if chapter.odu in self.books:
                            self.books[chapter.odu].chapters.append(chapter)
                elif file.endswith('.py'):
                    # Only try parsing if it looks like an OduModule
                    with open(filepath, 'r', encoding='utf-8') as f:
                        if 'OduModule' in f.read(1024): # Quick check
                            chapters = self.parse_python_file(filepath)
                            for chapter in chapters:
                                if chapter.odu in self.books:
                                    self.books[chapter.odu].chapters.append(chapter)
        
        return self.books


# =============================================================================
# HTML GENERATOR - Creates the Ifá Corpus website
# =============================================================================

class IfaDocGenerator:
    """Generates HTML documentation in Ifá Corpus style."""
    
    CSS = """
    :root {
        --bg-dark: #1a1a2e;
        --bg-card: #16213e;
        --accent: #e94560;
        --gold: #ffd700;
        --text: #eaeaea;
        --text-dim: #a0a0a0;
    }
    
    * { box-sizing: border-box; margin: 0; padding: 0; }
    
    body {
        font-family: 'Segoe UI', system-ui, sans-serif;
        background: var(--bg-dark);
        color: var(--text);
        line-height: 1.6;
    }
    
    .container { max-width: 1200px; margin: 0 auto; padding: 2rem; }
    
    header {
        text-align: center;
        padding: 3rem 0;
        border-bottom: 2px solid var(--accent);
        margin-bottom: 2rem;
    }
    
    h1 {
        font-size: 3rem;
        background: linear-gradient(135deg, var(--gold), var(--accent));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        margin-bottom: 0.5rem;
    }
    
    h2 {
        color: var(--gold);
        border-bottom: 1px solid var(--accent);
        padding-bottom: 0.5rem;
        margin: 2rem 0 1rem;
    }
    
    h3 { color: var(--accent); margin: 1.5rem 0 0.5rem; }
    
    .odu-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
        gap: 1.5rem;
        margin: 2rem 0;
    }
    
    .odu-card {
        background: var(--bg-card);
        border-radius: 12px;
        padding: 1.5rem;
        border: 1px solid rgba(233, 69, 96, 0.3);
        transition: transform 0.2s, box-shadow 0.2s;
    }
    
    .odu-card:hover {
        transform: translateY(-4px);
        box-shadow: 0 8px 24px rgba(233, 69, 96, 0.2);
    }
    
    .odu-card h3 {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    
    .binary {
        font-family: monospace;
        font-size: 0.8rem;
        color: var(--gold);
        background: rgba(255, 215, 0, 0.1);
        padding: 2px 6px;
        border-radius: 4px;
    }
    
    .meaning { color: var(--text-dim); font-style: italic; }
    
    .verse {
        background: rgba(0, 0, 0, 0.2);
        border-left: 3px solid var(--accent);
        padding: 1rem;
        margin: 1rem 0;
        border-radius: 0 8px 8px 0;
    }
    
    .verse-name {
        font-family: monospace;
        color: var(--gold);
        font-size: 1.1rem;
    }
    
    .verse-desc { margin-top: 0.5rem; }
    
    code {
        background: rgba(255, 215, 0, 0.1);
        padding: 2px 6px;
        border-radius: 4px;
        font-family: 'Consolas', monospace;
    }
    
    .nav {
        position: fixed;
        top: 0;
        left: 0;
        width: 250px;
        height: 100vh;
        background: var(--bg-card);
        padding: 2rem 1rem;
        overflow-y: auto;
        border-right: 1px solid var(--accent);
    }
    
    .nav a {
        display: block;
        color: var(--text);
        text-decoration: none;
        padding: 0.5rem;
        border-radius: 4px;
        transition: background 0.2s;
    }
    
    .nav a:hover { background: rgba(233, 69, 96, 0.2); }
    
    .main-content { margin-left: 270px; }
    
    footer {
        text-align: center;
        padding: 2rem;
        color: var(--text-dim);
        border-top: 1px solid var(--accent);
        margin-top: 3rem;
    }
    
    @media (max-width: 768px) {
        .nav { display: none; }
        .main-content { margin-left: 0; }
    }
    """
    
    def __init__(self, books: Dict[str, OduBook]):
        self.books = books
    
    def generate_index(self) -> str:
        """Generate the main index page."""
        cards = []
        for name, book in sorted(self.books.items()):
            chapter_count = len(book.chapters)
            verse_count = sum(len(c.verses) for c in book.chapters)
            cards.append(f'''
            <a href="{name.lower()}.html" class="odu-card">
                <h3><span class="binary">{book.binary}</span> {name}</h3>
                <p class="meaning">{book.meaning}</p>
                <p>{chapter_count} chapters, {verse_count} verses</p>
            </a>
            ''')
        
        return f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ifá-Lang Corpus</title>
    <style>{self.CSS}</style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🔮 The Ifá Corpus</h1>
            <p>Documentation for Ifá-Lang</p>
            <p style="color: var(--text-dim);">Generated {datetime.now().strftime('%Y-%m-%d %H:%M')}</p>
        </header>
        
        <h2>The 16 Odù Domains</h2>
        <div class="odu-grid">
            {''.join(cards)}
        </div>
        
        <h2>📦 Ọjà - The Market (Package Manager)</h2>
        <div class="verse">
            <div class="verse-name">Install Libraries from Git</div>
            <div class="verse-desc">
                <p>Ọjà is the decentralized package manager for Ifá-Lang. Install libraries directly from any Git repository:</p>
                <p><code>ifa oja add https://github.com/user/my-library.git</code></p>
            </div>
        </div>
        
        <div class="odu-grid">
            <div class="odu-card">
                <h3>📥 ra (Buy/Add)</h3>
                <p><code>ifa oja add &lt;url&gt;</code></p>
                <p class="meaning">Download & add dependency</p>
            </div>
            <div class="odu-card">
                <h3>📤 ta (Sell/Remove)</h3>
                <p><code>ifa oja remove &lt;name&gt;</code></p>
                <p class="meaning">Remove a dependency</p>
            </div>
            <div class="odu-card">
                <h3>🔄 install</h3>
                <p><code>ifa oja install</code></p>
                <p class="meaning">Sync all dependencies</p>
            </div>
            <div class="odu-card">
                <h3>🔐 lock</h3>
                <p><code>ifa oja lock</code></p>
                <p class="meaning">Generate checksums</p>
            </div>
        </div>
        
        <h2>🚀 Quick Start</h2>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>▶️ Run</h3>
                <p><code>ifa run program.ifa</code></p>
                <p class="meaning">Interpreted execution</p>
            </div>
            <div class="odu-card">
                <h3>🔨 Build</h3>
                <p><code>ifa build program.ifa -o app</code></p>
                <p class="meaning">Compile to native binary</p>
            </div>
            <div class="odu-card">
                <h3>🔍 Check</h3>
                <p><code>ifa check program.ifa</code></p>
                <p class="meaning">Validate balance (Ìwà)</p>
            </div>
            <div class="odu-card">
                <h3>📚 Docs</h3>
                <p><code>ifa doc lib/ docs/</code></p>
                <p class="meaning">Generate documentation</p>
            </div>
        </div>
        
        <h2>⚡ Bytecode Execution</h2>
        <p style="color: var(--text-dim); margin-bottom: 1rem;">Compile to optimized .ifab bytecode for fast startup and IoT/embedded systems:</p>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>Compile</h3>
                <p><code>ifa bytecode program.ifa</code></p>
                <p class="meaning">Generate .ifab bytecode</p>
            </div>
            <div class="odu-card">
                <h3>Run Bytecode</h3>
                <p><code>ifa runb program.ifab</code></p>
                <p class="meaning">Execute bytecode (fast)</p>
            </div>
            <div class="odu-card">
                <h3>Disassemble</h3>
                <p><code>ifa disasm program.ifab</code></p>
                <p class="meaning">View bytecode instructions</p>
            </div>
            <div class="odu-card">
                <h3>8-bit ISA</h3>
                <p><code>256 Amulu instructions</code></p>
                <p class="meaning">16 Verbs × 16 Nouns</p>
            </div>
        </div>
        
        <h2>📝 Language Syntax</h2>
        <h3>Variables</h3>
        <div class="verse">
            <div class="verse-name">Declaration</div>
            <div class="verse-desc">
                <code>ayanmọ x = 50;</code> (Yoruba) or <code>let x = 50;</code> (English)
            </div>
        </div>
        
        <h3>Control Flow</h3>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>If/Else</h3>
                <p><code>ti x &gt; 5 {{ ... }} bibẹkọ {{ ... }}</code></p>
                <p class="meaning">or: if/else</p>
            </div>
            <div class="odu-card">
                <h3>While Loop</h3>
                <p><code>nigba x &lt; 10 {{ ... }}</code></p>
                <p class="meaning">or: while</p>
            </div>
            <div class="odu-card">
                <h3>For Loop</h3>
                <p><code>fun item ninu list {{ ... }}</code></p>
                <p class="meaning">or: for...in</p>
            </div>
            <div class="odu-card">
                <h3>Match</h3>
                <p><code>yàn (x) {{ 1 =&gt; ... }}</code></p>
                <p class="meaning">or: match/select</p>
            </div>
        </div>
        
        <h3>Functions & Classes</h3>
        <div class="verse">
            <div class="verse-name">Function (Ese/Verse)</div>
            <div class="verse-desc">
                <code>ese greet(name) {{ Irosu.fo("Hello " + name); }}</code>
            </div>
        </div>
        <div class="verse">
            <div class="verse-name">Class (Odù/Domain)</div>
            <div class="verse-desc">
                <code>odù Calculator {{ ayanmọ value = 0; ese add(n) {{ value = value + n; }} }}</code>
            </div>
        </div>
        
        <h2>🌍 Dual Lexicon (Yoruba ↔ English)</h2>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>ìbà</h3>
                <p class="meaning">import, respect</p>
            </div>
            <div class="odu-card">
                <h3>ayanmọ</h3>
                <p class="meaning">let, var, destiny</p>
            </div>
            <div class="odu-card">
                <h3>ese / ẹsẹ</h3>
                <p class="meaning">fn, def, verse</p>
            </div>
            <div class="odu-card">
                <h3>odù</h3>
                <p class="meaning">class, domain, module</p>
            </div>
            <div class="odu-card">
                <h3>ti / bí</h3>
                <p class="meaning">if, divine</p>
            </div>
            <div class="odu-card">
                <h3>bibẹkọ</h3>
                <p class="meaning">else, otherwise</p>
            </div>
            <div class="odu-card">
                <h3>nigba</h3>
                <p class="meaning">while, cycle</p>
            </div>
            <div class="odu-card">
                <h3>padà</h3>
                <p class="meaning">return</p>
            </div>
            <div class="odu-card">
                <h3>àṣẹ</h3>
                <p class="meaning">end</p>
            </div>
            <div class="odu-card">
                <h3>òótọ́ / irọ́</h3>
                <p class="meaning">true / false</p>
            </div>
        </div>
        
        <h2>💡 Code Examples</h2>
        <div class="verse">
            <div class="verse-name">Hello World</div>
            <div class="verse-desc">
                <pre style="background: rgba(0,0,0,0.3); padding: 1rem; border-radius: 8px; overflow-x:auto;">ìbà Irosu;
Irosu.fo("Ẹ kú àbọ̀ sí Ifá-Lang!");
àṣẹ;</pre>
            </div>
        </div>
        <div class="verse">
            <div class="verse-name">Math Operations</div>
            <div class="verse-desc">
                <pre style="background: rgba(0,0,0,0.3); padding: 1rem; border-radius: 8px; overflow-x:auto;">ayanmọ x = Obara.fikun(10, 5);   // Add: 15
ayanmọ y = Oturupon.din(10, 3);  // Subtract: 7
Irosu.fo(x + y);                 // Print: 22</pre>
            </div>
        </div>
        <div class="verse">
            <div class="verse-name">Network (Ether)</div>
            <div class="verse-desc">
                <pre style="background: rgba(0,0,0,0.3); padding: 1rem; border-radius: 8px; overflow-x:auto;">Otura.ether_de(1);    // Join channel 1
Otura.ether_ran(42);  // Broadcast value
ayanmọ msg = Otura.ether_gba();  // Receive
Otura.ether_pa();     // Leave network</pre>
            </div>
        </div>
        
        <h2>⚖️ Ìwà-Pẹ̀lẹ́ (Balance Rules)</h2>
        <p style="color: var(--text-dim); margin-bottom: 1rem;">Every resource opened must be closed. The Ìwà Engine enforces good character:</p>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>📁 Files</h3>
                <p><code>Odi.si()</code> → <code>Odi.pa()</code></p>
                <p class="meaning">Open → Close</p>
            </div>
            <div class="odu-card">
                <h3>🌐 Network</h3>
                <p><code>Otura.ether_de()</code> → <code>Otura.ether_pa()</code></p>
                <p class="meaning">Join → Leave</p>
            </div>
            <div class="odu-card">
                <h3>🔒 Locks</h3>
                <p><code>Osa.khoa()</code> → <code>Osa.si()</code></p>
                <p class="meaning">Lock → Unlock</p>
            </div>
            <div class="odu-card">
                <h3>📦 Arrays</h3>
                <p><code>Ogunda.ge()</code> → <code>Irete.tu()</code></p>
                <p class="meaning">Create → Release</p>
            </div>
        </div>
        
        <h2>💻 CLI Reference</h2>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>run</h3>
                <p><code>ifa run &lt;file&gt;</code></p>
                <p class="meaning">Execute interpreted</p>
            </div>
            <div class="odu-card">
                <h3>build</h3>
                <p><code>ifa build &lt;file&gt; -o &lt;out&gt;</code></p>
                <p class="meaning">Compile to Rust binary</p>
            </div>
            <div class="odu-card">
                <h3>check</h3>
                <p><code>ifa check &lt;file&gt;</code></p>
                <p class="meaning">Validate Ìwà balance</p>
            </div>
            <div class="odu-card">
                <h3>repl</h3>
                <p><code>ifa repl</code></p>
                <p class="meaning">Interactive shell</p>
            </div>
            <div class="odu-card">
                <h3>doc</h3>
                <p><code>ifa doc &lt;src&gt; &lt;out&gt;</code></p>
                <p class="meaning">Generate HTML docs</p>
            </div>
            <div class="odu-card">
                <h3>lsp</h3>
                <p><code>ifa lsp --stdio</code></p>
                <p class="meaning">Language Server</p>
            </div>
        </div>
        
        <h2>🔮 Babalawo Error Guide</h2>
        <p style="color: var(--text-dim); margin-bottom: 1rem;">Errors come with proverbs and wisdom:</p>
        <div class="verse">
            <div class="verse-name">OTURUPON-OYEKU (Division by Zero)</div>
            <div class="verse-desc">"One cannot carry a load that does not exist."</div>
        </div>
        <div class="verse">
            <div class="verse-name">ODI-OKANRAN (File Not Found)</div>
            <div class="verse-desc">"The calabash cannot hold what was never placed inside."</div>
        </div>
        <div class="verse">
            <div class="verse-name">OTURA-OYEKU (Connection Failed)</div>
            <div class="verse-desc">"The messenger cannot deliver to a house that does not exist."</div>
        </div>
        
        <h2>🎨 VS Code Extension (Ilé Ìwé)</h2>
        <p style="color: var(--text-dim); margin-bottom: 1rem;">Full IDE support for Ifá-Lang development:</p>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>🌈 Syntax Highlighting</h3>
                <p class="meaning">Colors for Odù, keywords, strings</p>
            </div>
            <div class="odu-card">
                <h3>💡 Intellisense</h3>
                <p class="meaning">Autocomplete for all 16 domains</p>
            </div>
            <div class="odu-card">
                <h3>🐛 Debugging</h3>
                <p class="meaning">Breakpoints, stepping, inspection</p>
            </div>
            <div class="odu-card">
                <h3>⚡ Diagnostics</h3>
                <p class="meaning">Real-time error checking</p>
            </div>
        </div>
        <div class="verse">
            <div class="verse-name">Installation</div>
            <div class="verse-desc">
                <code>cd vscode_extension && npm install && code --extensionDevelopmentPath=.</code>
            </div>
        </div>
        
        <footer>
            <p>Àṣẹ! - Generated by Ifá-Lang Documentation System</p>
            <p style="margin-top: 0.5rem;"><a href="https://github.com/AAEO04/ifa-lang" style="color: var(--gold);">GitHub Repository</a></p>
        </footer>
    </div>
</body>
</html>'''
    
    def generate_odu_page(self, book: OduBook) -> str:
        """Generate a page for a single Odù domain."""
        chapters_html = []
        
        for chapter in book.chapters:
            verses_html = []
            for verse in chapter.verses:
                params = ', '.join(verse.params) if verse.params else 'none'
                verses_html.append(f'''
                <div class="verse">
                    <div class="verse-name">{verse.name}()</div>
                    <div class="verse-desc">{verse.description}</div>
                    <p><small>Parameters: <code>{params}</code> | Line {verse.line_number}</small></p>
                </div>
                ''')
            
            chapters_html.append(f'''
            <h3>📜 {chapter.name}</h3>
            <p>{chapter.description}</p>
            <p><small>Source: {chapter.source_file}</small></p>
            {''.join(verses_html)}
            ''')
        
        nav_links = '\n'.join([
            f'<a href="{name.lower()}.html">{name}</a>'
            for name in sorted(self.books.keys())
        ])
        
        return f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{book.name} - Ifá Corpus</title>
    <style>{self.CSS}</style>
</head>
<body>
    <nav class="nav">
        <h3><a href="index.html">🔮 Ifá Corpus</a></h3>
        <hr style="border-color: var(--accent); margin: 1rem 0;">
        {nav_links}
    </nav>
    
    <div class="main-content">
        <div class="container">
            <header>
                <h1><span class="binary">{book.binary}</span> {book.name}</h1>
                <p class="meaning">{book.meaning}</p>
            </header>
            
            <h2>Chapters ({len(book.chapters)})</h2>
            {(''.join(chapters_html)) if chapters_html else '<p>No chapters documented yet.</p>'}
            
            <footer>
                <p><a href="index.html">← Back to Index</a></p>
            </footer>
        </div>
    </div>
</body>
</html>'''
    
    def generate(self, output_dir: str):
        """Generate the complete documentation website."""
        os.makedirs(output_dir, exist_ok=True)
        
        # Write index
        with open(os.path.join(output_dir, 'index.html'), 'w', encoding='utf-8') as f:
            f.write(self.generate_index())
        
        # Write each Odù page
        for name, book in self.books.items():
            with open(os.path.join(output_dir, f'{name.lower()}.html'), 'w', encoding='utf-8') as f:
                f.write(self.generate_odu_page(book))
        
        print(f"[Ifá Doc] Generated documentation in: {output_dir}")
        print(f"[Ifá Doc] Open {os.path.join(output_dir, 'index.html')} to view")


# =============================================================================
# CLI ENTRY POINT
# =============================================================================

def generate_docs(source_dir: str = ".", output_dir: str = "docs"):
    """Main entry point for documentation generation."""
    print(f"\n=== Ifá Documentation Generator ===")
    print(f"Scanning: {source_dir}")
    
    parser = IfaDocParser()
    books = parser.parse_directory(source_dir)
    
    total_chapters = sum(len(b.chapters) for b in books.values())
    total_verses = sum(sum(len(c.verses) for c in b.chapters) for b in books.values())
    
    print(f"Found: {total_chapters} chapters, {total_verses} verses")
    
    generator = IfaDocGenerator(books)
    generator.generate(output_dir)
    
    return output_dir


if __name__ == "__main__":
    import sys
    source = sys.argv[1] if len(sys.argv) > 1 else "."
    output = sys.argv[2] if len(sys.argv) > 2 else "docs"
    generate_docs(source, output)
