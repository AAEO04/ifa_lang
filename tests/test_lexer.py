# -*- coding: utf-8 -*-
"""Unit tests for lexer.py - Dual lexicon and keyword system."""

import sys
import os
import unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.lexer import RESERVED_KEYWORDS, SOFT_KEYWORDS, KEYWORDS, KEYWORD_ALIASES


class TestReservedKeywords(unittest.TestCase):
    """Test strictly reserved keywords."""
    
    def test_core_control_flow_reserved(self):
        """Core control flow keywords must be reserved."""
        for kw in ["if", "else", "for", "while", "return", "break", "continue"]:
            self.assertIn(kw, RESERVED_KEYWORDS, f"'{kw}' should be reserved")
    
    def test_yoruba_control_flow_reserved(self):
        """Yoruba control flow keywords must be reserved."""
        for kw in ["ti", "bí", "bibẹkọ", "fun", "nigba", "pada"]:
            self.assertIn(kw, RESERVED_KEYWORDS, f"'{kw}' should be reserved")
    
    def test_declarations_reserved(self):
        """Declaration keywords must be reserved."""
        for kw in ["let", "var", "class", "function", "fn"]:
            self.assertIn(kw, RESERVED_KEYWORDS, f"'{kw}' should be reserved")
    
    def test_boolean_reserved(self):
        """Boolean literals must be reserved."""
        for kw in ["true", "false", "nil", "null"]:
            self.assertIn(kw, RESERVED_KEYWORDS, f"'{kw}' should be reserved")
    
    def test_cen_core_reserved(self):
        """CEN model core keywords must be reserved."""
        for kw in ["ebo", "ẹbọ", "sacrifice", "difa", "dífá", "opon", "ọpọ́n"]:
            self.assertIn(kw, RESERVED_KEYWORDS, f"'{kw}' should be reserved")


class TestSoftKeywords(unittest.TestCase):
    """Test context-sensitive (soft) keywords."""
    
    def test_common_words_soft(self):
        """Common English words should be soft (usable as identifiers)."""
        for kw in ["parent", "child", "small", "large", "type", "string"]:
            self.assertIn(kw, SOFT_KEYWORDS, f"'{kw}' should be soft")
    
    def test_size_keywords_soft(self):
        """Size keywords should be soft."""
        # 'default' is excluded - it's reserved for match/case/default
        for kw in ["small", "mini", "standard", "large", "big", "mega", "huge"]:
            self.assertIn(kw, SOFT_KEYWORDS, f"'{kw}' should be soft")
    
    def test_type_keywords_soft(self):
        """Type keywords should be soft."""
        for kw in ["int", "float", "string", "array", "list", "dict"]:
            self.assertIn(kw, SOFT_KEYWORDS, f"'{kw}' should be soft")


class TestKeywordsCombined(unittest.TestCase):
    """Test the combined KEYWORDS set."""
    
    def test_keywords_is_union(self):
        """KEYWORDS should be union of reserved and soft."""
        self.assertEqual(KEYWORDS, RESERVED_KEYWORDS | SOFT_KEYWORDS)
    
    def test_no_overlap(self):
        """Reserved and soft should not overlap."""
        overlap = RESERVED_KEYWORDS & SOFT_KEYWORDS
        self.assertEqual(len(overlap), 0, f"Overlap found: {overlap}")
    
    def test_minimum_count(self):
        """Should have at least 100 keywords total."""
        self.assertGreaterEqual(len(KEYWORDS), 100)


class TestKeywordAliases(unittest.TestCase):
    """Test English to Yoruba aliases."""
    
    def test_import_alias(self):
        """'import' should map to 'iba'."""
        self.assertEqual(KEYWORD_ALIASES.get("import"), "iba")
    
    def test_if_alias(self):
        """'if' should map to 'ti'."""
        self.assertEqual(KEYWORD_ALIASES.get("if"), "ti")
    
    def test_class_alias(self):
        """'class' should map to 'odu'."""
        self.assertEqual(KEYWORD_ALIASES.get("class"), "odu")
    
    def test_function_alias(self):
        """'function' should map to 'ese'."""
        self.assertEqual(KEYWORD_ALIASES.get("function"), "ese")
    
    def test_sacrifice_alias(self):
        """'sacrifice' should map to 'ebo'."""
        self.assertEqual(KEYWORD_ALIASES.get("sacrifice"), "ebo")


class TestDualLexiconCompleteness(unittest.TestCase):
    """Test that both languages are fully supported."""
    
    def test_control_flow_bilingual(self):
        """Both Yoruba and English control flow should exist."""
        yoruba = ["ti", "bibẹkọ", "fun", "nigba"]
        english = ["if", "else", "for", "while"]
        for y, e in zip(yoruba, english):
            self.assertIn(y, KEYWORDS, f"Yoruba '{y}' missing")
            self.assertIn(e, KEYWORDS, f"English '{e}' missing")
    
    def test_odu_import_not_confused_with_if(self):
        """ìbà should be import, NOT if."""
        self.assertEqual(KEYWORD_ALIASES.get("import"), "iba")
        # ti should be if
        self.assertEqual(KEYWORD_ALIASES.get("if"), "ti")


if __name__ == "__main__":
    print("=" * 60)
    print("  LEXER / KEYWORD UNIT TESTS")
    print("=" * 60)
    unittest.main(verbosity=2)
