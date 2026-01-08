# -*- coding: utf-8 -*-
"""Unit tests for memory.py - Ọpọ́n memory systems."""

import sys
import os
import unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.memory import (
    ODU_NAMES, NAME_TO_INDEX, INDEX_TO_NAME,
    Odu12Bit, Calabash4K, IfaStandardLibrary,
    OponSize, OponCalabash
)


class TestOduNames(unittest.TestCase):
    """Test the 16 Principal Odù definitions."""
    
    def test_16_principal_odu(self):
        """Should have exactly 16 principal Odù."""
        self.assertEqual(len(ODU_NAMES), 16)
    
    def test_odu_names_uppercase(self):
        """All Odù names should be uppercase."""
        for name in ODU_NAMES:
            self.assertEqual(name, name.upper())
    
    def test_name_to_index_mapping(self):
        """NAME_TO_INDEX should map names to indices 0-15."""
        self.assertEqual(NAME_TO_INDEX["OGBE"], 0)
        self.assertEqual(NAME_TO_INDEX["OFUN"], 15)
    
    def test_index_to_name_mapping(self):
        """INDEX_TO_NAME should be inverse of NAME_TO_INDEX."""
        for name, idx in NAME_TO_INDEX.items():
            self.assertEqual(INDEX_TO_NAME[idx], name)


class TestOdu12Bit(unittest.TestCase):
    """Test 12-bit Odù encoding/decoding."""
    
    def test_encode_zero(self):
        """OGBE.OGBE.OGBE should encode to 0."""
        self.assertEqual(Odu12Bit.encode(0, 0, 0), 0)
    
    def test_encode_max(self):
        """OFUN.OFUN.OFUN should encode to 4095."""
        self.assertEqual(Odu12Bit.encode(15, 15, 15), 4095)
    
    def test_decode_round_trip(self):
        """Encode then decode should return original."""
        for addr in [0, 100, 1000, 4095]:
            encoded = addr
            a, b, c = Odu12Bit.decode(encoded)
            self.assertEqual(Odu12Bit.encode(a, b, c), addr)
    
    def test_to_string_format(self):
        """to_string should return Odù.Odù.Odù format."""
        result = Odu12Bit.to_string(0)
        self.assertIn("OGBE", result)
    
    def test_from_string(self):
        """from_string should parse Odù notation."""
        addr = Odu12Bit.from_string("OGBE-OGBE-OGBE")
        self.assertEqual(addr, 0)


class TestCalabash4K(unittest.TestCase):
    """Test 4KB memory calabash."""
    
    def setUp(self):
        self.mem = Calabash4K()
    
    def test_initial_memory_zero(self):
        """Memory should be initialized to zeros."""
        self.assertEqual(self.mem.read(0), 0)
        self.assertEqual(self.mem.read(4095), 0)
    
    def test_write_read(self):
        """Write then read should return written value."""
        self.mem.write(100, 42)
        self.assertEqual(self.mem.read(100), 42)
    
    def test_write_byte_truncation(self):
        """Values > 255 should be truncated to byte."""
        self.mem.write(100, 256)
        self.assertEqual(self.mem.read(100), 0)  # 256 & 0xFF = 0
        
        self.mem.write(100, 257)
        self.assertEqual(self.mem.read(100), 1)  # 257 & 0xFF = 1
    
    def test_address_wrapping(self):
        """Addresses > 4095 should wrap."""
        self.mem.write(4096, 99)  # Wraps to 0
        self.assertEqual(self.mem.read(0), 99)
    
    def test_clear(self):
        """Clear should reset all memory."""
        self.mem.write(100, 42)
        self.mem.clear()
        self.assertEqual(self.mem.read(100), 0)
    
    def test_string_address(self):
        """Should accept Odù string addresses."""
        self.mem.write("OGBE-OGBE-OYEKU", 77)
        self.assertEqual(self.mem.read("OGBE-OGBE-OYEKU"), 77)
    
    def test_access_log(self):
        """Access log should track operations."""
        self.mem.write(10, 5)
        self.mem.read(10)
        self.assertEqual(len(self.mem.access_log), 2)
    
    def test_get_region(self):
        """Should return correct memory region names."""
        region = self.mem.get_region(0x000)
        self.assertIsNotNone(region)  # Just verify it returns a region name


class TestOponSize(unittest.TestCase):
    """Test dynamic Ọpọ́n size enum."""
    
    def test_size_values(self):
        """Size values should be correct byte counts."""
        self.assertEqual(OponSize.KEKERE.value, 4 * 1024)
        self.assertEqual(OponSize.GIDI.value, 16 * 1024)
        self.assertEqual(OponSize.NLA.value, 64 * 1024)
        self.assertEqual(OponSize.MEGA.value, 1024 * 1024)


class TestOponCalabash(unittest.TestCase):
    """Test scalable memory workspace."""
    
    def test_default_size(self):
        """Default size should be GIDI (16KB)."""
        opon = OponCalabash()
        self.assertEqual(opon.total_bytes, 16 * 1024)
    
    def test_mega_size(self):
        """MEGA size should be 1MB."""
        opon = OponCalabash(OponSize.MEGA)
        self.assertEqual(opon.total_bytes, 1024 * 1024)
    
    def test_write_read(self):
        """Write then read should work."""
        opon = OponCalabash(OponSize.KEKERE)
        opon.write(100, 42)
        self.assertEqual(opon.read(100), 42)
    
    def test_out_of_bounds_error(self):
        """Out of bounds access should raise IndexError."""
        opon = OponCalabash(OponSize.KEKERE)  # 4KB
        with self.assertRaises(IndexError):
            opon.read(5000)
    
    def test_allocate_region(self):
        """Should allocate named memory regions."""
        opon = OponCalabash(OponSize.GIDI)
        start = opon.allocate_region("buffer", 1024)
        self.assertEqual(start, 0)
        
        start2 = opon.allocate_region("data", 1024)
        self.assertEqual(start2, 1024)
    
    def test_allocate_overflow(self):
        """Should raise MemoryError when full."""
        opon = OponCalabash(OponSize.KEKERE)  # 4KB
        opon.allocate_region("big", 4000)
        with self.assertRaises(MemoryError):
            opon.allocate_region("too_big", 1000)
    
    def test_clear(self):
        """Clear should reset memory and regions."""
        opon = OponCalabash()
        opon.write(100, 42)
        opon.allocate_region("test", 100)
        opon.clear()
        self.assertEqual(opon.read(100), 0)
        self.assertEqual(len(opon._regions), 0)


class TestIfaStandardLibrary(unittest.TestCase):
    """Test standard library function registry."""
    
    def setUp(self):
        self.lib = IfaStandardLibrary()
    
    def test_register_function(self):
        """Should register functions at Odù addresses."""
        # Use an address not pre-registered by stdlib
        self.lib.register("OFUN-OFUN-OFUN", "test_func", "A test function")
        func = self.lib.lookup("test_func")
        self.assertIsNotNone(func)
        self.assertEqual(func["name"], "test_func")
    
    def test_get_by_name(self):
        """Should retrieve functions by name."""
        # Use unique address not in stdlib 
        self.lib.register("OFUN-OFUN-OTURA", "my_func", "Description")
        func = self.lib.lookup("my_func")
        self.assertIsNotNone(func)


if __name__ == "__main__":
    print("=" * 60)
    print("  MEMORY SYSTEM UNIT TESTS")
    print("=" * 60)
    unittest.main(verbosity=2)
