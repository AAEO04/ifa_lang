# -*- coding: utf-8 -*-
"""Unit tests for CEN model modules: ebo.py, ajose.py, directives.py, gpc.py, dispatch.py"""

import sys
import os
import unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.ebo import EboBlock, SacrificeRegistry, ase_block
from src.ajose import AjosePredicateEngine, Relationship
from src.directives import DirectiveParser, OponSize, TargetPlatform, parse_directives
from src.gpc import GPCStack, GPCDebugger
from src.dispatch import SemanticDispatcher, OduType, TypeSignature


class TestEboBlock(unittest.TestCase):
    """Test Ẹbọ sacrifice resource management."""
    
    def test_acquire_resource(self):
        """Should track acquired resources."""
        with EboBlock("test") as ebo:
            resource = ebo.acquire("file", {"data": 42})
            self.assertEqual(resource["data"], 42)
    
    def test_cleanup_called(self):
        """Cleanup function should be called on exit."""
        cleaned = [False]
        def cleanup(r):
            cleaned[0] = True
        
        with EboBlock("test") as ebo:
            ebo.acquire("res", "value", cleanup)
        
        self.assertTrue(cleaned[0])
    
    def test_cleanup_order(self):
        """Resources should be cleaned in reverse order."""
        order = []
        with EboBlock("test") as ebo:
            ebo.acquire("first", 1, lambda x: order.append("first"))
            ebo.acquire("second", 2, lambda x: order.append("second"))
        
        self.assertEqual(order, ["second", "first"])


class TestAjosePredicateEngine(unittest.TestCase):
    """Test Àjọṣe reactive relationships."""
    
    def setUp(self):
        self.engine = AjosePredicateEngine()
    
    def test_define_relationship(self):
        """Should define relationship patterns."""
        rel = self.engine.define("Transfer", "Wallet", "Wallet")
        self.assertEqual(rel.name, "Transfer")
        self.assertIn("Transfer", self.engine.relationships)
    
    def test_subscribe_and_notify(self):
        """Callbacks should be notified on link."""
        triggered = [False]
        
        self.engine.define("Test", "A", "B")
        @self.engine.when("Test")
        def on_test(src, tgt, ctx):
            triggered[0] = True
        
        class Obj: pass
        self.engine.link("Test", Obj(), Obj())
        
        self.assertTrue(triggered[0])
    
    def test_bidirectional_relationship(self):
        """Bidirectional links should work both ways."""
        self.engine.define("Friend", "Person", "Person", bidirectional=True)
        
        class Person: pass
        p1, p2 = Person(), Person()
        self.engine.link("Friend", p1, p2)
        
        related = self.engine.get_related(p2, "Friend")
        self.assertIn(p1, related)


class TestDirectiveParser(unittest.TestCase):
    """Test preprocessing directive parsing."""
    
    def setUp(self):
        self.parser = DirectiveParser()
    
    def test_parse_opon(self):
        """Should parse #opon directives."""
        source = "#opon mega\nlet x = 5;"
        directives, clean = self.parser.parse(source)
        self.assertEqual(directives.opon_size, OponSize.MEGA)
    
    def test_parse_target(self):
        """Should parse #target directives."""
        source = "#target rust\nprint('hi');"
        directives, clean = self.parser.parse(source)
        self.assertEqual(directives.target, TargetPlatform.RUST)
    
    def test_parse_ewo(self):
        """Should parse #ewọ directives."""
        source = "#ewọ Osa -> Irosu\n"
        directives, clean = self.parser.parse(source)
        self.assertEqual(len(directives.ewos), 1)
        self.assertEqual(directives.ewos[0].source_domain, "OSA")
    
    def test_clean_source(self):
        """Should remove directives from clean source."""
        source = "#opon mega\nlet x = 5;"
        directives, clean = self.parser.parse(source)
        self.assertNotIn("#opon", clean)
        self.assertIn("let x = 5", clean)


class TestGPCStack(unittest.TestCase):
    """Test Grandparent-Parent-Child call stack."""
    
    def setUp(self):
        self.stack = GPCStack()
    
    def test_push_pop(self):
        """Should push and pop frames."""
        self.stack.push("main", line=1)
        self.assertEqual(len(self.stack), 1)
        
        frame = self.stack.pop()
        self.assertEqual(frame.name, "main")
        self.assertEqual(len(self.stack), 0)
    
    def test_gpc_view(self):
        """Should provide GPC hierarchy view."""
        self.stack.push("grandparent", line=1)
        self.stack.push("parent", line=10)
        self.stack.push("child", line=20)
        
        gpc = self.stack.get_gpc()
        self.assertEqual(gpc["grandparent"].name, "grandparent")
        self.assertEqual(gpc["parent"].name, "parent")
        self.assertEqual(gpc["child"].name, "child")
    
    def test_traceback(self):
        """Should generate traceback string."""
        self.stack.push("main", line=1, file="app.ifa")
        self.stack.push("process", line=10, file="app.ifa")
        
        tb = self.stack.traceback()
        self.assertIn("main", tb)
        self.assertIn("process", tb)


class TestSemanticDispatcher(unittest.TestCase):
    """Test Odù type signature dispatch."""
    
    def setUp(self):
        self.dispatcher = SemanticDispatcher()
    
    def test_resolve_string_method(self):
        """Should resolve string methods to IKA domain."""
        domain = self.dispatcher.resolve("ge", ["hello"])
        self.assertEqual(domain, "IKA")
    
    def test_resolve_array_method(self):
        """Should resolve array methods to OGUNDA domain."""
        domain = self.dispatcher.resolve("ge", [[1, 2, 3]])
        self.assertEqual(domain, "OGUNDA")
    
    def test_type_signature_matching(self):
        """TypeSignature should match arguments."""
        sig = TypeSignature("IKA", "ge", [OduType.STRING], OduType.STRING)
        self.assertTrue(sig.matches(["test"]))
        self.assertFalse(sig.matches([123]))
    
    def test_dispatch_returns_signature(self):
        """dispatch() should return domain and signature."""
        domain, sig = self.dispatcher.dispatch("fo", [42])
        self.assertEqual(domain, "IROSU")


if __name__ == "__main__":
    print("=" * 60)
    print("  CEN MODEL UNIT TESTS")
    print("=" * 60)
    unittest.main(verbosity=2)
