//! # Ifá Documentation Generator (DocGen)
//!
//! Generates HTML documentation for Ifá-Lang projects in the style of the Ifá Corpus.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use color_eyre::eyre::Result;
use chrono::Local;

/// Odù domain metadata with ASCII slug for filenames
pub struct OduInfo {
    pub name: &'static str,
    pub slug: &'static str,  // ASCII-only for filenames
    pub alias: &'static str, // English alias (Log, Math, etc.)
    pub binary: &'static str,
    pub meaning: &'static str,
    pub description: &'static str,
}

/// The 16 Odù domains with their meanings
pub const ODU_DOMAINS: &[OduInfo] = &[
    OduInfo { name: "Ogbè", slug: "ogbe", alias: "System", binary: "1111", meaning: "The Light", description: "System initialization, beginnings, CLI arguments" },
    OduInfo { name: "Ọ̀yẹ̀kú", slug: "oyeku", alias: "Exit", binary: "0000", meaning: "The Darkness", description: "Process termination, endings, sleep" },
    OduInfo { name: "Ìwòrì", slug: "iwori", alias: "Time", binary: "0110", meaning: "The Mirror", description: "Reflection, time, iteration, loops" },
    OduInfo { name: "Òdí", slug: "odi", alias: "File", binary: "1001", meaning: "The Vessel", description: "Storage, file operations, containment" },
    OduInfo { name: "Ìrosù", slug: "irosu", alias: "Log", binary: "1100", meaning: "The Speaker", description: "Communication, console I/O, expression" },
    OduInfo { name: "Ọ̀wọ́nrín", slug: "owonrin", alias: "Random", binary: "0011", meaning: "The Chaotic", description: "Randomness, chance, unpredictability" },
    OduInfo { name: "Ọ̀bàrà", slug: "obara", alias: "Math", binary: "1000", meaning: "The King", description: "Expansion, addition, multiplication" },
    OduInfo { name: "Ọ̀kànràn", slug: "okanran", alias: "Error", binary: "0001", meaning: "The Troublemaker", description: "Errors, exceptions, warnings" },
    OduInfo { name: "Ògúndá", slug: "ogunda", alias: "Array", binary: "1110", meaning: "The Cutter", description: "Arrays, process control, separation" },
    OduInfo { name: "Ọ̀sá", slug: "osa", alias: "Flow", binary: "0111", meaning: "The Wind", description: "Control flow, jumps, conditionals" },
    OduInfo { name: "Ìká", slug: "ika", alias: "String", binary: "0100", meaning: "The Constrictor", description: "Strings, compression, binding" },
    OduInfo { name: "Òtúúrúpọ̀n", slug: "oturupon", alias: "Reduce", binary: "0010", meaning: "The Bearer", description: "Reduction, subtraction, division" },
    OduInfo { name: "Òtúrá", slug: "otura", alias: "Net", binary: "1011", meaning: "The Messenger", description: "Network, communication, sending" },
    OduInfo { name: "Ìrẹtẹ̀", slug: "irete", alias: "Crypto", binary: "1101", meaning: "The Crusher", description: "Memory management, garbage collection" },
    OduInfo { name: "Ọ̀ṣẹ́", slug: "ose", alias: "UI", binary: "1010", meaning: "The Beautifier", description: "Graphics, display, aesthetics" },
    OduInfo { name: "Òfún", slug: "ofun", alias: "Root", binary: "0101", meaning: "The Creator", description: "Object creation, inheritance" },
];

/// CSS for the documentation site
const CSS: &str = r#"
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
    background-clip: text;
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
    text-decoration: none;
    color: inherit;
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

pre {
    background: rgba(0,0,0,0.3);
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
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
"#;

/// Generate the main index.html page
pub fn generate_index_html() -> String {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();
    
    let mut odu_cards = String::new();
    for odu in ODU_DOMAINS {
        odu_cards.push_str(&format!(r#"
            <a href="{slug}.html" class="odu-card">
                <h3><span class="binary">{binary}</span> {name} <span class="meaning">({alias})</span></h3>
                <p class="meaning">{meaning}</p>
                <p>{desc}</p>
            </a>
        "#,
            slug = odu.slug,
            binary = odu.binary,
            name = odu.name,
            alias = odu.alias,
            meaning = odu.meaning,
            desc = odu.description
        ));
    }

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ifá-Lang Documentation</title>
    <style>{css}</style>
</head>
<body>
    <div class="container">
        <header>
            <h1>The Ifá Corpus</h1>
            <p>Documentation for Ifá-Lang - The Yoruba Programming Language</p>
            <p style="color: var(--text-dim);">Generated {timestamp}</p>
        </header>
        
        <h2>The 16 Odù Domains</h2>
        <div class="odu-grid">
            {odu_cards}
        </div>
        
        <h2>Quick Start</h2>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>Run</h3>
                <p><code>ifa run program.ifa</code></p>
                <p class="meaning">Interpreted execution</p>
            </div>
            <div class="odu-card">
                <h3>Build</h3>
                <p><code>ifa build program.ifa -o app</code></p>
                <p class="meaning">Compile to native binary</p>
            </div>
            <div class="odu-card">
                <h3>Check</h3>
                <p><code>ifa check program.ifa</code></p>
                <p class="meaning">Validate syntax</p>
            </div>
            <div class="odu-card">
                <h3>Docs</h3>
                <p><code>ifa doc src/ -o docs/</code></p>
                <p class="meaning">Generate documentation</p>
            </div>
        </div>

        <h2>Bytecode Execution</h2>
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
        </div>

        <h2>Native Compilation</h2>
        <p style="color: var(--text-dim); margin-bottom: 1rem;">Compile to standalone executable using Rust:</p>
        <div class="verse">
            <div class="verse-name">Build Command</div>
            <div class="verse-desc">
                <pre>ifa build myapp.ifa -o myapp
./myapp.exe  # Windows
./myapp      # Linux/Mac</pre>
            </div>
        </div>

        <h2>Interactive Tools</h2>
        <div class="odu-grid">
            <a href="playground.html" class="odu-card">
                <h3>Playground</h3>
                <p>Interactive code editor with examples</p>
                <p class="meaning">Try Ifá-Lang in your browser</p>
            </a>
            <a href="sandbox.html" class="odu-card">
                <h3>Ìgbálẹ̀ Sandbox</h3>
                <p>Sandboxed execution documentation</p>
                <p class="meaning">Security & isolation features</p>
            </a>
        </div>

        <h2>Domain Stacks</h2>
        <p style="color: var(--text-dim); margin-bottom: 1rem;">Pre-built libraries for common application domains:</p>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>Frontend</h3>
                <p>HTML/CSS generation, components</p>
                <p class="meaning">ifa-std/stacks/frontend</p>
            </div>
            <div class="odu-card">
                <h3>Backend</h3>
                <p>HTTP servers, routing, middleware</p>
                <p class="meaning">ifa-std/stacks/backend</p>
            </div>
            <div class="odu-card">
                <h3>Crypto</h3>
                <p>Encryption, hashing, signatures</p>
                <p class="meaning">ifa-std/stacks/crypto</p>
            </div>
            <div class="odu-card">
                <h3>GameDev</h3>
                <p>Sprites, physics, game loop</p>
                <p class="meaning">ifa-std/stacks/gamedev</p>
            </div>
            <div class="odu-card">
                <h3>IoT</h3>
                <p>Sensors, GPIO, embedded systems</p>
                <p class="meaning">ifa-std/stacks/iot</p>
            </div>
            <div class="odu-card">
                <h3>ML</h3>
                <p>Machine learning, neural networks</p>
                <p class="meaning">ifa-std/stacks/ml</p>
            </div>
        </div>

        <h2>Opele Chain & Divination</h2>
        <p style="color: var(--text-dim); margin-bottom: 1rem;">Unique Ifá-Lang features inspired by the Ifá divination system:</p>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>OpeleChain</h3>
                <p>Tamper-evident, append-only log</p>
                <p class="meaning">Blockchain-like audit trail</p>
            </div>
            <div class="odu-card">
                <h3>Opele.cast()</h3>
                <p>Random Odù selection for divination</p>
                <p class="meaning">Cryptographically secure RNG</p>
            </div>
            <div class="odu-card">
                <h3>Odù Patterns</h3>
                <p>16 binary patterns (0000-1111)</p>
                <p class="meaning">Pattern matching & wisdom</p>
            </div>
            <div class="odu-card">
                <h3>Ìwà Balance</h3>
                <p>Resource lifecycle management</p>
                <p class="meaning">Open/Close, Acquire/Release</p>
            </div>
        </div>

        <h2>Language Syntax</h2>
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
                <p><code>ti x > 5 {{ ... }} bibẹkọ {{ ... }}</code></p>
                <p class="meaning">or: if/else</p>
            </div>
            <div class="odu-card">
                <h3>While Loop</h3>
                <p><code>nigba x < 10 {{ ... }}</code></p>
                <p class="meaning">or: while</p>
            </div>
            <div class="odu-card">
                <h3>For Loop</h3>
                <p><code>fun item ninu list {{ ... }}</code></p>
                <p class="meaning">or: for...in</p>
            </div>
            <div class="odu-card">
                <h3>Match</h3>
                <p><code>yàn (x) {{ 1 => ... }}</code></p>
                <p class="meaning">or: match/select</p>
            </div>
        </div>

        <h3>Functions & Classes</h3>
        <div class="verse">
            <div class="verse-name">Function (Ese/Verse)</div>
            <div class="verse-desc">
                <pre>ese greet(name) {{
    Irosu.fo("Hello " + name);
}}</pre>
            </div>
        </div>
        <div class="verse">
            <div class="verse-name">Class (Odù/Domain)</div>
            <div class="verse-desc">
                <pre>odù Calculator {{
    ayanmọ value = 0;
    ese add(n) {{
        value = value + n;
    }}
}}</pre>
            </div>
        </div>

        <h2>Dual Lexicon (Yoruba ↔ English)</h2>
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

        <h2>CLI Reference</h2>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>run</h3>
                <p><code>ifa run &lt;file&gt;</code></p>
                <p class="meaning">Execute interpreted</p>
            </div>
            <div class="odu-card">
                <h3>build</h3>
                <p><code>ifa build &lt;file&gt; -o &lt;out&gt;</code></p>
                <p class="meaning">Compile to native binary</p>
            </div>
            <div class="odu-card">
                <h3>bytecode</h3>
                <p><code>ifa bytecode &lt;file&gt;</code></p>
                <p class="meaning">Compile to .ifab</p>
            </div>
            <div class="odu-card">
                <h3>runb</h3>
                <p><code>ifa runb &lt;file.ifab&gt;</code></p>
                <p class="meaning">Run bytecode</p>
            </div>
            <div class="odu-card">
                <h3>check</h3>
                <p><code>ifa check &lt;file&gt;</code></p>
                <p class="meaning">Validate syntax</p>
            </div>
            <div class="odu-card">
                <h3>fmt</h3>
                <p><code>ifa fmt &lt;file&gt;</code></p>
                <p class="meaning">Format source code</p>
            </div>
            <div class="odu-card">
                <h3>babalawo</h3>
                <p><code>ifa babalawo &lt;file&gt;</code></p>
                <p class="meaning">Type checker / linter</p>
            </div>
            <div class="odu-card">
                <h3>test</h3>
                <p><code>ifa test [path]</code></p>
                <p class="meaning">Run test files</p>
            </div>
            <div class="odu-card">
                <h3>repl</h3>
                <p><code>ifa repl</code></p>
                <p class="meaning">Interactive shell</p>
            </div>
            <div class="odu-card">
                <h3>doc</h3>
                <p><code>ifa doc &lt;src&gt; -o &lt;out&gt;</code></p>
                <p class="meaning">Generate HTML docs</p>
            </div>
            <div class="odu-card">
                <h3>lsp</h3>
                <p><code>ifa lsp</code></p>
                <p class="meaning">Start Language Server</p>
            </div>
            <div class="odu-card">
                <h3>sandbox</h3>
                <p><code>ifa sandbox run &lt;file&gt;</code></p>
                <p class="meaning">Sandboxed execution</p>
            </div>
            <div class="odu-card">
                <h3>oja</h3>
                <p><code>ifa oja init|add|build</code></p>
                <p class="meaning">Package manager</p>
            </div>
            <div class="odu-card">
                <h3>flash</h3>
                <p><code>ifa flash &lt;file&gt; --target esp32</code></p>
                <p class="meaning">Flash to embedded</p>
            </div>
            <div class="odu-card">
                <h3>version</h3>
                <p><code>ifa version</code></p>
                <p class="meaning">Show version info</p>
            </div>
        </div>

        <footer>
            <p>Àṣẹ! - Generated by Ifá-Lang Documentation System (Rust)</p>
            <p style="margin-top: 0.5rem;"><a href="https://github.com/AAEO04/ifa-lang" style="color: var(--gold);">GitHub Repository</a></p>
        </footer>
    </div>
</body>
</html>"#,
        css = CSS,
        timestamp = timestamp,
        odu_cards = odu_cards
    )
}

/// Generate an individual Odù domain page
pub fn generate_odu_page(odu: &OduInfo, methods: &[(String, String)]) -> String {
    let mut nav_links = String::new();
    for o in ODU_DOMAINS {
        nav_links.push_str(&format!(r#"<a href="{}.html">{}</a>"#, o.slug, o.name));
        nav_links.push('\n');
    }

    let mut methods_html = String::new();
    for (name, desc) in methods {
        methods_html.push_str(&format!(r#"
            <div class="verse">
                <div class="verse-name">{name}()</div>
                <div class="verse-desc">{desc}</div>
            </div>
        "#, name = name, desc = desc));
    }

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name} - Ifá Corpus</title>
    <style>{css}</style>
</head>
<body>
    <nav class="nav">
        <h3><a href="index.html">🔮 Ifá Corpus</a></h3>
        <hr style="border-color: var(--accent); margin: 1rem 0;">
        {nav_links}
        <hr style="border-color: var(--accent); margin: 1rem 0;">
        <a href="playground.html">🎮 Playground</a>
        <a href="sandbox.html">🏖️ Sandbox</a>
    </nav>
    
    <div class="main-content">
        <div class="container">
            <header>
                <h1><span class="binary">{binary}</span> {name}</h1>
                <p class="meaning">{meaning}</p>
                <p>{desc}</p>
            </header>
            
            <h2>Methods</h2>
            {methods_html}
            
            <footer>
                <p><a href="index.html">← Back to Index</a></p>
            </footer>
        </div>
    </div>
</body>
</html>"#,
        css = CSS,
        name = odu.name,
        binary = odu.binary,
        meaning = odu.meaning,
        desc = odu.description,
        nav_links = nav_links,
        methods_html = if methods_html.is_empty() {
            "<p class=\"meaning\">No methods documented yet.</p>".to_string()
        } else {
            methods_html
        }
    )
}

/// Standard library methods for each domain
pub fn get_stdlib_methods() -> HashMap<&'static str, Vec<(&'static str, &'static str)>> {
    let mut map = HashMap::new();
    
    map.insert("ogbe", vec![
        ("bi", "Initialize system/environment"),
        ("gba", "Get input from user/environment"),
        ("env", "Get environment variable"),
        ("args", "Get CLI arguments"),
        ("platform", "Get current platform (windows/linux/macos)"),
    ]);
    
    map.insert("oyeku", vec![
        ("ku", "Exit program with code"),
        ("duro", "Stop execution gracefully"),
        ("gbale", "Garbage collect / Clean up resources"),
    ]);
    
    map.insert("iwori", vec![
        ("ago", "Get current time as formatted string"),
        ("duro", "Sleep for specified milliseconds"),
        ("now", "Get current timestamp"),
        ("millis", "Get milliseconds since epoch"),
    ]);
    
    map.insert("odi", vec![
        ("ka", "Read data from file"),
        ("ko", "Write data to file"),
        ("fi", "Append data to file"),
        ("si", "Open file handle"),
        ("pa", "Close active file"),
        ("wa", "Check if file exists"),
        ("pa_iwe", "Delete file"),
    ]);
    
    map.insert("irosu", vec![
        ("fo", "Print with newline"),
        ("so", "Log with label prefix"),
        ("kigbe", "Log error to stderr"),
    ]);
    
    map.insert("owonrin", vec![
        ("bo", "Random integer from 0 to n"),
        ("range", "Random integer in range [min, max]"),
        ("paaro", "Shuffle list randomly"),
    ]);
    
    map.insert("obara", vec![
        ("ro", "Add two numbers"),
        ("fikun", "Increment value"),
        ("isodipupo", "Multiply two numbers"),
        ("kun", "Sum a list of numbers"),
    ]);
    
    map.insert("okanran", vec![
        ("binu", "Raise error with message"),
        ("je", "Handle/catch error"),
    ]);
    
    map.insert("ogunda", vec![
        ("ge", "Create new array"),
        ("fi", "Push element to array"),
        ("mu", "Pop element from array"),
        ("to", "Sort array"),
        ("map", "Map function over array"),
        ("filter", "Filter array with predicate"),
        ("gigun", "Get array length"),
    ]);
    
    map.insert("osa", vec![
        ("sa", "Spawn thread/async task"),
        ("duro", "Wait for task completion"),
        ("json_si", "Parse JSON string to object"),
        ("json_lati", "Convert object to JSON string"),
    ]);
    
    map.insert("ika", vec![
        ("so", "Concatenate strings"),
        ("ge", "Slice string [start:end]"),
        ("ka", "Get string length"),
        ("split", "Split string by delimiter"),
        ("upper", "Convert to uppercase"),
        ("lower", "Convert to lowercase"),
        ("trim", "Trim whitespace"),
        ("contains", "Check if string contains substring"),
        ("replace", "Replace substring"),
    ]);
    
    map.insert("oturupon", vec![
        ("din", "Subtract two numbers"),
        ("pin", "Divide two numbers"),
        ("ku", "Modulo (remainder)"),
    ]);
    
    map.insert("otura", vec![
        ("ran", "Send HTTP request"),
        ("de", "Bind to network port"),
        ("gba", "Receive network data"),
        ("http_get", "HTTP GET request"),
        ("http_post", "HTTP POST request"),
    ]);
    
    map.insert("irete", vec![
        ("di", "Hash data (SHA256)"),
        ("fun", "Compress data"),
        ("tu", "Decompress data"),
        ("base64_si", "Encode to base64"),
        ("base64_lati", "Decode from base64"),
    ]);
    
    map.insert("ose", vec![
        ("ya", "Draw pixel/shape"),
        ("han", "Render frame"),
        ("html", "Generate HTML element"),
        ("css", "Generate CSS styles"),
    ]);
    
    map.insert("ofun", vec![
        ("ase", "Request elevated permissions"),
        ("fun", "Grant permission"),
        ("ka_iwe", "Read manifest/documentation"),
    ]);
    
    map
}

/// Generate all documentation files to the output directory
pub fn generate_docs(output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    
    // Always regenerate index.html
    let index_html = generate_index_html();
    fs::write(output_dir.join("index.html"), index_html)?;
    println!("  Generated: index.html");
    
    // Get stdlib methods
    let stdlib = get_stdlib_methods();
    
    // Generate individual Odù pages with ASCII filenames
    // Skip if file already exists (preserve legacy Python-generated docs)
    for odu in ODU_DOMAINS {
        let filename = format!("{}.html", odu.slug);
        let filepath = output_dir.join(&filename);
        
        if filepath.exists() {
            println!("  Skipped:   {} (exists)", filename);
            continue;
        }
        
        let methods: Vec<(String, String)> = stdlib
            .get(odu.slug)
            .map(|m| m.iter().map(|(n, d)| (n.to_string(), d.to_string())).collect())
            .unwrap_or_default();
        
        let page_html = generate_odu_page(odu, &methods);
        fs::write(&filepath, page_html)?;
        println!("  Generated: {}", filename);
    }
    
    Ok(())
}
