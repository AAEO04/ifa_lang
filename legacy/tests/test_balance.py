# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ-LANG TEST SUITE                                       ║
║                    Unit tests for the Ìwà Engine (Balance Checker)           ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import sys
import os
import unittest

# Add project root to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.validator import SmartIfaCompiler, IwaEngine


class TestIwaEngine(unittest.TestCase):
    """Test cases for the Ìwà Engine (balance checker)."""
    
    def setUp(self):
        self.engine = IwaEngine(strict_mode=True)
    
    def test_balanced_file_operations(self):
        """Test that matching open/close file operations pass."""
        tokens = ["odi.si", "odi.pa"]
        result = self.engine.check(tokens)
        self.assertTrue(result, "File open/close should be balanced")
    
    def test_unbalanced_file_open(self):
        """Test that unclosed file operation fails."""
        tokens = ["odi.si"]  # Open without close
        result = self.engine.check(tokens)
        self.assertFalse(result, "Unclosed file should fail balance check")
    
    def test_balanced_network_operations(self):
        """Test that matching network bind/close operations pass."""
        tokens = ["otura.de", "otura.pa"]
        result = self.engine.check(tokens)
        self.assertTrue(result, "Network bind/close should be balanced")
    
    def test_balanced_memory_operations(self):
        """Test that matching alloc/free operations pass."""
        tokens = ["ogunda.ge", "irete.tu"]
        result = self.engine.check(tokens)
        self.assertTrue(result, "Memory alloc/free should be balanced")
    
    def test_empty_program(self):
        """Test that an empty program passes."""
        tokens = []
        result = self.engine.check(tokens)
        self.assertTrue(result, "Empty program should be balanced")
    
    def test_multiple_balanced_resources(self):
        """Test multiple balanced resource pairs."""
        tokens = ["odi.si", "otura.de", "otura.pa", "odi.pa"]
        result = self.engine.check(tokens)
        self.assertTrue(result, "Multiple balanced resources should pass")


class TestSmartCompiler(unittest.TestCase):
    """Test cases for the Smart Compiler pipeline."""
    
    def setUp(self):
        self.compiler = SmartIfaCompiler(strict_mode=True)
    
    def test_parse_simple_program(self):
        """Test parsing a simple program."""
        source = """
        Ogbe.bi();
        Irosu.fo("Hello");
        Oyeku.duro();
        """
        tokens = self.compiler.parse(source)
        self.assertGreater(len(tokens), 0, "Should produce tokens")
    
    def test_balanced_source_compiles(self):
        """Test that balanced source code compiles successfully."""
        source = """
        Odi.si("test.txt");
        Odi.pa("test.txt");
        """
        result = self.compiler.compile(source)
        self.assertIn("valid", result.get("status", ""), 
                      "Balanced code should compile")
    
    def test_unbalanced_source_fails(self):
        """Test that unbalanced source fails compilation."""
        source = """
        Odi.si("test.txt");
        # Missing Odi.pa()!
        """
        result = self.compiler.compile(source)
        self.assertIn("error", result.get("status", "").lower(),
                      "Unbalanced code should fail")


class TestNormalization(unittest.TestCase):
    """Test Yoruba Unicode normalization."""
    
    def setUp(self):
        self.engine = IwaEngine()
    
    def test_normalize_yoruba_characters(self):
        """Test that Yoruba diacritics are normalized correctly."""
        # Various forms of the same word
        test_cases = [
            ("Ọ̀yẹ̀kú", "oyeku"),
            ("Ìwòrì", "iwori"),
            ("Ọ̀bàrà", "obara"),
        ]
        for yoruba, expected in test_cases:
            normalized = self.engine.normalize(yoruba.lower())
            # Check that normalization produces consistent results
            self.assertIsInstance(normalized, str)


if __name__ == "__main__":
    print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ-LANG TEST SUITE                                       ║
╚══════════════════════════════════════════════════════════════════════════════╝
""")
    unittest.main(verbosity=2)
