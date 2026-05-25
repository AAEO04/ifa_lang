# Good Features — Straightforward Additions

Features with clear semantics that slot into the existing pipeline without major architectural changes.

---

## 1. Pipeline Operator `|>`

**Syntax:** `expr |> func` or `expr |> func(arg1, ...)`

Inspired by Elixir/Elm/F#. Feeds the left-hand value as the first argument to the right-hand call.

**Examples:**
```
"hello" |> Ose.to_uppercase() |> Ika.reverse()
# Equivalent to: Ika.reverse(Ose.to_uppercase("hello"))

users |> Owo.fun.filter(.age > 18) |> Owo.fun.map(.name)
```

**Implementation:**
1. Add `Expression::Pipeline { lhs, rhs, span }` to AST
2. Parse `|>` as a binary operator (lowest precedence, right after null-coalesce)
3. Compiler desugars: `lhs |> rhs` → desugar `rhs` into a call with `lhs` prepended to args
   - If `rhs` is a `Call(name, args)`, rewrite to `Call(name, [lhs, ...args])`
   - If `rhs` is a bare `Ident`, rewrite to `Call(rhs, [lhs])`
4. No new opcodes needed

**Grammar (pest):**
```pest
pipeline_expr = { null_coalesce_expr ~ ("|>" ~ null_coalesce_expr)* }
expression = { pipeline_expr }
```

---

## 2. Domain-Typed Strings

**Concept:** String literals tagged with an Odù domain, checked at the type level.

**Syntax:**
```
let name: Ogbe = ogbe"system_name";
let key: Otura = otura"config.key";
```

The prefix is lowercase Odu name. At parse time, the string carries a domain tag. At the type level, `ogbe"foo"` has type `Ogbe` (not `Str`), preventing accidental mixing:

```ifa
let a: Ogbe = ogbe"sys";
let b: Otura = otura"cfg";
let c = a ++ b;  // Babalawo error: cannot concat Ogbe + Otura
```

**Implementation:**
1. `TypeHint` gains `OduDomain(u8)` variant (domain id + string)
2. Parser: `domain_ident ~ string_literal` produces `Expression::StrLit { value, domain: Some(id) }`
3. Type inference: domain-typed string has type `OduDomain(id)`. Compatible only with same domain or `Str`
4. Babalawo: warn on mixing domains without explicit `.to_str()` cast
5. No VM changes (strings are still `IfaValue::Str` at runtime — domain tag is compile-time only)

---

## 3. Elm-Style `did_you_mean` Suggestions

**Current:** Elm-style error formatting with source-line rendering and caret pointers exists.

**Extension:** When a name lookup fails, suggest the closest matching name in scope using Levenshtein distance.

**Examples:**
```
error: Undefined variable 'prnit'
  --> examples/test.ifa:3:5
  |
3 |     prnit("hello");
  |     ^^^^^^
  |
  note: Did you mean 'print'?
```

**Implementation:**
1. In Babalawo's `resolve_name()` or `check_expression()` for `Expression::Ident`:
   - On lookup failure, compute Levenshtein distance against all visible names in current scope
   - If closest match is within threshold (distance <= 3 or name length / 3), add a `note` to the `LintError`
2. In `format_with_source()`, render `notes` as `note:` lines after the caret block
3. Only suggest names from the same scope (not globals or domain methods)

---

## 4. Lexical `ewo` Scopes

**Concept:** The `ewo` (assert) keyword creates a lexical scope where constraints are enforced at compile time via Babalawo.

**Already partially implemented.** `Statement::Ewo` exists in the AST. The enhancement is:
- Track variable constraints through the scope
- Verify that constrained variables aren't reassigned to invalid values
- Allow `ewo` blocks as expressions returning their body's last value

**Syntax:**
```
let x = get_value();
ewo x > 0;                    // assert at runtime, checked at compile-time
ewo (x > 0) { process(x); }   // scoped block where x > 0 is guaranteed
```

**Implementation:**
1. `Expression::Ewo` variant (expression form): `ewo condition { body }` evaluates to body's result
2. Babalawo tracks `VarConstraint { var: String, predicate: Expression }` per scope
3. On reassignment to constrained variable, re-check constraint satisfaction
4. No VM changes (runtime check still compiles to assertion opcode)
