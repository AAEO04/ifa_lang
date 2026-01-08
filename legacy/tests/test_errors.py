# -*- coding: utf-8 -*-
"""Unit tests for errors.py - Babalawo error system and debugging tools."""

import sys
import os
import unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.errors import (
    IfaError, Babalawo, ODU_WISDOM,
    StateHistoryBuffer, HarmonyHeatmap,
    speak, warn
)


class TestOduWisdom(unittest.TestCase):
    """Test Odù wisdom definitions."""
    
    def test_16_odu_defined(self):
        """All 16 principal Odù should have wisdom."""
        expected = ["OGBE", "OYEKU", "IWORI", "ODI", "IROSU", "OWONRIN",
                    "OBARA", "OKANRAN", "OGUNDA", "OSA", "IKA", "OTURUPON",
                    "OTURA", "IRETE", "OSE", "OFUN"]
        for odu in expected:
            self.assertIn(odu, ODU_WISDOM, f"{odu} missing from ODU_WISDOM")
    
    def test_wisdom_has_proverbs(self):
        """Each Odù should have proverbs."""
        for odu, data in ODU_WISDOM.items():
            self.assertIn("proverbs", data, f"{odu} missing proverbs")
            self.assertGreater(len(data["proverbs"]), 0)
    
    def test_wisdom_has_advice(self):
        """Each Odù should have advice."""
        for odu, data in ODU_WISDOM.items():
            self.assertIn("advice", data, f"{odu} missing advice")


class TestIfaError(unittest.TestCase):
    """Test error dataclass."""
    
    def test_create_error(self):
        """Should create error with required fields."""
        error = IfaError(code="TEST_ERROR", odu="OGBE", line=10)
        self.assertEqual(error.code, "TEST_ERROR")
        self.assertEqual(error.odu, "OGBE")
        self.assertEqual(error.line, 10)
    
    def test_optional_fields(self):
        """Optional fields should have defaults."""
        error = IfaError(code="TEST", odu="OYEKU", line=1)
        self.assertEqual(error.column, 0)
        self.assertEqual(error.message, "")


class TestBabalawo(unittest.TestCase):
    """Test the Babalawo error diagnosis system."""
    
    def setUp(self):
        self.babalawo = Babalawo()
    
    def test_diagnose_creates_message(self):
        """Diagnose should return a non-empty message."""
        error = IfaError(code="NULL_REFERENCE", odu="OYEKU", line=5)
        message = self.babalawo.diagnose(error)
        self.assertIsNotNone(message)
        self.assertGreater(len(message), 0)
    
    def test_quick_diagnose(self):
        """Quick diagnose should return string."""
        result = self.babalawo.quick_diagnose("DIVISION_BY_ZERO", 10)
        self.assertIsInstance(result, str)
    
    def test_error_to_odu_mapping(self):
        """Should map error codes to Odù domains."""
        self.assertIn("NULL_REFERENCE", self.babalawo.ERROR_TO_ODU)
        self.assertIn("DIVISION_BY_ZERO", self.babalawo.ERROR_TO_ODU)


class TestStateHistoryBuffer(unittest.TestCase):
    """Test time-travel debugging buffer."""
    
    def setUp(self):
        self.buffer = StateHistoryBuffer()
    
    def test_push_and_length(self):
        """Should track pushed states."""
        self.buffer.push({"vars": {"x": 1}})
        self.assertEqual(len(self.buffer), 1)
    
    def test_circular_buffer(self):
        """Should wrap at size limit (32)."""
        for i in range(50):
            self.buffer.push({"step": i})
        self.assertEqual(len(self.buffer), 32)
    
    def test_rewind(self):
        """Should retrieve previous states."""
        self.buffer.push({"step": 1})
        self.buffer.push({"step": 2})
        self.buffer.push({"step": 3})
        
        state = self.buffer.rewind(0)  # Most recent
        self.assertEqual(state["step"], 3)
        
        state = self.buffer.rewind(1)
        self.assertEqual(state["step"], 2)
    
    def test_custom_size(self):
        """Should respect custom buffer size."""
        small_buffer = StateHistoryBuffer(size=5)
        for i in range(10):
            small_buffer.push({"step": i})
        self.assertEqual(len(small_buffer), 5)


class TestHarmonyHeatmap(unittest.TestCase):
    """Test 16x16 Odù interaction matrix."""
    
    def setUp(self):
        self.heatmap = HarmonyHeatmap()
    
    def test_record_interaction(self):
        """Should record domain interactions."""
        self.heatmap.record_interaction("OSA", "IROSU", 0.2)
        harmony = self.heatmap.get_harmony("OSA", "IROSU")
        self.assertEqual(harmony, 0.2)
    
    def test_default_harmony(self):
        """Unrecorded interactions should return 1.0 (full harmony)."""
        harmony = self.heatmap.get_harmony("OGBE", "OYEKU")
        self.assertEqual(harmony, 1.0)
    
    def test_find_discord(self):
        """Should find low-harmony interactions."""
        self.heatmap.record_interaction("A", "B", 0.3)
        self.heatmap.record_interaction("C", "D", 0.8)
        
        discord = self.heatmap.find_discord(threshold=0.5)
        self.assertEqual(len(discord), 1)
        self.assertEqual(discord[0][1], 0.3)
    
    def test_case_insensitive(self):
        """Domain names should be case-insensitive."""
        self.heatmap.record_interaction("osa", "irosu", 0.5)
        harmony = self.heatmap.get_harmony("OSA", "IROSU")
        self.assertEqual(harmony, 0.5)


class TestConvenienceFunctions(unittest.TestCase):
    """Test module-level convenience functions."""
    
    def test_speak_returns_string(self):
        """speak() should return diagnostic string."""
        result = speak("UNKNOWN_ERROR", 1)
        self.assertIsInstance(result, str)


if __name__ == "__main__":
    print("=" * 60)
    print("  ERRORS / BABALAWO UNIT TESTS")
    print("=" * 60)
    unittest.main(verbosity=2)
