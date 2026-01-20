# Fix encoding for simple-game.html
$content = @"
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
        <pre><code>// Number Guessing Game (English syntax)
Fmt.println("=== Number Guessing Game ===");
Fmt.println("I'm thinking of a number between 1 and 100...");

let secret = Rand.random(1, 100);
let attempts = 0;
let won = false;

while !won {
    let input = Fmt.input("Your guess: ");
    let guess = String.to_int(input);
    attempts = attempts + 1;
    
    if guess &lt; secret {
        Fmt.println("Too low! Try again.");
    } else if guess &gt; secret {
        Fmt.println("Too high! Try again.");
    } else {
        won = true;
        Fmt.println("Correct! You got it in " + attempts + " attempts!");
    }
}

if attempts &lt;= 5 {
    Fmt.println("Amazing! You're a master guesser!");
} else if attempts &lt;= 10 {
    Fmt.println("Good job!");
} else {
    Fmt.println("Keep practicing!");
}</code></pre>

        <h2>Game: Text Adventure</h2>
        <pre><code>// Mini Text Adventure (English syntax)
let room = "start";
let inventory = [];

while true {
    match room {
        "start" =&gt; {
            Fmt.println("You are in a dark room. Exits: NORTH, EAST");
            let cmd = Fmt.input("&gt; ");
            if cmd == "north" { room = "hall"; }
            if cmd == "east" { room = "garden"; }
        }
        "hall" =&gt; {
            Fmt.println("A grand hall with a KEY on the floor. Exits: SOUTH");
            let cmd = Fmt.input("&gt; ");
            if cmd == "take key" { 
                List.push(inventory, "key"); 
                Fmt.println("Key taken!");
            }
            if cmd == "south" { room = "start"; }
        }
        "garden" =&gt; {
            Fmt.println("A beautiful garden with a locked GATE. Exits: WEST");
            let cmd = Fmt.input("&gt; ");
            if cmd == "open gate" {
                if List.contains(inventory, "key") {
                    Fmt.println("You escape! YOU WIN!");
                    break;
                } else {
                    Fmt.println("It's locked.");
                }
            }
            if cmd == "west" { room = "start"; }
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

# Write with BOM for proper UTF-8
[System.IO.File]::WriteAllText("c:\Users\allio\Desktop\ifa_lang\docs\examples\use-cases\simple-game.html", $content, [System.Text.UTF8Encoding]::new($false))
Write-Host "Fixed: simple-game.html"
