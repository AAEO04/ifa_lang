# -*- coding: utf-8 -*-
"""
Master test runner for all Ifá-Lang unit tests.
Run with: python tests/run_all_tests.py
"""

import sys
import os
import unittest

# Add project root to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def run_all_tests():
    """Discover and run all tests."""
    print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ-LANG COMPLETE TEST SUITE                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
""")
    
    # Discover all tests
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()
    
    # Get tests directory
    tests_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Discover tests
    discovered = loader.discover(tests_dir, pattern='test_*.py')
    suite.addTests(discovered)
    
    # Run with verbosity
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    
    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print(f"  Tests run: {result.testsRun}")
    print(f"  Failures:  {len(result.failures)}")
    print(f"  Errors:    {len(result.errors)}")
    print(f"  Skipped:   {len(result.skipped)}")
    
    if result.wasSuccessful():
        print("\n  ✅ ALL TESTS PASSED!")
    else:
        print("\n  ❌ SOME TESTS FAILED")
        if result.failures:
            print("\n  FAILURES:")
            for test, traceback in result.failures:
                print(f"    - {test}")
        if result.errors:
            print("\n  ERRORS:")
            for test, traceback in result.errors:
                print(f"    - {test}")
    
    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)
