# -*- coding: utf-8 -*-
"""Unit tests for ffi.py - SecureFFI and API layer."""

import sys
import os
import unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.ffi import SecureFFI, SecurityError, IfaFFI, IfaAPI


class TestSecureFFI(unittest.TestCase):
    """Test sandboxed Python imports."""
    
    def setUp(self):
        self.ffi = SecureFFI()
    
    def test_whitelisted_import(self):
        """Whitelisted modules should import successfully."""
        math = self.ffi.import_module("math")
        self.assertIsNotNone(math)
        self.assertEqual(math.sqrt(16), 4.0)
    
    def test_blocked_import(self):
        """Non-whitelisted modules should raise SecurityError."""
        with self.assertRaises(SecurityError):
            self.ffi.import_module("os")
    
    def test_subprocess_blocked(self):
        """Subprocess should be blocked."""
        with self.assertRaises(SecurityError):
            self.ffi.import_module("subprocess")
    
    def test_add_to_whitelist(self):
        """Should be able to add modules to whitelist."""
        self.ffi.add_to_whitelist("os")
        # Should not raise now
        os_mod = self.ffi.import_module("os")
        self.assertIsNotNone(os_mod)
    
    def test_default_whitelist(self):
        """Default whitelist should contain safe modules."""
        safe_modules = ["math", "json", "datetime", "collections"]
        for mod in safe_modules:
            self.assertIn(mod, self.ffi.whitelist)
    
    def test_import_with_alias(self):
        """Should support import aliases."""
        m = self.ffi.import_module("math", alias="m")
        self.assertIn("m", self.ffi.imported_modules)


class TestIfaFFI(unittest.TestCase):
    """Test the general FFI bridge."""
    
    def setUp(self):
        self.ffi = IfaFFI()
    
    def test_call_python_function(self):
        """Should call Python functions via FFI."""
        result = self.ffi.call_python("math", "sqrt", 16)
        self.assertEqual(result, 4.0)
    
    def test_call_python_with_multiple_args(self):
        """Should handle multiple arguments."""
        result = self.ffi.call_python("math", "pow", 2, 3)
        self.assertEqual(result, 8.0)


class TestIfaAPI(unittest.TestCase):
    """Test the API exposure layer."""
    
    def setUp(self):
        self.api = IfaAPI()
    
    def test_expose_function(self):
        """Should expose functions as endpoints."""
        self.api.expose("add", lambda x, y: x + y)
        self.assertIn("add", self.api.endpoints)
    
    def test_call_exposed_function(self):
        """Should call exposed functions."""
        self.api.expose("multiply", lambda x, y: x * y)
        result = self.api.call("multiply", 3, 4)
        self.assertEqual(result, 12)
    
    def test_call_unknown_raises(self):
        """Calling unknown endpoint should raise."""
        with self.assertRaises((KeyError, RuntimeError)):
            self.api.call("nonexistent")
    
    def test_to_json_schema(self):
        """Should generate JSON schema."""
        self.api.expose("test_fn", lambda: 42)
        schema = self.api.to_json_schema()
        # Schema should be a dict with function info
        self.assertIsInstance(schema, dict)
        # Should contain our test function
        self.assertIn("test_fn", schema)


if __name__ == "__main__":
    print("=" * 60)
    print("  FFI UNIT TESTS")
    print("=" * 60)
    unittest.main(verbosity=2)
