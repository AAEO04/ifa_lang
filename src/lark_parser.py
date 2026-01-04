# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ-LANG FORMAL PARSER                                    ║
║                    Lark-Based EBNF Grammar Implementation                    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Replaces fragile regex-based parsing with formal grammar.                   ║
║  Uses Lark: https://lark-parser.readthedocs.io/                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import sys
from typing import Any, Dict, List, Optional, Union
from dataclasses import dataclass, field

# Try to import Lark
try:
    from lark import Lark, Transformer, v_args, Token, Tree
    LARK_AVAILABLE = True
except ImportError:
    LARK_AVAILABLE = False
    print("Warning: Lark not installed. Run: pip install lark")


# =============================================================================
# AST NODE DEFINITIONS
# =============================================================================

@dataclass
class ASTNode:
    """Base class for all AST nodes."""
    line: int = 0
    column: int = 0


@dataclass
class Program(ASTNode):
    """Root node containing all statements."""
    statements: List[ASTNode] = field(default_factory=list)


@dataclass
class ImportStmt(ASTNode):
    """Import statement (ìbà module.path;)"""
    path: List[str] = field(default_factory=list)


@dataclass
class VarDecl(ASTNode):
    """Variable declaration with optional type hint.
    
    Dynamic (Default):  ayanmo x = 50;           → IfaValue (flexible)
    Destined (Typed):   ayanmo x: Int = 50;      → i64 (native speed)
    """
    name: str = ""
    value: Any = None
    type_hint: Optional[str] = None  # None = dynamic, "Int"/"Str"/etc = static


@dataclass
class OduCall(ASTNode):
    """Method call (Otura.ran("hello"))"""
    odu: str = ""
    ese: str = ""
    args: List[Any] = field(default_factory=list)


@dataclass
class Instruction(ASTNode):
    """Instruction statement (odu_call;)"""
    call: OduCall = None


@dataclass
class OduDef(ASTNode):
    """Class/Module definition (odù Name { ... })"""
    name: str = ""
    methods: List['EseDef'] = field(default_factory=list)
    variables: List[VarDecl] = field(default_factory=list)


@dataclass
class EseDef(ASTNode):
    """Function/Method definition (ese name(params) { ... })"""
    name: str = ""
    params: List[str] = field(default_factory=list)
    body: List[ASTNode] = field(default_factory=list)


@dataclass
class IfStmt(ASTNode):
    """If statement (ti condition { ... } bibẹkọ { ... })"""
    condition: Any = None
    then_body: List[ASTNode] = field(default_factory=list)
    else_body: List[ASTNode] = field(default_factory=list)


@dataclass
class WhileStmt(ASTNode):
    """While loop (nigba condition { ... })"""
    condition: Any = None
    body: List[ASTNode] = field(default_factory=list)


@dataclass
class ReturnStmt(ASTNode):
    """Return statement (pada value;)"""
    value: Any = None


@dataclass
class EndStmt(ASTNode):
    """End statement (ase;)"""
    pass


@dataclass
class BinaryOp(ASTNode):
    """Binary operation (left op right)"""
    op: str = ""
    left: Any = None
    right: Any = None


@dataclass
class UnaryOp(ASTNode):
    """Unary operation (op operand)"""
    op: str = ""
    operand: Any = None


@dataclass
class Literal(ASTNode):
    """Literal value (number, string, boolean)"""
    value: Any = None
    type: str = ""  # "number", "string", "boolean"


@dataclass
class Identifier(ASTNode):
    """Variable reference"""
    name: str = ""


@dataclass
class ForStmt(ASTNode):
    """For loop (fun i ninu items { ... })"""
    var_name: str = ""
    iterable: Any = None
    body: List[ASTNode] = field(default_factory=list)


@dataclass
class TryStmt(ASTNode):
    """Try/Catch (dida_ewu { ... } kaka_ewu (err) { ... })"""
    try_body: List[ASTNode] = field(default_factory=list)
    error_var: str = ""
    catch_body: List[ASTNode] = field(default_factory=list)


@dataclass
class ListLiteral(ASTNode):
    """List/Array literal [1, 2, 3]"""
    elements: List[Any] = field(default_factory=list)


@dataclass
class MapLiteral(ASTNode):
    """HashMap/Dict literal { "key": value }"""
    entries: Dict[str, Any] = field(default_factory=dict)


@dataclass
class IndexAccess(ASTNode):
    """Index access (arr[0] or map["key"])"""
    target: str = ""
    index: Any = None


@dataclass
class SliceAccess(ASTNode):
    """Slice access (arr[0:5] or arr[::2] or arr[1:10:2])
    
    Supports Python-style slicing with negative indices.
    """
    target: str = ""
    start: Any = None   # None = from beginning
    end: Any = None     # None = to end
    step: Any = None    # None = step of 1


@dataclass
class MethodCall(ASTNode):
    """Method call on an object (obj.method(args))
    
    Used for calling methods on object instances created via odù.dá().
    """
    target: str = ""                           # The variable/object name
    method: str = ""                           # Method name
    args: List[Any] = field(default_factory=list)


@dataclass
class AssignmentStmt(ASTNode):
    """Assignment (x = 5; or arr[0] = 5;)"""
    target: Any = None  # Identifier or IndexAccess
    value: Any = None


@dataclass
class TabooStmt(ASTNode):
    """Èèwọ̀ (Taboo) - Architectural constraint declaration.
    
    Examples:
        èèwọ̀: Ọ̀ṣẹ́(UI) -> Òdí(DB);   # UI can't call DB
        èèwọ̀: Òtúrá.*;               # No network in this file
    """
    source_domain: str = ""          # e.g. "ose" or "otura"
    source_context: str = ""         # e.g. "UI" (optional)
    target_domain: str = ""          # e.g. "odi" (empty for wildcard block)
    target_context: str = ""         # e.g. "DB" (optional)
    is_wildcard: bool = False        # True if blocking all from source


@dataclass
class AseBlock(ASTNode):
    """Àṣẹ (Authority) - Critical/Atomic execution block.
    
    Features:
        - Atomic execution (transaction-like)
        - Auto-rollback on failure
        - No interrupts on embedded systems
    """
    body: List[ASTNode] = field(default_factory=list)
    label: str = ""  # Optional label for the critical section


# =============================================================================
# AST TRANSFORMER
# =============================================================================

if LARK_AVAILABLE:
    @v_args(inline=True)
    class IfaTransformer(Transformer):
        """
        Transforms Lark parse tree into Ifá AST nodes.
        """
        
        def start(self, *statements):
            return Program(statements=list(statements))
        
        # === Statements ===
        
        def statement(self, stmt):
            return stmt
        
        def import_stmt(self, path):
            return ImportStmt(path=path)
        
        def module_path(self, *names):
            return [str(n) for n in names]
        
        def var_decl(self, name, *rest):
            """Parse variable declaration with optional type hint.
            
            Cases:
                ayanmo x = 50;           → VarDecl(name='x', value=50, type_hint=None)
                ayanmo x: Int = 50;      → VarDecl(name='x', value=50, type_hint='Int')
            """
            type_hint = None
            value = None
            
            for item in rest:
                if isinstance(item, str) and item in ('Int', 'Float', 'Str', 'Bool', 'List', 'Map', 'Any',
                                                       'Nọmbà', 'Number', 'Ìdá', 'Ida', 'Ọ̀rọ̀', 'Oro', 'String',
                                                       'Òtítọ́', 'Otito', 'Àkójọ', 'Akojo', 'Array',
                                                       'Àwòrán', 'Aworan', 'Dict', 'Àìyẹ', 'Dynamic'):
                    type_hint = item
                else:
                    value = item
            
            # Normalize type hints to canonical form
            type_map = {
                'Nọmbà': 'Int', 'Number': 'Int',
                'Ìdá': 'Float', 'Ida': 'Float',
                'Ọ̀rọ̀': 'Str', 'Oro': 'Str', 'String': 'Str',
                'Òtítọ́': 'Bool', 'Otito': 'Bool',
                'Àkójọ': 'List', 'Akojo': 'List', 'Array': 'List',
                'Àwòrán': 'Map', 'Aworan': 'Map', 'Dict': 'Map',
                'Àìyẹ': 'Any', 'Dynamic': 'Any',
            }
            if type_hint in type_map:
                type_hint = type_map[type_hint]
            
            return VarDecl(name=str(name), value=value, type_hint=type_hint)
        
        def type_hint(self, type_name):
            """Extract type name from type hint."""
            return str(type_name)
        
        def type_name(self, name):
            """Return the type name token."""
            return str(name)
        
        def instruction(self, call):
            return Instruction(call=call)
        
        def odu_call(self, odu, ese, *args):
            arg_list = list(args[0]) if args else []
            return OduCall(odu=str(odu), ese=str(ese), args=arg_list)
        
        def odu_name(self, name):
            return str(name)
        
        def ese_name(self, name):
            return str(name)
        
        def arguments(self, *args):
            return list(args)
        
        # === Odù Definitions ===
        
        def odu_def(self, name, body):
            methods = [m for m in body if isinstance(m, EseDef)]
            variables = [v for v in body if isinstance(v, VarDecl)]
            return OduDef(name=str(name), methods=methods, variables=variables)
        
        def odu_body(self, *items):
            return [i for i in items if i is not None]
        
        def ese_def(self, name, *rest):
            params = []
            body = []
            for item in rest:
                if isinstance(item, list) and all(isinstance(p, str) for p in item):
                    params = item
                elif isinstance(item, list):
                    body = item
            return EseDef(name=str(name), params=params, body=body)
        
        def params(self, *params):
            return [str(p) for p in params]
        
        def param(self, name, *type_hint):
            return str(name)
        
        # === Control Flow ===
        
        def if_stmt(self, condition, *rest):
            then_body = list(rest[0]) if rest else []
            else_body = list(rest[1]) if len(rest) > 1 else []
            return IfStmt(condition=condition, then_body=then_body, else_body=else_body)
        
        def else_clause(self, *stmts):
            return list(stmts)
        
        def while_stmt(self, condition, *body):
            return WhileStmt(condition=condition, body=list(body))
        
        def return_stmt(self, *value):
            return ReturnStmt(value=value[0] if value else None)
        
        def end_stmt(self):
            return EndStmt()
        
        # === For Loop ===
        
        def for_stmt(self, var_name, iterable, *body):
            return ForStmt(var_name=str(var_name), iterable=iterable, body=list(body))
        
        # === Try/Catch ===
        
        def try_stmt(self, *items):
            try_body = []
            error_var = "err"
            catch_body = []
            
            # Parse the items
            for i, item in enumerate(items):
                if isinstance(item, str):
                    error_var = item
                elif isinstance(item, list):
                    if not try_body:
                        try_body = item
                    else:
                        catch_body = item
                elif hasattr(item, 'name'):
                    error_var = item.name
            
            return TryStmt(try_body=try_body, error_var=error_var, catch_body=catch_body)
        
        # === Data Structures ===
        
        def list_literal(self, *items):
            return ListLiteral(elements=list(items))
        
        def map_literal(self, *entries):
            result = {}
            for entry in entries:
                if isinstance(entry, tuple) and len(entry) == 2:
                    result[entry[0]] = entry[1]
            return MapLiteral(entries=result)
        
        def map_entry(self, key, value):
            key_str = str(key)
            if key_str.startswith('"') and key_str.endswith('"'):
                key_str = key_str[1:-1]
            return (key_str, value)
        
        def index_access(self, target, index):
            if hasattr(target, 'name'):
                target = target.name
            return IndexAccess(target=str(target), index=index)
        
        def slice_access(self, target, *args):
            """Parse slice syntax: arr[start:end] or arr[::step] or arr[start:end:step]
            
            Grammar: NAME "[" expression? ":" expression? (":" expression?)? "]"
            """
            if hasattr(target, 'name'):
                target = target.name
            
            # Extract start, end, step from args
            # args contains only the expressions that were provided (not empty slots)
            start = None
            end = None
            step = None
            
            # Parse based on number of arguments
            if len(args) == 0:
                # [:] - all elements
                pass
            elif len(args) == 1:
                # [start:] or [:end]
                start = args[0]
            elif len(args) == 2:
                # [start:end] or [start::step] or [::step] depending on positions
                start = args[0]
                end = args[1]
            elif len(args) >= 3:
                # [start:end:step]
                start = args[0]
                end = args[1]
                step = args[2] if len(args) > 2 else None
            
            return SliceAccess(target=str(target), start=start, end=end, step=step)
        
        def method_call(self, target, method, *args):
            """Parse method call: obj.method(args)"""
            if hasattr(target, 'name'):
                target = target.name
            arg_list = list(args[0]) if args else []
            return MethodCall(target=str(target), method=str(method), args=arg_list)
        
        # === Assignment ===
        
        def assignment_stmt(self, target, value):
            return AssignmentStmt(target=target, value=value)
        
        # === Èèwọ̀ (Taboo) - Architectural Constraints ===
        
        def taboo_stmt(self, rule):
            return rule  # Already a TabooStmt from taboo_rule
        
        def taboo_rule(self, *items):
            # Parse the taboo rule variants
            if len(items) >= 2:
                source = items[0]
                # Check if wildcard (source.*)
                if len(items) == 2 and items[1] == '*':
                    return TabooStmt(
                        source_domain=source[0] if isinstance(source, tuple) else str(source),
                        source_context=source[1] if isinstance(source, tuple) and len(source) > 1 else "",
                        is_wildcard=True
                    )
                # Specific dependency ban (source -> target)
                target = items[1]
                return TabooStmt(
                    source_domain=source[0] if isinstance(source, tuple) else str(source),
                    source_context=source[1] if isinstance(source, tuple) and len(source) > 1 else "",
                    target_domain=target[0] if isinstance(target, tuple) else str(target),
                    target_context=target[1] if isinstance(target, tuple) and len(target) > 1 else "",
                    is_wildcard=False
                )
            return TabooStmt()
        
        def taboo_source(self, domain, *context):
            domain_str = str(domain)
            ctx = str(context[0]) if context else ""
            return (domain_str, ctx)
        
        def taboo_target(self, domain, *context):
            domain_str = str(domain)
            ctx = str(context[0]) if context else ""
            return (domain_str, ctx)
        
        # === Àṣẹ (Authority) - Critical Block ===
        
        def ase_block(self, *statements):
            return AseBlock(body=list(statements))
        
        # === Expressions ===
        
        def expression(self, expr):
            return expr
        
        def or_expr(self, *args):
            if len(args) == 1:
                return args[0]
            result = args[0]
            for i in range(1, len(args)):
                result = BinaryOp(op="||", left=result, right=args[i])
            return result
        
        def and_expr(self, *args):
            if len(args) == 1:
                return args[0]
            result = args[0]
            for i in range(1, len(args)):
                result = BinaryOp(op="&&", left=result, right=args[i])
            return result
        
        def not_expr(self, *args):
            if len(args) == 1:
                return args[0]
            return UnaryOp(op="!", operand=args[0])
        
        def comparison(self, left, *rest):
            if not rest:
                return left
            op, right = rest
            return BinaryOp(op=str(op), left=left, right=right)
        
        def arith_expr(self, *args):
            if len(args) == 1:
                return args[0]
            result = args[0]
            i = 1
            while i < len(args):
                op = str(args[i])
                right = args[i + 1]
                result = BinaryOp(op=op, left=result, right=right)
                i += 2
            return result
        
        def term(self, *args):
            if len(args) == 1:
                return args[0]
            result = args[0]
            i = 1
            while i < len(args):
                op = str(args[i])
                right = args[i + 1]
                result = BinaryOp(op=op, left=result, right=right)
                i += 2
            return result
        
        def factor(self, *args):
            if len(args) == 1:
                return args[0]
            return UnaryOp(op=str(args[0]), operand=args[1])
        
        def atom(self, value):
            return value
        
        # === Literals ===
        
        def NUMBER(self, token):
            return Literal(value=int(token), type="number")
        
        def FLOAT(self, token):
            return Literal(value=float(token), type="float")
        
        def STRING(self, token):
            # Remove quotes
            s = str(token)
            if s.startswith('"') and s.endswith('"'):
                s = s[1:-1]
            elif s.startswith("'") and s.endswith("'"):
                s = s[1:-1]
            return Literal(value=s, type="string")
        
        def BOOLEAN(self, token):
            val = str(token).lower()
            return Literal(value=val in ("true", "otito"), type="boolean")
        
        def NAME(self, token):
            return Identifier(name=str(token))
        
        # === Odù Names (normalize to ASCII) ===
        
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


# =============================================================================
# PARSER CLASS
# =============================================================================

class IfaLarkParser:
    """
    Formal Ifá-Lang parser using Lark EBNF grammar.
    """
    
    def __init__(self, grammar_path: str = None):
        if not LARK_AVAILABLE:
            raise ImportError("Lark is required. Install with: pip install lark")
        
        # Find grammar file
        if grammar_path is None:
            grammar_path = os.path.join(
                os.path.dirname(__file__),
                "grammar.lark"
            )
        
        if not os.path.exists(grammar_path):
            raise FileNotFoundError(f"Grammar file not found: {grammar_path}")
        
        # Load grammar
        with open(grammar_path, 'r', encoding='utf-8') as f:
            grammar = f.read()
        
        # Create parser
        self.parser = Lark(
            grammar,
            start='start',
            parser='lalr',
            transformer=IfaTransformer()
        )
    
    def parse(self, source: str) -> Program:
        """
        Parse Ifá source code into AST.
        
        Args:
            source: Ifá source code string
            
        Returns:
            Program AST node
        """
        return self.parser.parse(source)
    
    def parse_file(self, filepath: str) -> Program:
        """Parse a .ifa file into AST."""
        with open(filepath, 'r', encoding='utf-8') as f:
            source = f.read()
        return self.parse(source)


# =============================================================================
# AST UTILITIES
# =============================================================================

def ast_to_dict(node: ASTNode) -> Dict:
    """Convert AST node to dictionary (for debugging/serialization)."""
    if isinstance(node, (int, float, str, bool, type(None))):
        return node
    if isinstance(node, list):
        return [ast_to_dict(item) for item in node]
    if isinstance(node, ASTNode):
        result = {"_type": type(node).__name__}
        for key, value in node.__dict__.items():
            result[key] = ast_to_dict(value)
        return result
    return str(node)


def print_ast(node: ASTNode, indent: int = 0):
    """Pretty print an AST node."""
    prefix = "  " * indent
    if isinstance(node, Program):
        print(f"{prefix}Program:")
        for stmt in node.statements:
            print_ast(stmt, indent + 1)
    elif isinstance(node, ImportStmt):
        print(f"{prefix}Import: {'.'.join(node.path)}")
    elif isinstance(node, VarDecl):
        print(f"{prefix}VarDecl: {node.name} = {node.value}")
    elif isinstance(node, Instruction):
        print(f"{prefix}Instruction:")
        print_ast(node.call, indent + 1)
    elif isinstance(node, OduCall):
        args_str = ", ".join(str(a) for a in node.args)
        print(f"{prefix}Call: {node.odu}.{node.ese}({args_str})")
    elif isinstance(node, OduDef):
        print(f"{prefix}OduDef: {node.name}")
        for method in node.methods:
            print_ast(method, indent + 1)
    elif isinstance(node, EseDef):
        print(f"{prefix}EseDef: {node.name}({', '.join(node.params)})")
        for stmt in node.body:
            print_ast(stmt, indent + 1)
    elif isinstance(node, IfStmt):
        print(f"{prefix}If:")
        print_ast(node.condition, indent + 1)
        print(f"{prefix}  Then:")
        for stmt in node.then_body:
            print_ast(stmt, indent + 2)
        if node.else_body:
            print(f"{prefix}  Else:")
            for stmt in node.else_body:
                print_ast(stmt, indent + 2)
    elif isinstance(node, WhileStmt):
        print(f"{prefix}While:")
        print_ast(node.condition, indent + 1)
        for stmt in node.body:
            print_ast(stmt, indent + 1)
    elif isinstance(node, ReturnStmt):
        print(f"{prefix}Return: {node.value}")
    elif isinstance(node, EndStmt):
        print(f"{prefix}End")
    elif isinstance(node, BinaryOp):
        print(f"{prefix}BinaryOp: {node.op}")
        print_ast(node.left, indent + 1)
        print_ast(node.right, indent + 1)
    elif isinstance(node, Literal):
        print(f"{prefix}Literal: {node.value} ({node.type})")
    elif isinstance(node, Identifier):
        print(f"{prefix}Identifier: {node.name}")
    else:
        print(f"{prefix}{node}")


# =============================================================================
# DEMO
# =============================================================================

if __name__ == "__main__":
    if not LARK_AVAILABLE:
        print("Please install Lark: pip install lark")
        sys.exit(1)
    
    print("""
╔══════════════════════════════════════════════════════════════╗
║              IFÁ-LANG FORMAL PARSER DEMO                     ║
╠══════════════════════════════════════════════════════════════╣
║  Using Lark EBNF Grammar                                     ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    # Test code
    test_code = '''
iba std.otura;

ayanmo x = 50;

Irosu.fo("Hello World");
Ogbe.bi(100);
Obara.ro(x);

ase;
'''
    
    try:
        parser = IfaLarkParser()
        ast = parser.parse(test_code)
        
        print("=== Parsed AST ===")
        print_ast(ast)
        
        print("\n=== AST as Dict ===")
        import json
        print(json.dumps(ast_to_dict(ast), indent=2, default=str))
        
    except Exception as e:
        print(f"Parse error: {e}")
        import traceback
        traceback.print_exc()
