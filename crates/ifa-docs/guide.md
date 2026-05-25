# Language guide

## Hello world

```ifa
Irosu.fo("Hello, Ifá-Lang!");
ase;
```

Output domains (`Irosu`) are available without imports.

## Bilingual keywords

Every keyword exists in both Yoruba and English. Mix freely.

| Category | Yoruba | English |
|----------|--------|---------|
| Import | `iba`, `ìbà` | `import`, `mu` |
| From | `lati`, `láti` | `from` |
| Variable | `ayanmo`, `ayanmọ`, `àyànmọ́` | `variable`, `let`, `var` |
| Constant | `loruko`, `ka`, `ayanfe`, `àyànfẹ́` | `const` |
| Class | `odu`, `odù` | `class` |
| Function | `ese`, `ẹsẹ` | `fn`, `function`, `def` |
| If | `ti`, `bí` | `if` |
| Else | `bibẹkọ` | `else` |
| While | `nigba` | `while` |
| For | `fun` | `for` |
| In | `ninu` | `in` |
| Return | `pada` | `return` |
| Break | `da` | `break` |
| Continue | `tesiwaju`, `bayan` | `continue` |
| End of ritual | `ase`, `àṣẹ` | `end` |
| Match | `yàn`, `yán` | `match` |
| Async | `daro` | `async` |
| Await | `reti` | `await` |
| Try | `gbiyanju`, `gbìyànjú` | `try` |
| Catch | `gba`, `gbà` | `catch` |
| Finally | `nipari`, `nípàrí` | `finally` |
| Public/export | `gbangba`, `fi` | `public`, `export` |
| Unsafe | `ailewu`, `àìléwu` | `unsafe` |
| Assert | `ewo`, `ẹ̀wọ̀` | `assert`, `verify` |
| Taboo | `èèwọ̀`, `ewọ` | `taboo` |
| True | `otito` | `true` |
| False | `iro` | `false` |
| Null | `ofo` | `null`, `nil` |
| And | `ati`, `and` | `&&` |
| Or | `tabi`, `or` | `\|\|` |
| Not | `kii`, `not` | `!` |
| Yield | `jowo`, `jọ̀wọ́` | `yield` |

## Variables

```ifa
ayanmo name = "Ifá-Lang";   // Yoruba
let count = 42;              // English
var total = 100;             // Also English
```

Variables are mutable by default. Constants use `const` / `ayanfe`:

```ifa
const PI = 3.14159;
ayanfe MAX = 100;
```

### Type hints

Optional type annotations:

```ifa
ayanmo name: Str = "Ifá";
ayanmo count: Int = 42;
ayanmo ratio: Float = 3.14;
ayanmo ok: Bool = otito;
ayanmo items: List = [1, 2, 3];
ayanmo mapping: Map = {"key": "value"};
```

Available types: `Int`, `Float`, `Str`, `Bool`, `List`, `Map`, `Any`, `void`, plus sized integers (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`) and floats (`f32`, `f64`).

## Comments

```ifa
# Hash comment
// Slash comment
/* Block comment */
/// Doc comment (for oriki documentation generator)
```

## Operators

### Arithmetic

| Op | Meaning |
|----|---------|
| `+` | Add |
| `-` | Subtract |
| `*` | Multiply |
| `/` | Divide |
| `%` | Modulo |

### Comparison

| Op | Meaning |
|----|---------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less or equal |
| `>=` | Greater or equal |

### Logical

| Symbol | Yoruba | English | Meaning |
|--------|--------|---------|---------|
| `&&` | `ati` | `and` | Logical AND (short-circuit) |
| `\|\|` | `tabi` | `or` | Logical OR (short-circuit) |
| `!` | `kii` | `not` | Logical NOT |
| `??` | | | Null-coalescing |

Logical operators short-circuit and return the actual operand value (not just `true`/`false`).

```ifa
ayanmo x = ofo || "default";  // x = "default"
ayanmo y = 0 && (1 / 0);      // y = 0 (short-circuit prevents division)
```

### Unary

`+`, `-`, `&` (reference), `*` (dereference).

### Augmented assignment

```ifa
count += 1;   // count = count + 1
count -= 5;   // count = count - 5
count *= 2;   // count = count * 2
count /= 3;   // count = count / 3
count %= 2;   // count = count % 2
```

## String interpolation

```ifa
ayanmo name = "Orunmila";
ayanmo count = 16;
ayanmo msg = $"Alafia, {name}! The count is {count}.";
// → "Alafia, Orunmila! The count is 16."
```

Nested expressions, method calls, and arithmetic work inside `{}`.

## Strings

Double-quoted and single-quoted strings:

```ifa
ayanmo s1 = "Hello";
ayanmo s2 = 'World';
```

## Lists

```ifa
ayanmo items = [1, 2, 3, 4, 5];
ayanmo first = items[0];
ayanmo last = items[-1];   // Negative indexing
```

## Maps

```ifa
ayanmo config = {
    "host": "localhost",
    "port": 8080
};
ayanmo host = config["host"];
```

## Control flow

### If / Else

```ifa
ti x > 10 {            // Yoruba
    Irosu.fo("big");
} bibẹkọ {             // else
    Irosu.fo("small");
}

// English equivalent:
if x > 10 {
    Irosu.fo("big");
} else {
    Irosu.fo("small");
}
```

### While

```ifa
nigba count < 5 {     // Yoruba
    count = count + 1;
}

while count < 5 {     // English
    count = count + 1;
}
```

### For

```ifa
fun item ninu items {     // Yoruba: for item in items
    Irosu.fo(item);
}

for item in items {        // English
    Irosu.fo(item);
}
```

### Break / Continue

```ifa
da;              // Yoruba: break
tesiwaju;        // Yoruba: continue
break;           // English
continue;        // English
```

### Match

```ifa
yan (status) {          // Yoruba
    200 => Irosu.fo("OK");
    404 => Irosu.fo("Not found");
    500 => Irosu.fo("Error");
    _   => Irosu.fo("Unknown");
}

match (status) {         // English
    200 => Irosu.fo("OK");
    404 => Irosu.fo("Not found");
    _   => Irosu.fo("Other");
}
```

Match arms support:
- Literal patterns (`42`, `"hello"`)
- Range patterns (`1..10`)
- Wildcard (`_`)

## Functions

```ifa
ese add(a, b) {          // Yoruba
    pada a + b;
}

fn subtract(a, b) {      // English
    return a - b;
}
```

Functions are first-class values. Can be stored in variables and passed as arguments.

### Async functions

```ifa
daro ese fetch_data(url) {     // daro = async
    reti Osa.ise("task");      // reti = await
    pada result;
}

async fn fetch_data(url) {     // English equivalent
    return await Osa.ise("task");
}
```

### Lambda expressions

```ifa
ayanmo double = (x) -> { pada x * 2; };
ayanmo triple = fn (x) { return x * 3; };
```

## Odù (Class) definitions

```ifa
odu Server {                     // Yoruba: odù = class
    ayanmo port = 8080;

    ese start() {
        Irosu.fo("Starting...");
    }
}

class Client {                   // English
    let timeout = 30;

    fn connect() {
        Irosu.fo("Connecting...");
    }
}
```

Public fields and methods:

```ifa
gbangba ese start() { ... }     // gbangba = public
public fn start() { ... }       // English
```

## Imports

```ifa
iba Irosu;                      // Yoruba: import
import Oturupon;                // English
iba std.otura;                  // Module path
iba { fo, so } lati Irosu;      // Named import: import { fo, so } from Irosu
```

Domains are imported by name (e.g. `Irosu`, `Obara`, `Ogbe`) or by module path (`std.otura`).

## Try / Catch / Finally

```ifa
gbiyanju {                    // Yoruba
    ayanmo result = risky();
} gba (e) {                   // catch
    Irosu.fo("Error: " + e);
} nipari {                    // finally
    Irosu.fo("Cleanup");
}

try {                         // English
    let result = risky();
} catch (e) {
    Irosu.fo("Error: " + e);
} finally {
    Irosu.fo("Cleanup");
}
```

## End of ritual

Every program ends with:

```ifa
ase;        // Yoruba: Àṣẹ
end;        // English
```

## Yield

```ifa
jowo 0;     // Yoruba: yield 0
yield 1;    // English
```

## Assert (Ewo)

```ifa
ewo condition, "message";    // Yoruba
assert condition, "message"; // English
verify condition;             // Also English
```

## Unsafe block

```ifa
ailewu {       // Yoruba
    // unsafe operations
}

unsafe {       // English
    // unsafe operations
}
```

## Memory / Opon directive

```ifa
opon: nla;          // Large memory
opon: kekere;       // Embedded/small
opon: arinrin;      // Default/medium
opon: ailopin;      // Unlimited/dynamic
```

## Taboo constraint

Declares that one Odù domain may not call another:

```ifa
ewo: Odi -> Otura;   // Odi cannot call Otura
```

## Comments on source

The grammar (`crates/ifa-parser/src/grammar.pest`) and AST definitions (`crates/ifa-types/src/ast.rs`) are the authoritative references for all syntax.
