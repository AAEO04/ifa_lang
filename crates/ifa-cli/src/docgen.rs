//! # Ifá Documentation Generator (DocGen)
//!
//! Generates HTML documentation for Ifá-Lang projects in the style of the Ifá Corpus.

use chrono::Local;
use color_eyre::eyre::Result;
use ifa_types::domain::OduDomain;
use ifa_types::odu_metadata::all_odu_domains_with_methods;
use std::fs;
use std::path::{Path, PathBuf};

// User Documentation Structures
#[derive(Debug, Clone, Default)]
pub struct UserDoc {
    pub odus: Vec<UserOdu>,
    pub orphans: Vec<DocItem>, // Functions/Constants not in an Odù
}

#[derive(Debug, Clone)]
pub struct UserOdu {
    pub name: String,
    pub description: String,
    pub items: Vec<DocItem>,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct DocItem {
    pub name: String,
    pub kind: String, // "ese", "ayanmo", "const"
    pub signature: String,
    pub description: String,
}

impl UserDoc {
    fn new() -> Self {
        Self::default()
    }

    fn add_odu(&mut self, name: String, description: String) -> &mut UserOdu {
        let slug = name.to_lowercase();
        self.odus.push(UserOdu {
            name,
            description,
            items: Vec::new(),
            slug,
        });
        self.odus.last_mut().unwrap()
    }
}

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
    OduInfo {
        name: "Ogbè",
        slug: "ogbe",
        alias: "System",
        binary: "1111",
        meaning: "The Light",
        description: "System initialization, beginnings, CLI arguments",
    },
    OduInfo {
        name: "Ọ̀yẹ̀kú",
        slug: "oyeku",
        alias: "Exit",
        binary: "0000",
        meaning: "The Darkness",
        description: "Process termination, endings, sleep",
    },
    OduInfo {
        name: "Ìwòrì",
        slug: "iwori",
        alias: "Time",
        binary: "0110",
        meaning: "The Mirror",
        description: "Reflection, time, iteration, loops",
    },
    OduInfo {
        name: "Òdí",
        slug: "odi",
        alias: "File",
        binary: "1001",
        meaning: "The Vessel",
        description: "Storage, file operations, containment",
    },
    OduInfo {
        name: "Ìrosù",
        slug: "irosu",
        alias: "Log",
        binary: "1100",
        meaning: "The Speaker",
        description: "Communication, console I/O, expression",
    },
    OduInfo {
        name: "Ọ̀wọ́nrín",
        slug: "owonrin",
        alias: "Random",
        binary: "0011",
        meaning: "The Chaotic",
        description: "Randomness, chance, unpredictability",
    },
    OduInfo {
        name: "Ọ̀bàrà",
        slug: "obara",
        alias: "Math",
        binary: "1000",
        meaning: "The King",
        description: "Expansion, addition, multiplication",
    },
    OduInfo {
        name: "Ọ̀kànràn",
        slug: "okanran",
        alias: "Error",
        binary: "0001",
        meaning: "The Troublemaker",
        description: "Errors, exceptions, warnings",
    },
    OduInfo {
        name: "Ògúndá",
        slug: "ogunda",
        alias: "Array",
        binary: "1110",
        meaning: "The Cutter",
        description: "Arrays, process control, separation",
    },
    OduInfo {
        name: "Ọ̀sá",
        slug: "osa",
        alias: "Flow",
        binary: "0111",
        meaning: "The Wind",
        description: "Control flow, jumps, conditionals",
    },
    OduInfo {
        name: "Ìká",
        slug: "ika",
        alias: "String",
        binary: "0100",
        meaning: "The Constrictor",
        description: "Strings, compression, binding",
    },
    OduInfo {
        name: "Òtúúrúpọ̀n",
        slug: "oturupon",
        alias: "Reduce",
        binary: "0010",
        meaning: "The Bearer",
        description: "Reduction, subtraction, division",
    },
    OduInfo {
        name: "Òtúrá",
        slug: "otura",
        alias: "Net",
        binary: "1011",
        meaning: "The Messenger",
        description: "Network, communication, sending",
    },
    OduInfo {
        name: "Ìrẹtẹ̀",
        slug: "irete",
        alias: "Crypto",
        binary: "1101",
        meaning: "The Crusher",
        description: "Memory management, garbage collection",
    },
    OduInfo {
        name: "Ọ̀ṣẹ́",
        slug: "ose",
        alias: "UI",
        binary: "1010",
        meaning: "The Beautifier",
        description: "Graphics, display, aesthetics",
    },
    OduInfo {
        name: "Òfún",
        slug: "ofun",
        alias: "Root",
        binary: "0101",
        meaning: "The Creator",
        description: "Object creation, inheritance",
    },
    OduInfo {
        name: "Cpu",
        slug: "cpu",
        alias: "Parallel",
        binary: "10010",
        meaning: "The Multitasker",
        description: "Parallel computing via rayon",
    },
    OduInfo {
        name: "Gpu",
        slug: "gpu",
        alias: "Compute",
        binary: "10011",
        meaning: "The Accelerator",
        description: "GPU compute via wgpu",
    },
    OduInfo {
        name: "Storage",
        slug: "storage",
        alias: "DB",
        binary: "10100",
        meaning: "The Vault",
        description: "Key-value persistence",
    },
    OduInfo {
        name: "Sys",
        slug: "sys",
        alias: "Kernel",
        binary: "11101",
        meaning: "The Core",
        description: "Kernel/OS interface",
    },
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
pub fn generate_index_html(user_doc: &UserDoc) -> String {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();

    let mut odu_cards = String::new();

    // Standard Odù
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

    // User Odù
    let mut user_odu_cards = String::new();
    if !user_doc.odus.is_empty() {
        for odu in &user_doc.odus {
            user_odu_cards.push_str(&format!(
                r#"
            <a href="user_{slug}.html" class="odu-card" style="border-color: var(--gold);">
                <h3>📘 {name}</h3>
                <p class="meaning">User Domain</p>
                <p>{desc}</p>
            </a>
        "#,
                slug = odu.slug,
                name = odu.name,
                desc = odu.description
            ));
        }
    }

    // User Orphans (Globals)
    let mut orphan_html = String::new();
    if !user_doc.orphans.is_empty() {
        orphan_html.push_str("<h2>Global Verses & Constants</h2><div class=\"odu-grid\">");
        for item in &user_doc.orphans {
            let icon = match item.kind.as_str() {
                "ese" => "📜",
                "const" => "💎",
                "ayanmo" => "📦",
                _ => "📄",
            };
            orphan_html.push_str(&format!(
                r#"
                <div class="odu-card">
                    <h3>{icon} {name}</h3>
                    <p><code>{sig}</code></p>
                    <p class="meaning">{desc}</p>
                </div>
            "#,
                icon = icon,
                name = item.name,
                sig = item.signature,
                desc = item.description
            ));
        }
        orphan_html.push_str("</div>");
    }

    format!(
        r#"<!DOCTYPE html>
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
        
        {user_section}
        
        {orphan_section}

        <h2>The 16 Standard Odù Domains</h2>
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

        <h2>Capabilities</h2>
        <p style="color: var(--text-dim); margin-bottom: 1rem;">Core runtime capabilities and domain groupings exposed by ifa-std:</p>
        <div class="odu-grid">
            <div class="odu-card">
                <h3>HTML</h3>
                <p>HTML parsing and document helpers</p>
                <p class="meaning">feature: html</p>
            </div>
            <div class="odu-card">
                <h3>Network</h3>
                <p>HTTP and socket-oriented messaging via Otura</p>
                <p class="meaning">feature: network</p>
            </div>
            <div class="odu-card">
                <h3>Crypto</h3>
                <p>Encryption, hashing, signatures</p>
                <p class="meaning">feature: crypto</p>
            </div>
            <div class="odu-card">
                <h3>Terminal UI</h3>
                <p>Interactive terminal rendering and input</p>
                <p class="meaning">feature: tui</p>
            </div>
            <div class="odu-card">
                <h3>GPU</h3>
                <p>WebGPU/WGPU compute buffers and pipelines</p>
                <p class="meaning">feature: gpu</p>
            </div>
            <div class="odu-card">
                <h3>Persistence</h3>
                <p>Async storage and compaction helpers</p>
                <p class="meaning">feature: persistence</p>
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
        odu_cards = odu_cards,
        user_section = if !user_odu_cards.is_empty() {
            format!(
                "<h2>Your Project Odù</h2><div class=\"odu-grid\">{}</div>",
                user_odu_cards
            )
        } else {
            String::new()
        },
        orphan_section = orphan_html
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
        methods_html.push_str(&format!(
            r#"
            <div class="verse">
                <div class="verse-name">{name}()</div>
                <div class="verse-desc">{desc}</div>
            </div>
        "#,
            name = name,
            desc = desc
        ));
    }

    format!(
        r#"<!DOCTYPE html>
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

/// Generate a page for a User Odù
pub fn generate_user_odu_page(odu: &UserOdu) -> String {
    let mut methods_html = String::new();

    for item in &odu.items {
        let icon = match item.kind.as_str() {
            "ese" => "📜",
            "const" => "💎",
            "ayanmo" => "📦",
            _ => "📄",
        };

        methods_html.push_str(&format!(
            r#"
            <div class="verse">
                <div class="verse-name">{icon} {name}</div>
                <div class="verse-desc"><code>{sig}</code></div>
                <p style="margin-top:0.5rem">{desc}</p>
            </div>
        "#,
            icon = icon,
            name = item.name,
            sig = item.signature,
            desc = item.description
        ));
    }

    format!(
        r#"<!DOCTYPE html>
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
        <a href="index.html">← Back to Index</a>
    </nav>
    
    <div class="main-content">
        <div class="container">
            <header>
                <h1>📘 {name}</h1>
                <p class="meaning">User Domain</p>
                <p>{desc}</p>
            </header>
            
            <h2>Verses & Items</h2>
            {methods_html}
            
            <footer>
                <p>Generated from source code Oríkì</p>
            </footer>
        </div>
    </div>
</body>
</html>"#,
        css = CSS,
        name = odu.name,
        desc = odu.description,
        methods_html = if methods_html.is_empty() {
            "<p class=\"meaning\">No documentation found.</p>".to_string()
        } else {
            methods_html
        }
    )
}

/// Recursively scan directory for .ifa files
fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, files)?;
            } else if path.extension().is_some_and(|ext| ext == "ifa") {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn extract_docs_before(source: &str, span_start: usize) -> String {
    let _docs = String::new();
    let prefix = &source[..span_start];
    let lines = prefix.lines().rev();
    let mut doc_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("///") {
            doc_lines.push(trimmed.strip_prefix("///").unwrap().trim());
        } else if trimmed.is_empty() {
            continue;
        } else {
            break;
        }
    }

    doc_lines.reverse();
    doc_lines.join(" ")
}

/// Parse a single file for Oríkì
fn parse_file(path: &Path, doc: &mut UserDoc) -> Result<()> {
    let content = fs::read_to_string(path)?;

    let program = match ifa_parser::parser::parse(&content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "Docgen warning: Failed to parse AST for {}: {:?}",
                path.display(),
                e
            );
            return Ok(());
        }
    };

    use ifa_types::ast::Statement;
    for stmt in program.statements {
        let span = stmt.span();
        let docs = extract_docs_before(&content, span.start);

        match stmt {
            Statement::OduDef { name, .. } => {
                doc.add_odu(name, docs);
            }
            Statement::EseDef {
                name,
                params,
                return_type: _,
                ..
            } => {
                let params_str = params
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let item = DocItem {
                    name: name.clone(),
                    kind: "ese".to_string(),
                    signature: format!("{}({})", name, params_str),
                    description: docs,
                };
                if let Some(last_odu) = doc.odus.last_mut() {
                    last_odu.items.push(item);
                } else {
                    doc.orphans.push(item);
                }
            }
            Statement::VarDecl { name, .. } => {
                let item = DocItem {
                    name: name.clone(),
                    kind: "ayanmo".to_string(),
                    signature: format!("ayanmo {}", name),
                    description: docs,
                };
                if let Some(last_odu) = doc.odus.last_mut() {
                    last_odu.items.push(item);
                } else {
                    doc.orphans.push(item);
                }
            }
            Statement::Const { name, .. } => {
                let item = DocItem {
                    name: name.clone(),
                    kind: "const".to_string(),
                    signature: format!("const {}", name),
                    description: docs,
                };
                if let Some(last_odu) = doc.odus.last_mut() {
                    last_odu.items.push(item);
                } else {
                    doc.orphans.push(item);
                }
            }
            Statement::IwaDef(iwa) => {
                doc.add_odu(iwa.name.clone(), docs);
                for method in iwa.methods {
                    let mut sig = format!("{}(", method.name);
                    let params_str = method
                        .params
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    sig.push_str(&params_str);
                    sig.push(')');

                    let mut attrs = method.attributes.join(" ");
                    if !attrs.is_empty() {
                        attrs = format!(" [{}]", attrs);
                    }

                    let method_docs = extract_docs_before(&content, method.span.start);
                    let item = DocItem {
                        name: method.name,
                        kind: format!("iwa_method{}", attrs),
                        signature: sig,
                        description: method_docs,
                    };
                    if let Some(last_odu) = doc.odus.last_mut() {
                        last_odu.items.push(item);
                    }
                }
            }
            Statement::Alias {
                name, target: _, ..
            } => {
                let item = DocItem {
                    name: name.clone(),
                    kind: "alias".to_string(),
                    signature: format!("alias {}", name),
                    description: docs,
                };
                doc.orphans.push(item);
            }
            Statement::Taboo { source, target, .. } => {
                let item = DocItem {
                    name: format!("{} -> {}", source, target),
                    kind: "taboo".to_string(),
                    signature: format!("èèwọ̀: {} -> {}", source, target),
                    description: docs,
                };
                doc.orphans.push(item);
            }
            Statement::Abo { .. } => {
                let item = DocItem {
                    name: "abo".to_string(),
                    kind: "abo".to_string(),
                    signature: "abo;".to_string(),
                    description: docs,
                };
                doc.orphans.push(item);
            }
            Statement::Ewo { .. } => {
                let item = DocItem {
                    name: "ewo".to_string(),
                    kind: "ewo".to_string(),
                    signature: "ewo ...".to_string(),
                    description: docs,
                };
                doc.orphans.push(item);
            }
            Statement::Ebo { .. } => {
                let item = DocItem {
                    name: "ebo".to_string(),
                    kind: "ebo".to_string(),
                    signature: "ebo ...".to_string(),
                    description: docs,
                };
                doc.orphans.push(item);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Generate all documentation files to the output directory
pub fn generate_docs(input_path: &Path, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    // 1. Scan User Code
    let mut user_doc = UserDoc::new();
    let mut files = Vec::new();

    if input_path.is_file() {
        files.push(input_path.to_path_buf());
    } else {
        walk_dir(input_path, &mut files)?;
    }

    for file in files {
        if let Err(e) = parse_file(&file, &mut user_doc) {
            eprintln!("Warning: Failed to parse {}: {}", file.display(), e);
        }
    }

    // 2. Generate Index
    let index_html = generate_index_html(&user_doc);
    fs::write(output_dir.join("index.html"), index_html)?;
    println!(
        "  Generated: index.html (with {} User Odùs, {} Global verses)",
        user_doc.odus.len(),
        user_doc.orphans.len()
    );

    // 3. Generate StdLib Pages
    let all_methods = all_odu_domains_with_methods();

    for odu in ODU_DOMAINS {
        let filename = format!("{}.html", odu.slug);
        let filepath = output_dir.join(&filename);

        if filepath.exists() {
            // For standard libs, we might skip to preserve overrides, or always overwrite
            // Let's overwrite for consistency if version changed
        }

        let domain_enum = match odu.slug {
            "ogbe" => Some(OduDomain::Ogbe),
            "oyeku" => Some(OduDomain::Oyeku),
            "iwori" => Some(OduDomain::Iwori),
            "odi" => Some(OduDomain::Odi),
            "irosu" => Some(OduDomain::Irosu),
            "owonrin" => Some(OduDomain::Owonrin),
            "obara" => Some(OduDomain::Obara),
            "okanran" => Some(OduDomain::Okanran),
            "ogunda" => Some(OduDomain::Ogunda),
            "osa" => Some(OduDomain::Osa),
            "ika" => Some(OduDomain::Ika),
            "oturupon" => Some(OduDomain::Oturupon),
            "otura" => Some(OduDomain::Otura),
            "irete" => Some(OduDomain::Irete),
            "ose" => Some(OduDomain::Ose),
            "ofun" => Some(OduDomain::Ofun),
            "cpu" => Some(OduDomain::Cpu),
            "gpu" => Some(OduDomain::Gpu),
            "storage" => Some(OduDomain::Storage),
            "sys" => Some(OduDomain::Sys),
            _ => None,
        };

        let methods: Vec<(String, String)> = if let Some(d) = domain_enum {
            all_methods
                .iter()
                .find(|(dom, _)| *dom == d)
                .map(|(_, m)| {
                    m.iter()
                        .map(|info| (info.yoruba.to_string(), info.english.to_string()))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

        let page_html = generate_odu_page(odu, &methods);
        fs::write(&filepath, page_html)?;
    }

    // 4. Generate User Odù Pages
    for odu in &user_doc.odus {
        let filename = format!("user_{}.html", odu.slug);
        let filepath = output_dir.join(&filename);
        let page_html = generate_user_odu_page(odu);
        fs::write(&filepath, page_html)?;
        println!("  Generated: {}", filename);
    }

    Ok(())
}
