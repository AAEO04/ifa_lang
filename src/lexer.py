# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ-LANG LEXER - THE TOKENIZER                            ║
║                    Transforms source code into tokens                        ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import re
import unicodedata
from typing import List, Tuple, Optional
from enum import Enum, auto


class TokenType(Enum):
    """Token types for the Ifá-Lang lexer."""
    # Keywords
    KEYWORD = auto()          # ayanmo, ase, iba, etc.
    DOMAIN = auto()           # Ogbe, Otura, Irosu, etc.
    ESE = auto()              # Method names like fikun, din, so
    
    # Literals
    NUMBER = auto()           # 42, 3.14
    STRING = auto()           # "hello"
    IDENTIFIER = auto()       # variable names
    
    # Operators
    OPERATOR = auto()         # +, -, *, /
    ASSIGN = auto()           # =
    DOT = auto()              # .
    COMMA = auto()            # ,
    SEMICOLON = auto()        # ;
    
    # Brackets
    LPAREN = auto()           # (
    RPAREN = auto()           # )
    LBRACE = auto()           # {
    RBRACE = auto()           # }
    LBRACKET = auto()         # [
    RBRACKET = auto()         # ]
    
    # Special
    COMMENT = auto()          # # comment
    NEWLINE = auto()
    EOF = auto()
    ERROR = auto()


# The 16 Odù Domain Names (with normalized ASCII versions)
ODU_DOMAINS = {
    "ogbe": "OGBE", "ogbè": "OGBE",
    "oyeku": "OYEKU", "ọ̀yẹ̀kú": "OYEKU", "oyẹku": "OYEKU",
    "iwori": "IWORI", "ìwòrì": "IWORI",
    "odi": "ODI", "òdí": "ODI",
    "irosu": "IROSU", "ìrosù": "IROSU",
    "owonrin": "OWONRIN", "ọ̀wọ́nrín": "OWONRIN",
    "obara": "OBARA", "ọ̀bàrà": "OBARA",
    "okanran": "OKANRAN", "ọ̀kànràn": "OKANRAN",
    "ogunda": "OGUNDA", "ògúndá": "OGUNDA",
    "osa": "OSA", "ọ̀sá": "OSA",
    "ika": "IKA", "ìká": "IKA",
    "oturupon": "OTURUPON", "òtúúrúpọ̀n": "OTURUPON",
    "otura": "OTURA", "òtúrá": "OTURA",
    "irete": "IRETE", "ìrẹtẹ̀": "IRETE",
    "ose": "OSE", "ọ̀ṣẹ́": "OSE",
    "ofun": "OFUN", "òfún": "OFUN",
}

# =============================================================================
# DUAL LEXICON - Context-Sensitive Keywords
# =============================================================================

# Reserved Keywords - These cannot be used as identifiers
RESERVED_KEYWORDS = {
    # === CONTROL FLOW (English) ===
    "if", "else", "elif", "for", "while", "break", "continue", "return",
    "try", "catch", "throw", "match", "case", "default",
    
    # === CONTROL FLOW (Yoruba) ===
    "ti", "bí", "bibẹkọ", "bíbẹ́kọ́", "fun", "nigba", "pada", "da", "gbiyanju",
    "mu", "sọ", "yàn", "ìran",
    
    # === DECLARATIONS (English) ===
    "let", "var", "const", "class", "function", "fn", "def", "import", "export",
    "public", "private", "static", "async", "await",
    
    # === DECLARATIONS (Yoruba) ===
    "ayanmo", "àyànmọ́", "ọdù", "odu", "ẹsẹ", "ese", "ìbà", "iba", "gbangba",
    "àṣírí", "asiri",
    
    # === LITERALS ===
    "true", "false", "nil", "null", "otito", "iro", "ohunkohun",
    
    # === CEN MODEL CORE ===
    "ebo", "ẹbọ", "sacrifice", "ajose", "àjọṣe", "relationship",
    "difa", "dífá", "divination", "opon", "ọpọ́n", "board",
    "gpc", "ase", "àṣẹ", "taboo", "ewọ", "èèwọ̀",
    
    # === SEMANTIC DISPATCH ===
    "dispatch", "verb", "domain",
}

# Soft Keywords - Can be used as identifiers when not in keyword context
SOFT_KEYWORDS = {
    # === SIZE SPECIFIERS ===
    "small", "mini", "kekere", "kékeré",
    "standard", "gidi",  # Removed 'default' - it's reserved for match/case
    "large", "big", "nla", "nlá",
    "mega", "huge", "tobi",
    
    # === TYPES (can be identifiers) ===
    "int", "float", "string", "str", "bool", "array", "list", "dict", "map",
    "any", "void", "number", "object", "type",
    
    # === GPC HIERARCHY ===
    "parent", "child", "grandparent", "baba", "omo", "babanla",
    
    # === RELATIONSHIP PATTERNS ===
    "source", "target", "from", "to", "link", "unlink",
    
    # === ODUN DOMAINS ===
    "ogbe", "oyeku", "iwori", "odi", "irosu", "owonrin", "obara", "okanran",
    "ogunda", "osa", "ika", "oturupon", "otura", "irete", "ose", "ofun",
    
    # === OPERATION KEYWORDS ===
    "create", "delete", "update", "read", "send", "receive",
    "open", "close", "start", "stop", "begin", "end",
    
    # === MODIFIERS ===
    "readonly", "mutable", "optional", "required",
}

# Combined set for quick lookup
KEYWORDS = RESERVED_KEYWORDS | SOFT_KEYWORDS

# English to Yoruba keyword aliases
KEYWORD_ALIASES = {
    # Control flow
    "import": "iba",
    "if": "ti",
    "else": "bibẹkọ",
    "for": "fun",
    "while": "nigba",
    "return": "pada",
    "try": "gbiyanju",
    "catch": "mu",
    "throw": "sọ",
    "match": "yàn",
    
    # Declarations
    "class": "odu",
    "function": "ese",
    "let": "ayanmo",
    "public": "gbangba",
    "private": "asiri",
    
    # CEN Model
    "sacrifice": "ebo",
    "relationship": "ajose",
    "divination": "difa",
    "board": "opon",
    "taboo": "ewọ",
    
    # Sizes
    "small": "kekere",
    "large": "nla",
    "mega": "tobi",
    
    # GPC
    "parent": "baba",
    "child": "omo",
    "grandparent": "babanla",
}


class Token:
    """Represents a single token."""
    
    def __init__(self, type_: TokenType, value: str, line: int, column: int):
        self.type = type_
        self.value = value
        self.line = line
        self.column = column
    
    def __repr__(self):
        return f"Token({self.type.name}, {self.value!r}, L{self.line}:{self.column})"


class IfaLexer:
    """
    The Ifá-Lang Lexer - Tokenizes source code.
    Handles Yoruba diacritics and Unicode normalization.
    """
    
    def __init__(self):
        self.source = ""
        self.pos = 0
        self.line = 1
        self.column = 1
        self.tokens: List[Token] = []
    
    def normalize(self, text: str) -> str:
        """Normalize Unicode for consistent matching."""
        return unicodedata.normalize('NFC', text.lower())
    
    def tokenize(self, source: str) -> List[Token]:
        """Tokenize source code into a list of tokens."""
        self.source = source
        self.pos = 0
        self.line = 1
        self.column = 1
        self.tokens = []
        
        while self.pos < len(self.source):
            self._scan_token()
        
        self.tokens.append(Token(TokenType.EOF, "", self.line, self.column))
        return self.tokens
    
    def _current(self) -> str:
        """Get current character."""
        if self.pos >= len(self.source):
            return '\0'
        return self.source[self.pos]
    
    def _peek(self, offset: int = 1) -> str:
        """Peek ahead."""
        pos = self.pos + offset
        if pos >= len(self.source):
            return '\0'
        return self.source[pos]
    
    def _advance(self) -> str:
        """Advance position and return current char."""
        char = self._current()
        self.pos += 1
        if char == '\n':
            self.line += 1
            self.column = 1
        else:
            self.column += 1
        return char
    
    def _add_token(self, type_: TokenType, value: str):
        """Add a token to the list."""
        self.tokens.append(Token(type_, value, self.line, self.column))
    
    def _scan_token(self):
        """Scan a single token."""
        char = self._current()
        
        # Skip whitespace (except newlines)
        if char in ' \t\r':
            self._advance()
            return
        
        # Newline
        if char == '\n':
            self._advance()
            return
        
        # Comment
        if char == '#':
            self._scan_comment()
            return
        
        # String literal
        if char in '"\'':
            self._scan_string(char)
            return
        
        # Number
        if char.isdigit():
            self._scan_number()
            return
        
        # Identifier / Keyword / Domain
        if char.isalpha() or char == '_' or ord(char) > 127:
            self._scan_identifier()
            return
        
        # Single-character tokens
        single_tokens = {
            '(': TokenType.LPAREN, ')': TokenType.RPAREN,
            '{': TokenType.LBRACE, '}': TokenType.RBRACE,
            '[': TokenType.LBRACKET, ']': TokenType.RBRACKET,
            '.': TokenType.DOT, ',': TokenType.COMMA,
            ';': TokenType.SEMICOLON, '=': TokenType.ASSIGN,
            '+': TokenType.OPERATOR, '-': TokenType.OPERATOR,
            '*': TokenType.OPERATOR, '/': TokenType.OPERATOR,
            '%': TokenType.OPERATOR,
        }
        
        if char in single_tokens:
            self._add_token(single_tokens[char], char)
            self._advance()
            return
        
        # Unknown character
        self._add_token(TokenType.ERROR, char)
        self._advance()
    
    def _scan_comment(self):
        """Scan a comment until end of line."""
        start = self.pos
        while self._current() != '\n' and self._current() != '\0':
            self._advance()
        value = self.source[start:self.pos]
        self._add_token(TokenType.COMMENT, value)
    
    def _scan_string(self, quote: str):
        """Scan a string literal."""
        self._advance()  # Skip opening quote
        start = self.pos
        while self._current() != quote and self._current() != '\0':
            self._advance()
        value = self.source[start:self.pos]
        self._add_token(TokenType.STRING, value)
        if self._current() == quote:
            self._advance()  # Skip closing quote
    
    def _scan_number(self):
        """Scan a number literal."""
        start = self.pos
        while self._current().isdigit():
            self._advance()
        if self._current() == '.' and self._peek().isdigit():
            self._advance()  # Skip dot
            while self._current().isdigit():
                self._advance()
        value = self.source[start:self.pos]
        self._add_token(TokenType.NUMBER, value)
    
    def _scan_identifier(self):
        """Scan an identifier, keyword, or domain name."""
        start = self.pos
        while (self._current().isalnum() or 
               self._current() == '_' or 
               ord(self._current()) > 127):
            self._advance()
        
        value = self.source[start:self.pos]
        normalized = self.normalize(value)
        
        # Check if it's a domain name
        if normalized in ODU_DOMAINS:
            self._add_token(TokenType.DOMAIN, value)
        # Check if it's a keyword
        elif normalized in KEYWORDS:
            self._add_token(TokenType.KEYWORD, value)
        else:
            self._add_token(TokenType.IDENTIFIER, value)


# =============================================================================
# DEMO
# =============================================================================
if __name__ == "__main__":
    lexer = IfaLexer()
    
    source = """
# Ifá Demo
ayanmo x = 50;
Obara.fikun(10);
Ìrosù.fo("Hello Ifá!");
ase;
"""
    
    tokens = lexer.tokenize(source)
    for tok in tokens:
        if tok.type != TokenType.COMMENT:
            print(tok)
