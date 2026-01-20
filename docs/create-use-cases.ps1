# Create use-case page content

$basePath = "c:\Users\allio\Desktop\ifa_lang\docs\examples\use-cases"

# Static Site Generator
$staticSite = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Static Site Generator - Ifa-Lang Use Cases</title>
    <link rel="stylesheet" href="../../js/common.css">
    <script src="../../js/nav.js" defer></script>
    <script src="../../js/common.js" defer></script>
</head>
<body>
    <div id="nav-placeholder"></div>
    <div class="breadcrumbs"><div class="breadcrumbs-container"><ul class="breadcrumb-list"></ul></div></div>
    <div class="container">
        <h1>Static Site Generator</h1>
        <p>Build a static site generator using Ifa-Lang's file I/O and string processing.</p>

        <h2>Overview</h2>
        <p>This example demonstrates using <strong>Odi (Fs)</strong> for file operations and <strong>Ika (String)</strong> for template processing.</p>

        <h2>Domains Used</h2>
        <ul>
            <li><strong>Odi / Fs</strong> - Read markdown files, write HTML output</li>
            <li><strong>Ika / String</strong> - Template substitution, markdown parsing</li>
            <li><strong>Ogunda / List</strong> - File lists, navigation</li>
            <li><strong>Irosu / Fmt</strong> - Build logging</li>
        </ul>

        <h2>Code Example</h2>
        <pre><code>// Static Site Generator
ayanmo template = Odi.ka("template.html");
ayanmo posts = Odi.akojopo("posts/*.md");

fun post ninu posts {
    ayanmo content = Odi.ka(post);
    ayanmo html = markdown_to_html(content);
    ayanmo output = Ika.ropo(template, "{{content}}", html);
    
    ayanmo filename = Ika.ropo(post, ".md", ".html");
    Odi.ko("public/" + filename, output);
    Irosu.fo("Built: " + filename);
}

Irosu.fo("Site generated!");</code></pre>

        <h2>Template System</h2>
        <pre><code>// Simple template engine
ese markdown_to_html(text) {
    ayanmo result = text;
    
    // Headers
    result = Ika.ropo(result, "# ", "&lt;h1&gt;");
    result = Ika.ropo(result, "## ", "&lt;h2&gt;");
    
    // Bold
    result = Ika.ropo(result, "**", "&lt;strong&gt;");
    
    pada result;
}</code></pre>

        <footer class="doc-footer">
            <p><a href="index.html">Back to Use Cases</a></p>
        </footer>
    </div>
</body>
</html>
"@
Set-Content -Path "$basePath\static-site.html" -Value $staticSite -Encoding UTF8
Write-Host "Created: static-site.html"

# File Processor
$fileProcessor = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>File Processor - Ifa-Lang Use Cases</title>
    <link rel="stylesheet" href="../../js/common.css">
    <script src="../../js/nav.js" defer></script>
    <script src="../../js/common.js" defer></script>
</head>
<body>
    <div id="nav-placeholder"></div>
    <div class="breadcrumbs"><div class="breadcrumbs-container"><ul class="breadcrumb-list"></ul></div></div>
    <div class="container">
        <h1>File Processor</h1>
        <p>Batch process files with Ifa-Lang's powerful I/O and concurrency features.</p>

        <h2>Use Case</h2>
        <p>Process multiple files in parallel - resize images, convert formats, or extract data.</p>

        <h2>Domains Used</h2>
        <ul>
            <li><strong>Odi / Fs</strong> - File reading/writing</li>
            <li><strong>Osa / Async</strong> - Parallel processing</li>
            <li><strong>Ika / String</strong> - Path manipulation</li>
        </ul>

        <h2>Code Example</h2>
        <pre><code>// Batch File Processor
ayanmo files = Odi.akojopo("input/*.txt");
ayanmo processed = 0;

// Process files in parallel
Osa.pelu_ara(files, ese(file) {
    ayanmo content = Odi.ka(file);
    
    // Transform content
    ayanmo upper = Ika.uppercase(content);
    
    // Write to output
    ayanmo outname = Ika.ropo(file, "input/", "output/");
    Odi.ko(outname, upper);
    
    processed = processed + 1;
});

Irosu.fo("Processed " + processed + " files");</code></pre>

        <h2>CSV Processing Example</h2>
        <pre><code>// Parse and transform CSV
ayanmo csv = Odi.ka("data.csv");
ayanmo lines = Ika.pin(csv, "\n");
ayanmo output = [];

fun line ninu lines {
    ayanmo fields = Ika.pin(line, ",");
    ti Ogunda.len(fields) >= 2 {
        ayanmo name = Ogunda.gba(fields, 0);
        ayanmo value = Ogunda.gba(fields, 1);
        Ogunda.fi(output, name + ": " + value);
    }
}

Odi.ko("output.txt", Ika.so(output, "\n"));</code></pre>

        <footer class="doc-footer">
            <p><a href="index.html">Back to Use Cases</a></p>
        </footer>
    </div>
</body>
</html>
"@
Set-Content -Path "$basePath\file-processor.html" -Value $fileProcessor -Encoding UTF8
Write-Host "Created: file-processor.html"

# Data Pipeline
$dataPipeline = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Data Pipeline - Ifa-Lang Use Cases</title>
    <link rel="stylesheet" href="../../js/common.css">
    <script src="../../js/nav.js" defer></script>
    <script src="../../js/common.js" defer></script>
</head>
<body>
    <div id="nav-placeholder"></div>
    <div class="breadcrumbs"><div class="breadcrumbs-container"><ul class="breadcrumb-list"></ul></div></div>
    <div class="container">
        <h1>Data Pipeline</h1>
        <p>Build ETL pipelines with Ifa-Lang's functional data processing.</p>

        <h2>Pipeline Stages</h2>
        <ol>
            <li><strong>Extract</strong> - Fetch data from APIs or files</li>
            <li><strong>Transform</strong> - Filter, map, aggregate</li>
            <li><strong>Load</strong> - Write to database or file</li>
        </ol>

        <h2>Domains Used</h2>
        <ul>
            <li><strong>Otura / Net</strong> - API data fetching</li>
            <li><strong>Ogunda / List</strong> - Data transformations</li>
            <li><strong>Storage / Db</strong> - Persistence</li>
            <li><strong>Osa / Async</strong> - Parallel processing</li>
        </ul>

        <h2>Code Example</h2>
        <pre><code>// ETL Data Pipeline
ese extract() {
    // Fetch from API
    ayanmo response = Otura.gba("https://api.example.com/data");
    ayanmo data = Ika.json_parse(response);
    pada data;
}

ese transform(records) {
    // Filter and map
    ayanmo result = [];
    fun record ninu records {
        ti record.active == otito {
            ayanmo cleaned = {
                id: record.id,
                name: Ika.uppercase(record.name),
                score: record.value * 100
            };
            Ogunda.fi(result, cleaned);
        }
    }
    pada result;
}

ese load(data) {
    // Write to storage
    Storage.set("pipeline_results", data);
    Irosu.fo("Loaded " + Ogunda.len(data) + " records");
}

// Run pipeline
ayanmo raw = extract();
ayanmo processed = transform(raw);
load(processed);</code></pre>

        <footer class="doc-footer">
            <p><a href="index.html">Back to Use Cases</a></p>
        </footer>
    </div>
</body>
</html>
"@
Set-Content -Path "$basePath\data-pipeline.html" -Value $dataPipeline -Encoding UTF8
Write-Host "Created: data-pipeline.html"

# Password Manager
$passwordManager = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Password Manager - Ifa-Lang Use Cases</title>
    <link rel="stylesheet" href="../../js/common.css">
    <script src="../../js/nav.js" defer></script>
    <script src="../../js/common.js" defer></script>
</head>
<body>
    <div id="nav-placeholder"></div>
    <div class="breadcrumbs"><div class="breadcrumbs-container"><ul class="breadcrumb-list"></ul></div></div>
    <div class="container">
        <h1>Password Manager</h1>
        <p>Secure password storage using Ifa-Lang's cryptography domain.</p>

        <h2>Features</h2>
        <ul>
            <li>AES-256 encryption for vault</li>
            <li>Master password with PBKDF2 derivation</li>
            <li>Secure random password generation</li>
            <li>Clipboard integration</li>
        </ul>

        <h2>Domains Used</h2>
        <ul>
            <li><strong>Irete / Crypto</strong> - Encryption, hashing</li>
            <li><strong>Owonrin / Rand</strong> - Password generation</li>
            <li><strong>Odi / Fs</strong> - Vault storage</li>
            <li><strong>Irosu / Fmt</strong> - User interface</li>
        </ul>

        <h2>Code Example</h2>
        <pre><code>// Password Manager Core
ayanmo VAULT_FILE = "vault.enc";

ese generate_password(length) {
    ayanmo chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
    ayanmo password = "";
    fun i ninu 0..length {
        ayanmo idx = Owonrin.random(0, Ika.len(chars) - 1);
        password = password + Ika.gba(chars, idx);
    }
    pada password;
}

ese encrypt_vault(data, master_key) {
    ayanmo key = Irete.pbkdf2(master_key, "salt", 100000);
    ayanmo encrypted = Irete.aes_encrypt(data, key);
    Odi.ko(VAULT_FILE, encrypted);
}

ese decrypt_vault(master_key) {
    ayanmo encrypted = Odi.ka(VAULT_FILE);
    ayanmo key = Irete.pbkdf2(master_key, "salt", 100000);
    ayanmo data = Irete.aes_decrypt(encrypted, key);
    pada Ika.json_parse(data);
}

// Usage
ayanmo vault = decrypt_vault("master_password");
ayanmo new_pass = generate_password(20);
Irosu.fo("Generated: " + new_pass);</code></pre>

        <footer class="doc-footer">
            <p><a href="index.html">Back to Use Cases</a></p>
        </footer>
    </div>
</body>
</html>
"@
Set-Content -Path "$basePath\password-manager.html" -Value $passwordManager -Encoding UTF8
Write-Host "Created: password-manager.html"

# Audio Player
$audioPlayer = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Audio Player - Ifa-Lang Use Cases</title>
    <link rel="stylesheet" href="../../js/common.css">
    <script src="../../js/nav.js" defer></script>
    <script src="../../js/common.js" defer></script>
</head>
<body>
    <div id="nav-placeholder"></div>
    <div class="breadcrumbs"><div class="breadcrumbs-container"><ul class="breadcrumb-list"></ul></div></div>
    <div class="container">
        <h1>Audio Player</h1>
        <p>Build an audio player with Ifa-Lang's Ohun (Audio) domain.</p>

        <h2>Features</h2>
        <ul>
            <li>Play MP3, WAV, FLAC files</li>
            <li>Playlist management</li>
            <li>Volume and playback control</li>
            <li>Audio visualization</li>
        </ul>

        <h2>Domains Used</h2>
        <ul>
            <li><strong>Ohun / Audio</strong> - Audio playback</li>
            <li><strong>Odi / Fs</strong> - File discovery</li>
            <li><strong>Ogunda / List</strong> - Playlist</li>
            <li><strong>Ose / Tui</strong> - Terminal UI</li>
        </ul>

        <h2>Code Example</h2>
        <pre><code>// Simple Audio Player
ayanmo playlist = Odi.akojopo("music/*.mp3");
ayanmo current = 0;
ayanmo playing = iro;

ese play_track(index) {
    ti index >= 0 && index < Ogunda.len(playlist) {
        ayanmo track = Ogunda.gba(playlist, index);
        Ohun.play(track);
        playing = otito;
        Irosu.fo("Playing: " + track);
    }
}

ese pause() {
    Ohun.pause();
    playing = iro;
}

ese next() {
    current = current + 1;
    ti current >= Ogunda.len(playlist) {
        current = 0;
    }
    play_track(current);
}

ese previous() {
    current = current - 1;
    ti current < 0 {
        current = Ogunda.len(playlist) - 1;
    }
    play_track(current);
}

// Start playing
play_track(0);

// Handle events
Ohun.on_complete(ese() {
    next();
});</code></pre>

        <h2>TUI Interface</h2>
        <pre><code>// Terminal UI for player
nigba otito {
    Ose.clear();
    Irosu.fo("=== Ifa Music Player ===");
    Irosu.fo("Now: " + Ogunda.gba(playlist, current));
    Irosu.fo("");
    Irosu.fo("[P] Play/Pause  [N] Next  [B] Back  [Q] Quit");
    
    ayanmo key = Ose.getch();
    yàn key {
        "p" => ti playing { pause(); } bibẹkọ { play_track(current); }
        "n" => next();
        "b" => previous();
        "q" => da;
    }
}</code></pre>

        <footer class="doc-footer">
            <p><a href="index.html">Back to Use Cases</a></p>
        </footer>
    </div>
</body>
</html>
"@
Set-Content -Path "$basePath\audio-player.html" -Value $audioPlayer -Encoding UTF8
Write-Host "Created: audio-player.html"

# Simple Game
$simpleGame = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Simple Game - Ifa-Lang Use Cases</title>
    <link rel="stylesheet" href="../../js/common.css">
    <script src="../../js/nav.js" defer></script>
    <script src="../../js/common.js" defer></script>
</head>
<body>
    <div id="nav-placeholder"></div>
    <div class="breadcrumbs"><div class="breadcrumbs-container"><ul class="breadcrumb-list"></ul></div></div>
    <div class="container">
        <h1>Simple Game</h1>
        <p>Build a text-based game with Ifa-Lang.</p>

        <h2>Game: Number Guessing</h2>
        <p>A classic number guessing game demonstrating loops, conditionals, and I/O.</p>

        <h2>Domains Used</h2>
        <ul>
            <li><strong>Owonrin / Rand</strong> - Random number generation</li>
            <li><strong>Irosu / Fmt</strong> - Display and input</li>
            <li><strong>Ika / String</strong> - Input parsing</li>
        </ul>

        <h2>Code Example</h2>
        <pre><code>// Number Guessing Game
Irosu.fo("=== Number Guessing Game ===");
Irosu.fo("I'm thinking of a number between 1 and 100...");

ayanmo secret = Owonrin.random(1, 100);
ayanmo attempts = 0;
ayanmo won = iro;

nigba !won {
    ayanmo input = Irosu.gbigba("Your guess: ");
    ayanmo guess = Ika.to_int(input);
    attempts = attempts + 1;
    
    ti guess < secret {
        Irosu.fo("Too low! Try again.");
    } bibẹkọ ti guess > secret {
        Irosu.fo("Too high! Try again.");
    } bibẹkọ {
        won = otito;
        Irosu.fo("Correct! You got it in " + attempts + " attempts!");
    }
}

ti attempts <= 5 {
    Irosu.fo("Amazing! You're a master guesser!");
} bibẹkọ ti attempts <= 10 {
    Irosu.fo("Good job!");
} bibẹkọ {
    Irosu.fo("Keep practicing!");
}</code></pre>

        <h2>Game: Text Adventure</h2>
        <pre><code>// Mini Text Adventure
ayanmo room = "start";
ayanmo inventory = [];

nigba otito {
    yàn room {
        "start" => {
            Irosu.fo("You are in a dark room. Exits: NORTH, EAST");
            ayanmo cmd = Irosu.gbigba("> ");
            ti cmd == "north" { room = "hall"; }
            ti cmd == "east" { room = "garden"; }
        }
        "hall" => {
            Irosu.fo("A grand hall with a KEY on the floor. Exits: SOUTH");
            ayanmo cmd = Irosu.gbigba("> ");
            ti cmd == "take key" { 
                Ogunda.fi(inventory, "key"); 
                Irosu.fo("Key taken!");
            }
            ti cmd == "south" { room = "start"; }
        }
        "garden" => {
            Irosu.fo("A beautiful garden with a locked GATE. Exits: WEST");
            ayanmo cmd = Irosu.gbigba("> ");
            ti cmd == "open gate" {
                ti Ogunda.wa(inventory, "key") {
                    Irosu.fo("You escape! YOU WIN!");
                    da;
                } bibẹkọ {
                    Irosu.fo("It's locked.");
                }
            }
            ti cmd == "west" { room = "start"; }
        }
    }
}</code></pre>

        <footer class="doc-footer">
            <p><a href="index.html">Back to Use Cases</a></p>
        </footer>
    </div>
</body>
</html>
"@
Set-Content -Path "$basePath\simple-game.html" -Value $simpleGame -Encoding UTF8
Write-Host "Created: simple-game.html"

Write-Host "`nAll 6 use-case pages created!"
