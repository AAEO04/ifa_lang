# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║               IFA RUST TRANSPILER - UNIFIED V2                               ║
║                   "From Spirit to Steel"                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Combines the Rust runtime template from V1 with Lark AST-walking from V2.  ║
║  - If Lark available: Uses formal grammar + AST transformation              ║
║  - Fallback: Uses regex-based transpilation                                  ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import re
import os
import sys
import unicodedata
from typing import Any, Dict, List, Optional

# Try to import Lark
try:
    from lark import Transformer, Tree, Token
    LARK_AVAILABLE = True
except ImportError:
    LARK_AVAILABLE = False


# =============================================================================
# RUST RUNTIME TEMPLATE - Uses IfaValue for Dynamic Typing
# =============================================================================

RUST_RUNTIME = '''// ═══════════════════════════════════════════════════════════════════
// IFA-LANG RUST RUNTIME - Generated Code
// The Yoruba Programming Language - Dynamic Types via IfaValue
// ═══════════════════════════════════════════════════════════════════

mod core;
use core::{IfaValue, Opon, OPON};  // Include panic handler support
use std::collections::HashMap;

// Helper macros for IfaValue creation
macro_rules! ifa_int {
    ($x:expr) => { IfaValue::Int($x) };
}

macro_rules! ifa_str {
    ($x:expr) => { IfaValue::Str($x.to_string()) };
}

macro_rules! ifa_bool {
    ($x:expr) => { IfaValue::Bool($x) };
}

macro_rules! ifa_list {
    ($($x:expr),* $(,)?) => { IfaValue::List(vec![$($x),*]) };
}

macro_rules! ifa_map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut map = HashMap::new();
        $(map.insert($k.to_string(), $v);)*
        IfaValue::Map(map)
    }};
}

// ═══════════════════════════════════════════════════════════════════
// MAIN FUNCTION
// ═══════════════════════════════════════════════════════════════════
'''


# =============================================================================
# ODUN MAPPING - Ifá syntax -> Rust method
# =============================================================================

ODU_MAPPING = {
    # ═══════════════════════════════════════════════════════════════════
    # FULL DUAL-LEXICON MAPPING
    # All combinations work: Yoruba.yoruba, English.english, Mixed!
    # ═══════════════════════════════════════════════════════════════════
    
    # === OGBÈ / Init / Start / System ===
    "ogbe.bi": "opon::ogbe_bi", "ogbe.bere": "opon::ogbe_bi",
    "ogbe.gba": "opon::ogbe_gba",
    "init.create": "opon::ogbe_bi", "init.birth": "opon::ogbe_bi",
    "init.input": "opon::ogbe_gba", "start.input": "opon::ogbe_gba",
    "system.input": "opon::ogbe_gba", "system.env": "opon::ogbe_env",
    
    # === Ọ̀BÀRÀ / Add / Math / Plus ===
    "obara.ro": "opon::obara_fikun", "obara.fikun": "opon::obara_fikun",
    "add.sum": "opon::obara_fikun", "math.add": "opon::obara_fikun",
    "plus.add": "opon::obara_fikun",
    
    # === ÒTÚÚRÚPỌ̀N / Sub / Subtract / Minus ===
    "oturupon.din": "opon::oturupon_din", "oturupon.ge": "opon::ogunda_ge",
    "oturupon.ku": "opon::oturupon_ku",
    "sub.minus": "opon::oturupon_din", "subtract.minus": "opon::oturupon_din",
    "minus.sub": "opon::oturupon_din", "math.subtract": "opon::oturupon_din",
    "math.divide": "opon::ogunda_ge", "math.mod": "opon::oturupon_ku",
    
    # === ÌROSÙ / Log / Print / Out ===
    "irosu.fo": "opon::irosu_fo", "irosu.so": "opon::irosu_fo",
    "irosu.print": "opon::irosu_fo", "irosu.log": "opon::irosu_fo",
    "log.fo": "opon::irosu_fo", "log.print": "opon::irosu_fo",
    "log.say": "opon::irosu_fo", "log.write": "opon::irosu_fo",
    "print.fo": "opon::irosu_fo", "print.log": "opon::irosu_fo",
    "out.print": "opon::irosu_fo", "out.log": "opon::irosu_fo",
    
    # === ÒDÍ / File / Memory / Store / Database ===
    "odi.fi": "opon::odi_fi", "odi.gba": "opon::odi_gba",
    "odi.write": "opon::odi_fi", "odi.read": "opon::odi_gba",
    "file.fi": "opon::odi_fi", "file.write": "opon::odi_fi",
    "file.gba": "opon::odi_gba", "file.read": "opon::odi_gba",
    "memory.store": "opon::odi_fi", "memory.load": "opon::odi_gba",
    "store.save": "opon::odi_fi", "store.load": "opon::odi_gba",
    # SQLite Database Functions (Òdí - The Seal)
    "odi.si": "opon::odi_si", "odi.open": "opon::odi_si",
    "odi.pa_ase": "opon::odi_pa_ase", "odi.exec": "opon::odi_pa_ase",
    "odi.ka_inu": "opon::odi_ka_inu", "odi.query": "opon::odi_ka_inu",
    "odi.ti": "opon::odi_ti", "odi.close": "opon::odi_ti",
    "db.open": "opon::odi_si", "db.exec": "opon::odi_pa_ase",
    "db.query": "opon::odi_ka_inu", "db.close": "opon::odi_ti",
    "sql.open": "opon::odi_si", "sql.exec": "opon::odi_pa_ase",
    "sql.query": "opon::odi_ka_inu", "sql.close": "opon::odi_ti",
    
    # === ÒTÚRÁ / Net / Network / Http (RESTful) ===
    "otura.ran": "opon::otura_ran", "otura.de": "opon::otura_de",
    "otura.gba": "opon::otura_gba_http",
    "otura.send": "opon::otura_ran", "otura.bind": "opon::otura_de",
    # HTTP RESTful Client
    "otura.get": "opon::otura_gba_http", "otura.post": "opon::otura_ran_http",
    "otura.put": "opon::otura_fi_http", "otura.delete": "opon::otura_pa_http",
    "http.get": "opon::otura_gba_http", "http.post": "opon::otura_ran_http",
    "http.put": "opon::otura_fi_http", "http.delete": "opon::otura_pa_http",
    "net.ran": "opon::otura_ran", "net.send": "opon::otura_ran",
    "net.de": "opon::otura_de", "net.bind": "opon::otura_de",
    "net.gba": "opon::otura_gba_http", "net.recv": "opon::otura_gba_http",
    "net.get": "opon::otura_gba_http", "net.post": "opon::otura_ran_http",
    "network.send": "opon::otura_ran", "network.bind": "opon::otura_de",
    "api.get": "opon::otura_gba_http", "api.post": "opon::otura_ran_http",
    "api.put": "opon::otura_fi_http", "api.delete": "opon::otura_pa_http",
    
    # === Ọ̀ṢẸ́ / Draw / Graphics / UI ===
    "ose.ya": "opon::ose_ya", "ose.han": "opon::ose_han",
    "ose.nu": "opon::ose_nu",
    "ose.draw": "opon::ose_ya", "ose.show": "opon::ose_han",
    "draw.ya": "opon::ose_ya", "draw.pixel": "opon::ose_ya",
    "draw.show": "opon::ose_han", "draw.render": "opon::ose_han",
    "graphics.draw": "opon::ose_ya", "graphics.render": "opon::ose_han",
    "ui.draw": "opon::ose_ya", "ui.clear": "opon::ose_nu",
    
    # === ÌKÁ / Text / String / Str / JSON ===
    "ika.sopo": "opon::ika_sopo", "ika.ka": "opon::ika_ka",
    "ika.concat": "opon::ika_sopo", "ika.len": "opon::ika_ka",
    "text.sopo": "opon::ika_sopo", "text.concat": "opon::ika_sopo",
    "text.ka": "opon::ika_ka", "text.len": "opon::ika_ka",
    "string.concat": "opon::ika_sopo", "string.len": "opon::ika_ka",
    "str.concat": "opon::ika_sopo", "str.len": "opon::ika_ka",
    # JSON Functions (Ìká - The Controller)
    "ika.tu": "opon::ika_tu", "ika.parse": "opon::ika_tu",
    "ika.di": "opon::ika_di", "ika.stringify": "opon::ika_di",
    "ika.gba_inu": "opon::ika_gba_inu", "ika.get": "opon::ika_gba_inu",
    "json.parse": "opon::ika_tu", "json.tu": "opon::ika_tu",
    "json.stringify": "opon::ika_di", "json.di": "opon::ika_di",
    "json.get": "opon::ika_gba_inu",
    
    # === ÒGÚNDÁ / Array / List / Vec ===
    "ogunda.ge": "opon::ogunda_ge", "ogunda.fi": "opon::ogunda_fi",
    "ogunda.mu": "opon::ogunda_mu", "ogunda.ka": "opon::ogunda_ka",
    "ogunda.slice": "opon::ogunda_ge", "ogunda.push": "opon::ogunda_fi",
    "array.ge": "opon::ogunda_ge", "array.slice": "opon::ogunda_ge",
    "array.fi": "opon::ogunda_fi", "array.push": "opon::ogunda_fi",
    "array.mu": "opon::ogunda_mu", "array.pop": "opon::ogunda_mu",
    "array.ka": "opon::ogunda_ka", "array.len": "opon::ogunda_ka",
    "list.push": "opon::ogunda_fi", "list.pop": "opon::ogunda_mu",
    "list.len": "opon::ogunda_ka", "vec.push": "opon::ogunda_fi",
    
    # === Ọ̀WỌNRÍN / Rand / Random / Chaos ===
    "owonrin.bo": "opon::owonrin_bo",
    "owonrin.roll": "opon::owonrin_bo", "owonrin.random": "opon::owonrin_bo",
    "rand.bo": "opon::owonrin_bo", "rand.int": "opon::owonrin_bo",
    "random.bo": "opon::owonrin_bo", "random.int": "opon::owonrin_bo",
    "chaos.flip": "opon::owonrin_bo",
    
    # === ÌWÒRÌ / Time / Clock / ML ===
    "iwori.ago": "opon::iwori_ago", "iwori.duro": "opon::iwori_duro",
    "iwori.royin": "opon::iwori_royin", "iwori.nu": "opon::iwori_nu",
    "iwori.time": "opon::iwori_ago", "iwori.sleep": "opon::iwori_duro",
    "iwori.debug": "opon::iwori_royin", "iwori.clear": "opon::iwori_nu",
    "time.ago": "opon::iwori_ago", "time.now": "opon::iwori_ago",
    "time.duro": "opon::iwori_duro", "time.sleep": "opon::iwori_duro",
    "time.royin": "opon::iwori_royin", "time.debug": "opon::iwori_royin",
    "clock.now": "opon::iwori_ago", "clock.sleep": "opon::iwori_duro",
    "ml.debug": "opon::iwori_royin",
    
    # === Ọ̀YẸ̀KÚ / Exit / End / Halt ===
    "oyeku.duro": "opon::oyeku_duro", "oyeku.ku": "opon::oyeku_ku",
    "oyeku.halt": "opon::oyeku_duro", "oyeku.exit": "opon::oyeku_ku",
    "exit.duro": "opon::oyeku_duro", "exit.now": "opon::oyeku_ku",
    "end.halt": "opon::oyeku_duro", "halt.stop": "opon::oyeku_duro",
    "process.exit": "opon::oyeku_ku", "system.exit": "opon::oyeku_ku",
    
    # === Ọ̀SÁ / Async / Proc / Thread - Concurrency ===
    "osa.sa": "opon::osa_sa", "osa.spawn": "opon::osa_sa",
    "osa.duro": "opon::osa_duro", "osa.wait": "opon::osa_duro",
    "osa.ago": "opon::osa_ago", "osa.time": "opon::osa_ago",
    "async.spawn": "opon::osa_sa", "async.wait": "opon::osa_duro",
    "proc.spawn": "opon::osa_sa", "proc.wait": "opon::osa_duro",
    "thread.spawn": "opon::osa_sa", "thread.join": "opon::osa_duro",
    
    # === Ọ̀KÀNRÀN / Error / Except / Test - Error Handling ===
    "okanran.binu": "opon::okanran_binu", "okanran.throw": "opon::okanran_binu",
    "okanran.je": "opon::okanran_je", "okanran.assert": "opon::okanran_je",
    "error.throw": "opon::okanran_binu", "error.assert": "opon::okanran_je",
    "except.throw": "opon::okanran_binu", "except.assert": "opon::okanran_je",
    "test.assert": "opon::okanran_je", "test.fail": "opon::okanran_binu",
    
    # === ÌRẸTẸ̀ / Crypto / Hash / Zip - Compression & Hashing ===
    "irete.di": "opon::irete_di", "irete.hash": "opon::irete_di",
    "irete.fun": "opon::irete_fun", "irete.zip": "opon::irete_fun",
    "irete.tu": "opon::irete_tu", "irete.unzip": "opon::irete_tu",
    "crypto.hash": "opon::irete_di", "crypto.compress": "opon::irete_fun",
    "hash.di": "opon::irete_di", "hash.compute": "opon::irete_di",
    "zip.compress": "opon::irete_fun", "zip.decompress": "opon::irete_tu",
    
    # === ÒFÚN / Meta / Reflect / Root - Permissions & Reflection ===
    "ofun.ase": "opon::ofun_ase", "ofun.sudo": "opon::ofun_ase",
    "ofun.fun": "opon::ofun_fun", "ofun.grant": "opon::ofun_fun",
    "ofun.ka": "opon::ofun_ka", "ofun.config": "opon::ofun_ka",
    "ofun.iru": "opon::ofun_iru", "ofun.typeof": "opon::ofun_iru",
    "meta.type": "opon::ofun_iru", "meta.config": "opon::ofun_ka",
    "reflect.typeof": "opon::ofun_iru", "root.sudo": "opon::ofun_ase",
    
    # === REGEX Extensions (Ìká) ===
    "ika.wa": "opon::ika_wa", "ika.find": "opon::ika_wa",
    "ika.po": "opon::ika_po", "ika.match": "opon::ika_po",
    "regex.find": "opon::ika_wa", "regex.match": "opon::ika_po",
    "regex.search": "opon::ika_wa", "regex.capture": "opon::ika_po",
    
    # === DATE Extensions (Ìwòrì) ===
    "iwori.ojo": "opon::iwori_ojo", "iwori.date": "opon::iwori_ojo",
    "date.format": "opon::iwori_ojo", "time.format": "opon::iwori_ojo",
    
    # === ENV Extensions (Ogbè) ===
    "ogbe.ayika": "opon::ogbe_ayika", "ogbe.env": "opon::ogbe_ayika",
    "env.get": "opon::ogbe_ayika", "os.getenv": "opon::ogbe_ayika",
    "ogbe.awo": "opon::ogbe_awo", "ogbe.secret": "opon::ogbe_awo",
    "env.secret": "opon::ogbe_awo", "secret.get": "opon::ogbe_awo",
}


# =============================================================================
# LARK-BASED TRANSFORMER (Preferred path)
# =============================================================================

if LARK_AVAILABLE:
    class LarkRustTransformer(Transformer):
        """
        Walks AST bottom-up and generates Rust code.
        """
        
        def __init__(self):
            super().__init__()
            self.indent_level = 1
            self.variables: Dict[str, str] = {}
        
        def _indent(self) -> str:
            return "    " * self.indent_level
        
        # === ROOT ===
        def start(self, items):
            body = "\n".join([str(item) for item in items if item])
            return f'''{RUST_RUNTIME}
fn main() {{
    let mut opon = Opon::new();
    
{body}
    
    println!("\\nÀṣẹ! (Success)");
}}
'''
        
        # === STATEMENTS ===
        # === STATEMENTS ===
        def statement(self, items):
            if items:
                stmt_code = str(items[0])
                # Source Map Injection
                # Try to get line info from the AST node
                try:
                    meta = items[0].meta
                    line = meta.line
                    # Rust supports #[line] but comments are safer for now
                    # We inject a comment that the panic handler can parse
                    return f"{self._indent()}// IFA_LINE:{line}\n{stmt_code}"
                except AttributeError:
                    pass
                return stmt_code
            return ""
        
        def import_stmt(self, items):
            path = items[0] if items else ""
            
            # Handle list of tokens or single token
            if isinstance(path, list):
                # Check for native import: ìbà àlejò.modname
                if len(path) >= 2 and str(path[0]).lower() in ["alejo", "àlejò"]:
                    mod_name = str(path[1])
                    # We will collect these to inform the build system
                    # For now, we just emit the Rust mod declaration
                    return f"{self._indent()}mod {mod_name};\n{self._indent()}use {mod_name}::*;"
                
                rust_path = "::".join([str(p) for p in path])
            else:
                rust_path = str(path)
                
            return f"{self._indent()}// use {rust_path};"
        
        def module_path(self, items):
            return [str(item) for item in items]
        
        def var_decl(self, items):
            name = str(items[0])
            value = str(items[1]) if len(items) > 1 else "0"
            self.variables[name] = "i64"
            return f"{self._indent()}let mut {name}: i64 = {value};"
        
        def instruction(self, items):
            call = items[0] if items else ""
            return f"{self._indent()}{call};"
        
        def return_stmt(self, items):
            value = str(items[0]) if items else ""
            return f"{self._indent()}return {value};" if value else f"{self._indent()}return;"
        
        def end_stmt(self, items=None):
            return f"{self._indent()}opon.oyeku_duro();"
        
        # === CONTROL FLOW ===
        def if_stmt(self, items):
            condition = str(items[0])
            then_body = items[1] if len(items) > 1 else []
            else_body = items[2] if len(items) > 2 else None
            
            result = f"{self._indent()}if {condition} {{\n"
            self.indent_level += 1
            for stmt in (then_body if isinstance(then_body, list) else [then_body]):
                if stmt: result += f"{stmt}\n"
            self.indent_level -= 1
            result += f"{self._indent()}}}"
            
            if else_body:
                result += f" else {{\n"
                self.indent_level += 1
                for stmt in (else_body if isinstance(else_body, list) else [else_body]):
                    if stmt: result += f"{stmt}\n"
                self.indent_level -= 1
                result += f"{self._indent()}}}"
            
            return result
        
        def else_clause(self, items):
            return list(items)
        
        def while_stmt(self, items):
            condition = str(items[0])
            body = items[1:] if len(items) > 1 else []
            
            result = f"{self._indent()}while {condition} {{\n"
            self.indent_level += 1
            for stmt in body:
                if stmt: result += f"{stmt}\n"
            self.indent_level -= 1
            result += f"{self._indent()}}}"
            return result
        
        # === MATCH STATEMENT (yàn) ===
        def match_stmt(self, items):
            """
            Transpile yàn/match to Rust match.
            items[0] = expression to match
            items[1:] = match arms
            """
            expr = str(items[0])
            arms = items[1:] if len(items) > 1 else []
            
            result = f"{self._indent()}match {expr} {{\n"
            self.indent_level += 1
            
            for arm in arms:
                if arm:
                    result += f"{arm}\n"
            
            self.indent_level -= 1
            result += f"{self._indent()}}}"
            return result
        
        def match_arm(self, items):
            """
            Transpile a single match arm: pattern => action
            items[0] = pattern (or "_" for default)
            items[1] = action/statement
            """
            pattern = str(items[0])
            action = str(items[1]) if len(items) > 1 else ""
            
            # Handle wildcard default case
            if pattern == "_":
                return f"{self._indent()}_ => {{ {action} }},"
            
            # Wrap integer patterns in IfaValue::Int
            if pattern.isdigit():
                return f"{self._indent()}IfaValue::Int({pattern}) => {{ {action} }},"
            
            # Wrap string patterns in IfaValue::Str
            if pattern.startswith('"') or pattern.startswith("'"):
                return f'{self._indent()}IfaValue::Str({pattern}.to_string()) => {{ {action} }},'
            
            # Default: treat as variable binding or literal
            return f"{self._indent()}{pattern} => {{ {action} }},"
        
        # === LAMBDA EXPRESSION (arrow function) ===
        def lambda_expr(self, items):
            """
            Transpile (params) -> { body } to Rust closure wrapped in IfaValue::Fn.
            items[0] = params (optional)
            items[1:] = body statements
            """
            params = items[0] if items and isinstance(items[0], list) else []
            body_start = 1 if params else 0
            body = items[body_start:] if len(items) > body_start else []
            
            # Generate parameter unpacking
            param_unpack = ""
            for i, param in enumerate(params):
                param_name = str(param)
                param_unpack += f"    let {param_name} = args.get({i}).unwrap_or(&IfaValue::Null).clone();\n"
            
            # Generate body
            body_code = "\n".join([f"    {stmt}" for stmt in body if stmt])
            
            # Wrap in IfaValue::Fn with Rc closure
            return f"""IfaValue::Fn(Rc::new(move |args: Vec<IfaValue>| {{
{param_unpack}{body_code}
}}))"""
        
        # === CALLS ===
        def odu_call(self, items):
            odu = str(items[0]).lower()
            ese = str(items[1]).lower() if len(items) > 1 else ""
            args = items[2] if len(items) > 2 else []
            
            key = f"{odu}.{ese}"
            rust_func = ODU_MAPPING.get(key, f"{odu}_{ese}")
            
            if isinstance(args, list):
                arg_str = ", ".join([str(a) for a in args])
            else:
                arg_str = str(args) if args else ""
            
            return f"{rust_func}({arg_str})"
        
        def odu_name(self, items):
            return str(items[0]) if items else ""
        
        def ese_name(self, items):
            return str(items[0]) if items else ""
        
        def arguments(self, items):
            return list(items)
        
        # === EXPRESSIONS ===
        def expression(self, items):
            return items[0] if items else ""
        
        def or_expr(self, items):
            return " || ".join([str(item) for item in items])
        
        def and_expr(self, items):
            return " && ".join([str(item) for item in items])
        
        def not_expr(self, items):
            return items[0] if len(items) == 1 else f"!{items[0]}"
        
        def comparison(self, items):
            if len(items) == 1: return items[0]
            return f"{items[0]} {items[1]} {items[2]}"
        
        def arith_expr(self, items):
            if len(items) == 1: return items[0]
            result = str(items[0])
            i = 1
            while i < len(items):
                result = f"({result} {items[i]} {items[i+1]})"
                i += 2
            return result
        
        def term(self, items):
            if len(items) == 1: return items[0]
            result = str(items[0])
            i = 1
            while i < len(items):
                result = f"({result} {items[i]} {items[i+1]})"
                i += 2
            return result
        
        def factor(self, items):
            return items[0] if len(items) == 1 else f"{items[0]}{items[1]}"
        
        def atom(self, items):
            return items[0] if items else ""
        
        # === STRUCTURES ===
        def odu_def(self, items):
            """
            Transpile Odù (Class) definition to Rust struct + impl.
            Structure: [PUBLIC_MOD?] ODU_KW NAME "{" body "}"
            
            Generates:
              - struct ClassName { fields }
              - impl ClassName { fn da(...) -> Self, methods }
            """
            is_public = ""
            name_idx = 1
            
            # Check for optional public modifier
            if len(items) > 0 and str(items[0]) in ["gbangba", "public"]:
                is_public = "pub "
                name_idx = 2
                
            name = str(items[name_idx])
            body_items = items[name_idx+1:]
            
            # Separate fields and methods from body
            fields = []
            methods = []
            constructor_params = []
            
            for item in body_items:
                item_str = str(item) if item else ""
                if not item_str.strip():
                    continue
                    
                # Check if it's a method (fn ...)
                if item_str.strip().startswith("fn ") or item_str.strip().startswith("pub fn "):
                    # Extract method name to check if it's a constructor
                    if " da(" in item_str or " dá(" in item_str or " new(" in item_str:
                        # Parse constructor params for struct fields
                        import re
                        match = re.search(r'\(([^)]*)\)', item_str)
                        if match:
                            params = match.group(1)
                            for param in params.split(','):
                                param = param.strip()
                                if param and ':' in param:
                                    param_name = param.split(':')[0].strip()
                                    param_type = param.split(':')[1].strip() if ':' in param else "IfaValue"
                                    constructor_params.append((param_name, param_type))
                                    fields.append(f"    {is_public}{param_name}: {param_type},")
                        # Replace constructor name with 'new'
                        item_str = item_str.replace(" da(", " new(").replace(" dá(", " new(")
                        # Add -> Self return type if not present
                        if "-> Self" not in item_str:
                            item_str = item_str.replace(") {", ") -> Self {")
                        methods.append(self._indent() + "    " + item_str.strip())
                    else:
                        methods.append(self._indent() + "    " + item_str.strip())
                else:
                    # Treat as field declaration
                    if ":" in item_str:
                        fields.append(f"    {is_public}{item_str.strip()}")
                    elif "=" in item_str:
                        # ayanmo field = value; -> field: IfaValue
                        parts = item_str.replace("let ", "").replace("ayanmo ", "").split("=")
                        field_name = parts[0].strip()
                        fields.append(f"    {is_public}{field_name}: IfaValue,")
            
            # Build struct
            struct_fields = "\n".join(fields) if fields else "    // No fields"
            result = f"{self._indent()}{is_public}struct {name} {{\n{struct_fields}\n{self._indent()}}}\n\n"
            
            # Build impl block with constructor
            result += f"{self._indent()}impl {name} {{\n"
            
            # Add default constructor if none exists
            if not any("fn new(" in m for m in methods):
                param_init = ", ".join([f"{p[0]}: {p[1]}" for p in constructor_params]) if constructor_params else ""
                field_init = ", ".join([f"{p[0]}" for p in constructor_params]) if constructor_params else ""
                result += f"    {is_public}fn new({param_init}) -> Self {{\n"
                result += f"        Self {{ {field_init} }}\n"
                result += f"    }}\n"
            
            # Add methods
            for method in methods:
                result += method + "\n"
            
            result += f"{self._indent()}}}"
            
            return result
        
        def ese_def(self, items):
            """
            Transpile Ẹsẹ (Function/Method) definition.
            Structure: [PUBLIC_MOD?] ESE_KW NAME "(" params? ")" "{" body "}"
            """
            is_public = ""
            name_idx = 1
            
            # Check for optional public modifier
            if len(items) > 0 and str(items[0]) in ["gbangba", "public"]:
                is_public = "pub "
                name_idx = 2
                
            name = str(items[name_idx])
            
            # Find params and body
            # We need to scan remaining items
            params = ""
            body_start = name_idx + 1
            
            # Check if next item is params (list)
            if body_start < len(items) and isinstance(items[body_start], list):
                params_list = items[body_start]
                params = ", ".join(params_list)
                body_start += 1
                
            body_items = items[body_start:]
            body = "\n".join([str(item) for item in body_items if item])
            
            return f"{self._indent()}{is_public}fn {name}({params}) {{\n{body}\n{self._indent()}}}"

        # === LITERALS ===
        def NUMBER(self, token): return str(token)
        def FLOAT(self, token): return f"{token}f64"
        def STRING(self, token): return str(token)
        def BOOLEAN(self, token): return "true" if str(token).lower() in ("true", "otito") else "false"
        def NAME(self, token): return str(token)
        def PUBLIC_MOD(self, token): return str(token)
        
        # Odù terminals
        def ODU_OGBE(self, _): return "ogbe"
        def ODU_OYEKU(self, _): return "oyeku"
        def ODU_IWORI(self, _): return "iwori"
        def ODU_ODI(self, _): return "odi"
        def ODU_IROSU(self, _): return "irosu"
        def ODU_OWONRIN(self, _): return "owonrin"
        def ODU_OBARA(self, _): return "obara"
        def ODU_OKANRAN(self, _): return "okanran"
        def ODU_OGUNDA(self, _): return "ogunda"
        def ODU_OSA(self, _): return "osa"
        def ODU_IKA(self, _): return "ika"
        def ODU_OTURUPON(self, _): return "oturupon"
        def ODU_OTURA(self, _): return "otura"
        def ODU_IRETE(self, _): return "irete"
        def ODU_OSE(self, _): return "ose"
        def ODU_OFUN(self, _): return "ofun"
        
        # Operators
        def PLUS(self, _): return "+"
        def MINUS(self, _): return "-"
        def STAR(self, _): return "*"
        def SLASH(self, _): return "/"
        def PERCENT(self, _): return "%"
        def COMP_OP(self, token): return str(token)


# =============================================================================
# UNIFIED TRANSPILER CLASS
# =============================================================================

class IfaRustTranspiler:
    """
    Unified Ifá-Lang to Rust transpiler.
    - Uses Lark AST when available (formal grammar)
    - Falls back to regex-based parsing
    """
    
    def __init__(self):
        self._parser = None
        self._transformer = LarkRustTransformer() if LARK_AVAILABLE else None
    
    @property
    def parser(self):
        """Lazy-load the Lark parser."""
        if self._parser is None and LARK_AVAILABLE:
            try:
                from src.lark_parser import IfaLarkParser
                self._parser = IfaLarkParser()
            except ImportError:
                self._parser = None
        return self._parser
    
    def normalize(self, text: str) -> str:
        """Normalize Yoruba text for mapping."""
        text = unicodedata.normalize('NFD', text.lower())
        result = ""
        for char in text:
            if unicodedata.category(char) != 'Mn':
                if char == 'ọ': result += 'o'
                elif char == 'ẹ': result += 'e'
                elif char == 'ṣ': result += 's'
                else: result += char
        return result
    
    def transpile(self, source: str) -> str:
        """Transpile Ifá source to Rust code."""
        
        # Try Lark-based transpilation first
        if self.parser and self._transformer:
            try:
                ast = self.parser.parse(source)
                return self._transpile_ast(ast)
            except Exception as e:
                print(f"[WARN] Lark parsing failed: {e}")
                print("       Falling back to regex-based transpilation...")
        
        # Fallback to regex-based
        return self._transpile_regex(source)
    
    def _transpile_ast(self, ast) -> str:
        """Transpile from AST nodes."""
        if hasattr(ast, 'statements'):
            return self._transpile_program(ast)
        else:
            return self._transformer.transform(ast)
    
    def _transpile_program(self, program) -> str:
        """Transpile a Program node."""
        body_lines = []
        indent = "    "
        
        for stmt in program.statements:
            line = self._transpile_stmt(stmt, indent)
            if line:
                body_lines.append(line)
        
        body = "\n".join(body_lines)
        
        return f'''{RUST_RUNTIME}
fn main() {{
    // Initialize Opon with panic handler for automatic crash dump
    let mut opon = Opon::new_with_panic_handler();
    
{body}
    
    println!("\\nÀṣẹ! (Success)");
}}
'''
    
    def _transpile_stmt(self, stmt, indent: str = "    ") -> str:
        """Transpile a statement node."""
        node_type = type(stmt).__name__
        
        if node_type == "ImportStmt":
            path = "::".join(stmt.path)
            return f"{indent}// use {path};"
        
        elif node_type == "VarDecl":
            value = self._transpile_expr(stmt.value)
            
            # ORI SYSTEM: Check for type hint to determine native vs dynamic
            if hasattr(stmt, 'type_hint') and stmt.type_hint:
                # Native type - FAST PATH
                type_map = {
                    'Int': 'i64',
                    'Float': 'f64',
                    'Str': 'String',
                    'Bool': 'bool',
                    'List': 'Vec<IfaValue>',
                    'Map': 'HashMap<String, IfaValue>',
                    'Any': 'IfaValue',
                }
                rust_type = type_map.get(stmt.type_hint, 'IfaValue')
                
                # Generate native literal (unwrap IfaValue macro)
                if stmt.type_hint == 'Int' and value.startswith('ifa_int!('):
                    # Extract raw value: ifa_int!(10) -> 10
                    native_val = value[9:-1]  # Remove 'ifa_int!(' and ')'
                    return f"{indent}let mut {stmt.name}: {rust_type} = {native_val};"
                elif stmt.type_hint == 'Float' and 'Float' in value:
                    native_val = value.replace('IfaValue::Float(', '').rstrip(')')
                    return f"{indent}let mut {stmt.name}: {rust_type} = {native_val};"
                elif stmt.type_hint == 'Str' and value.startswith('ifa_str!('):
                    native_val = value[9:-1]  # Remove 'ifa_str!(' and ')'
                    return f"{indent}let mut {stmt.name}: {rust_type} = {native_val}.to_string();"
                elif stmt.type_hint == 'Bool' and value.startswith('ifa_bool!('):
                    native_val = value[10:-1]
                    return f"{indent}let mut {stmt.name}: {rust_type} = {native_val};"
                else:
                    # Fallback: still typed but using value as-is
                    return f"{indent}let mut {stmt.name}: {rust_type} = {value};"
            else:
                # Dynamic type - FLEXIBLE PATH (IfaValue)
                return f"{indent}let mut {stmt.name} = {value};"
        
        elif node_type == "Instruction":
            call = self._transpile_call(stmt.call)
            return f"{indent}{call};"
        
        elif node_type == "OduCall":
            return f"{indent}{self._transpile_call(stmt)};"
        
        elif node_type == "IfStmt":
            cond = self._transpile_expr(stmt.condition)
            lines = [f"{indent}if {cond} {{"]
            for s in stmt.then_body:
                lines.append(self._transpile_stmt(s, indent + "    "))
            lines.append(f"{indent}}}")
            if stmt.else_body:
                lines[-1] += " else {"
                for s in stmt.else_body:
                    lines.append(self._transpile_stmt(s, indent + "    "))
                lines.append(f"{indent}}}")
            return "\n".join(lines)
        
        elif node_type == "WhileStmt":
            cond = self._transpile_expr(stmt.condition)
            lines = [f"{indent}while {cond} {{"]
            for s in stmt.body:
                lines.append(self._transpile_stmt(s, indent + "    "))
            lines.append(f"{indent}}}")
            return "\n".join(lines)
        
        elif node_type == "ReturnStmt":
            val = self._transpile_expr(stmt.value) if stmt.value else ""
            return f"{indent}return {val};" if val else f"{indent}return;"
        
        elif node_type == "EndStmt":
            return f"{indent}opon.oyeku_duro();"
        
        elif node_type == "TabooStmt":
            # Taboos are compile-time checks, emit as comment
            if stmt.is_wildcard:
                return f"{indent}// [ÈÈWỌ̀] Taboo: {stmt.source_domain}.* forbidden"
            else:
                src = f"{stmt.source_domain}({stmt.source_context})" if stmt.source_context else stmt.source_domain
                tgt = f"{stmt.target_domain}({stmt.target_context})" if stmt.target_context else stmt.target_domain
                return f"{indent}// [ÈÈWỌ̀] Taboo: {src} -> {tgt} forbidden"
        
        elif node_type == "AseBlock":
            # Critical/Atomic block with transaction semantics
            lines = [
                f"{indent}// ═══ ÀṢẸ BLOCK (Critical Section) ═══",
                f"{indent}let _ase_checkpoint = opon.memory.clone();  // Snapshot for rollback",
                f"{indent}let _ase_result: Result<(), String> = (|| {{",
            ]
            for s in stmt.body:
                lines.append(self._transpile_stmt(s, indent + "    "))
            lines.extend([
                f"{indent}    Ok(())",
                f"{indent}}})();",
                f"{indent}if _ase_result.is_err() {{",
                f"{indent}    opon.memory = _ase_checkpoint;  // Rollback on failure",
                f"{indent}    eprintln!(\"[Ọ̀kànràn] Àṣẹ block failed, rolled back\");",
                f"{indent}}}",
                f"{indent}// ═══ END ÀṢẸ BLOCK ═══",
            ])
            return "\n".join(lines)
        
        elif node_type == "ForStmt":
            iterable = self._transpile_expr(stmt.iterable)
            var = stmt.var_name
            lines = [f"{indent}for {var} in {iterable}.into_iter() {{"]
            for s in stmt.body:
                lines.append(self._transpile_stmt(s, indent + "    "))
            lines.append(f"{indent}}}")
            return "\n".join(lines)
        
        elif node_type == "TryStmt":
            lines = [
                f"{indent}// Try block",
                f"{indent}let _try_result: Result<(), String> = (|| {{",
            ]
            for s in stmt.try_body:
                lines.append(self._transpile_stmt(s, indent + "    "))
            lines.extend([
                f"{indent}    Ok(())",
                f"{indent}}})();",
                f"{indent}if let Err({stmt.error_var}) = _try_result {{",
            ])
            for s in stmt.catch_body:
                lines.append(self._transpile_stmt(s, indent + "    "))
            lines.append(f"{indent}}}")
            return "\n".join(lines)
        
        return f"{indent}// Unknown: {node_type}"
    
    def _transpile_call(self, call) -> str:
        """Transpile an OduCall."""
        odu = call.odu.lower()
        ese = call.ese.lower()
        key = f"{odu}.{ese}"
        
        rust_func = ODU_MAPPING.get(key, f"{odu}_{ese}")
        args = [self._transpile_expr(a) for a in call.args]
        arg_str = ", ".join(args)
        
        return f"{rust_func}({arg_str})"
    
    def _transpile_expr(self, expr) -> str:
        """Transpile an expression with IfaValue wrapping."""
        node_type = type(expr).__name__
        
        if node_type == "Literal":
            if expr.type == "string":
                return f'ifa_str!("{expr.value}")'
            elif expr.type == "boolean":
                return f'ifa_bool!({str(expr.value).lower()})'
            elif expr.type == "float":
                return f'IfaValue::Float({expr.value})'
            else:  # number/int
                return f'ifa_int!({expr.value})'
        
        elif node_type == "Identifier":
            return f'{expr.name}.clone()'
        
        elif node_type == "BinaryOp":
            left = self._transpile_expr(expr.left)
            right = self._transpile_expr(expr.right)
            # Use operator overloading for +, -, *, /
            if expr.op in ('+', '-', '*', '/'):
                return f'({left} {expr.op} {right})'
            # Use compare method for comparison operators
            elif expr.op in ('==', '!=', '<', '<=', '>', '>='):
                return f'{left}.compare(&{right}, "{expr.op}")'
            # Logical operators
            elif expr.op == '&&':
                return f'IfaValue::Bool({left}.is_truthy() && {right}.is_truthy())'
            elif expr.op == '||':
                return f'IfaValue::Bool({left}.is_truthy() || {right}.is_truthy())'
            return f'({left} {expr.op} {right})'
        
        elif node_type == "UnaryOp":
            operand = self._transpile_expr(expr.operand)
            if expr.op == '-':
                return f'(-{operand})'
            elif expr.op == '!':
                return f'(!{operand})'
            return f'{expr.op}{operand}'
        
        elif node_type == "OduCall":
            return self._transpile_call(expr)
        
        elif node_type == "ListLiteral":
            elements = [self._transpile_expr(e) for e in expr.elements]
            return f'ifa_list![{", ".join(elements)}]'
        
        elif node_type == "MapLiteral":
            entries = [f'"{k}" => {self._transpile_expr(v)}' for k, v in expr.entries.items()]
            return f'ifa_map![{", ".join(entries)}]'
        
        elif node_type == "IndexAccess":
            target = expr.target
            index = self._transpile_expr(expr.index)
            return f'{target}.get(&{index})'
        
        return str(expr)
    
    def _transpile_regex(self, source: str) -> str:
        """Fallback regex-based transpilation."""
        print(">>> Transpiling to Rust (Regex Fallback)...")
        
        rust_main = [
            "fn main() {",
            "    let mut opon = Opon::new();",
            ""
        ]
        
        lines = source.replace('\n', ';').split(';')
        lines = [l.strip() for l in lines if l.strip()]
        
        for line in lines:
            if line.startswith(('iba', 'ìbà', '//', '#')):
                continue
            
            # Variable assignment
            match = re.search(r'ayanmo\s+(\w+)\s*=\s*(.+)', line)
            if match:
                name, val = match.groups()
                rust_main.append(f"    let mut {name}: i64 = {val.strip()};")
                continue
            
            # Method call
            match = re.search(r'([\w\u00C0-\u017F]+)\.([\w\u00C0-\u017F]+)\(([^)]*)\)', line)
            if match:
                cls_raw, mth_raw, arg_val = match.groups()
                clean_key = self.normalize(f"{cls_raw}.{mth_raw}")
                rust_func = ODU_MAPPING.get(clean_key)
                
                if rust_func:
                    rust_main.append(f"    {rust_func}({arg_val or ''});")
                else:
                    rust_main.append(f"    // Unknown: {clean_key}")
                continue
            
            # End statement
            if self.normalize(line) in ('ase', 'àṣẹ'):
                rust_main.append("    opon.oyeku_duro();")
        
        rust_main.append('    println!("\\nÀṣẹ! (Success)");')
        rust_main.append("}")
        
        return RUST_RUNTIME + "\n".join(rust_main)
    
    def transpile_file(self, input_path: str, output_path: str = None) -> str:
        """Transpile a .ifa file to .rs"""
        with open(input_path, 'r', encoding='utf-8') as f:
            source = f.read()
        
        rust_code = self.transpile(source)
        
        if output_path is None:
            output_path = input_path.replace('.ifa', '.rs')
            if output_path == input_path:
                output_path = 'main.rs'
        
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(rust_code)
        
        print(f"[SUCCESS] Generated '{output_path}'")
        print(f"  Compile with: rustc {output_path}")
        
        return output_path


# Alias for compatibility
IfaRustTranspilerV2 = IfaRustTranspiler


# =============================================================================
# CLI
# =============================================================================

if __name__ == "__main__":
    print("""
╔══════════════════════════════════════════════════════════════╗
║            IFA RUST TRANSPILER (Unified)                     ║
╠══════════════════════════════════════════════════════════════╣
║  Lark AST: """ + ("✓ Available" if LARK_AVAILABLE else "✗ Not installed") + """                                     ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    if len(sys.argv) < 2:
        # Demo
        demo = '''
iba std.otura;

ayanmo port = 8080;

Irosu.fo("Starting Server...");
Otura.ran("Hello from Ifá!");

ase;
'''
        print("=== Demo Transpilation ===")
        transpiler = IfaRustTranspiler()
        rust = transpiler.transpile(demo)
        print(rust)
    else:
        input_file = sys.argv[1]
        output_file = sys.argv[2] if len(sys.argv) > 2 else None
        
        transpiler = IfaRustTranspiler()
        transpiler.transpile_file(input_file, output_file)
