# -*- coding: utf-8 -*-
"""Ifá-Lang 2026 CEN Model - Integration Test"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def test_phase1_memory():
    from src.memory import OponSize, OponCalabash
    opon = OponCalabash(OponSize.MEGA)
    assert opon.total_bytes == 1024 * 1024
    opon.write(0, 42)
    assert opon.read(0) == 42
    print("✅ Phase 1: OponCalabash (1MB) PASSED")

def test_phase1b_divination():
    from src.interpreter import DivinationBlock
    div = DivinationBlock()
    assert "Ogbè" in div.evaluate(0.05)   # 0.0-0.125 = Ogbè
    assert "Ọ̀sá" in div.evaluate(0.55)   # 0.5-0.625 = Ọ̀sá
    print("✅ Phase 1b: DivinationBlock PASSED")

def test_phase2_isa():
    from src.isa import Ifa16BitISA
    isa = Ifa16BitISA()
    opcode = isa.encode_semantic("IROSU", "WRITE")
    decoded = isa.decode(opcode)
    assert decoded['action'] == "WRITE"
    print("✅ Phase 2: Ifa16BitISA (65K ops) PASSED")

def test_phase3_taint():
    from src.validator import TaintTracker
    tracker = TaintTracker()
    tracker.add_forbidden_flow("osa", "irosu")
    tracker.taint_variable("data", "osa")
    assert tracker.check_flow("data", "irosu") == False
    print("✅ Phase 3: TaintTracker PASSED")

def test_phase4_ffi():
    from src.ffi import SecureFFI, SecurityError
    ffi = SecureFFI()
    math = ffi.import_module("math")
    assert math.sqrt(16) == 4.0
    try:
        ffi.import_module("os")
        assert False
    except SecurityError:
        pass
    print("✅ Phase 4: SecureFFI PASSED")

def test_phase5_ajose():
    from src.ajose import AjosePredicateEngine
    engine = AjosePredicateEngine()
    engine.define("Test", "A", "B")
    triggered = [False]
    @engine.when("Test")
    def on_test(src, tgt, ctx): triggered[0] = True
    class Obj: pass
    engine.link("Test", Obj(), Obj())
    assert triggered[0]
    print("✅ Phase 5: AjosePredicateEngine PASSED")

def test_phase6_debugger():
    from src.errors import StateHistoryBuffer, HarmonyHeatmap
    buf = StateHistoryBuffer()
    for i in range(5): buf.push({"line": i, "vars": {"x": i}})
    assert len(buf) == 5
    hm = HarmonyHeatmap()
    hm.record_interaction("OSA", "IROSU", 0.2)
    assert hm.get_harmony("OSA", "IROSU") == 0.2
    print("✅ Phase 6: StateHistoryBuffer + HarmonyHeatmap PASSED")

def test_phase7_ui():
    sys.path.insert(0, ".")
    from lib.std.ose_ui import OseUIEngine
    engine = OseUIEngine()
    comp = engine.create("test", "div", "Hello", "OGBE")
    engine.set_state(comp, "OYEKU")
    assert comp.odu_state == "OYEKU"
    print("✅ Phase 7: OseUIEngine PASSED")

def test_phase8_ebo():
    from src.ebo import EboBlock
    sacrificed = [False]
    class Res:
        def cleanup(self): sacrificed[0] = True
    with EboBlock("test") as ebo:
        ebo.acquire("res", Res(), lambda x: x.cleanup())
    assert sacrificed[0]
    print("✅ Phase 8: Ẹbọ Sacrifice PASSED")

def test_phase9_directives():
    from src.directives import parse_directives
    source = "#opon mega\n#target rust\n#ewọ Osa -> Irosu\nlet x = 5;"
    directives, clean = parse_directives(source)
    assert directives.opon_size.value == "mega"
    assert directives.target.value == "rust"
    assert len(directives.ewos) == 1
    print("✅ Phase 9: DirectiveParser PASSED")

def test_phase10_gpc():
    from src.gpc import GPCStack
    stack = GPCStack()
    stack.push("main", 1, "app.ifa", "OGBE")
    stack.push("process", 15, "app.ifa", "OGUNDA")
    gpc = stack.get_gpc()
    assert gpc["child"].name == "process"
    assert gpc["parent"].name == "main"
    print("✅ Phase 10: GPCStack PASSED")

def test_phase11_dispatch():
    from src.dispatch import SemanticDispatcher
    disp = SemanticDispatcher()
    assert disp.resolve("ge", ["hello"]) == "IKA"
    assert disp.resolve("ge", [[1,2,3]]) == "OGUNDA"
    print("✅ Phase 11: SemanticDispatcher PASSED")

def main():
    print("=" * 60)
    print("  IFÁ-LANG 2026 CEN MODEL - INTEGRATION TEST")
    print("=" * 60 + "\n")
    
    tests = [test_phase1_memory, test_phase1b_divination, test_phase2_isa,
             test_phase3_taint, test_phase4_ffi, test_phase5_ajose,
             test_phase6_debugger, test_phase7_ui, test_phase8_ebo,
             test_phase9_directives, test_phase10_gpc, test_phase11_dispatch]
    
    passed, failed = 0, 0
    for test in tests:
        try:
            test()
            passed += 1
        except Exception as e:
            print(f"❌ {test.__name__}: {e}")
            failed += 1
    
    print(f"\n{'=' * 60}")
    print(f"  RESULTS: {passed} passed, {failed} failed")
    print("=" * 60)
    return failed == 0

if __name__ == "__main__":
    sys.exit(0 if main() else 1)
