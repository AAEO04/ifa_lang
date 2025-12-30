# -*- coding: utf-8 -*-
"""
Ifá Test Runner (Idanwo)
A framework for finding and running Ifá-Lang tests.
"""
import os
import sys
import time
import subprocess
from typing import List, Dict, Any

class IfaTestRunner:
    def __init__(self, root_dir: str = "."):
        self.root_dir = root_dir
        self.tests_found = 0
        self.tests_passed = 0
        self.tests_failed = 0
        self.failed_tests = []

    def discover(self, start_dir: str = None) -> List[str]:
        """Find test files (_test.ifa or test_*.ifa)"""
        if start_dir is None:
            start_dir = self.root_dir
            
        test_files = []
        for root, _, files in os.walk(start_dir):
            for file in files:
                if (file.endswith("_test.ifa") or 
                    file.startswith("test_") and file.endswith(".ifa")):
                    test_files.append(os.path.join(root, file))
        
        return test_files

    def run_tests(self, test_files: List[str]):
        """Run a list of test files."""
        print(f"\n🧪 Found {len(test_files)} test files.\n")
        
        start_time = time.time()
        
        for file in test_files:
            self.run_single_file(file)
            
        duration = time.time() - start_time
        self._print_summary(duration)

    def run_single_file(self, filepath: str):
        """Run a single test file using the interpreter."""
        # Use python -m src.cli run <file>
        # We assume 'src' is in pythonpath
        rel_path = os.path.relpath(filepath, self.root_dir)
        print(f"RUNIT {rel_path}...", end=" ", flush=True)
        
        try:
            # We run the file via the CLI "run" command
            # This uses the Python interpreter, which is faster for tests
            # than compiling to Rust every time.
            cmd = [sys.executable, "src/cli.py", "run", filepath]
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                cwd=self.root_dir
            )
            
            if result.returncode == 0:
                print("✅ PASS")
                self.tests_passed += 1
            else:
                print("❌ FAIL")
                self.tests_failed += 1
                self.failed_tests.append({
                    "file": rel_path,
                    "error": result.stderr or result.stdout
                })
        except Exception as e:
            print("❌ ERR")
            self.tests_failed += 1
            self.failed_tests.append({
                "file": rel_path,
                "error": str(e)
            })

    def _print_summary(self, duration: float):
        """Print test results summary."""
        print("-" * 50)
        if self.tests_failed == 0:
            print(f"🎉 All {self.tests_passed} tests passed in {duration:.2f}s")
        else:
            print(f"💥 {self.tests_failed} tests failed, {self.tests_passed} passed in {duration:.2f}s")
            print("\nFailures:")
            for fail in self.failed_tests:
                print(f"\n📄 {fail['file']}:")
                # Indent error message
                print("\n".join("  " + line for line in fail['error'].strip().splitlines()))
        print("-" * 50)

def run_cli(args):
    """Entry point for CLI."""
    runner = IfaTestRunner(os.getcwd())
    
    # If explicit files passed
    if args.files:
        files = []
        for f in args.files:
            if os.path.isdir(f):
                files.extend(runner.discover(f))
            else:
                files.append(f)
    else:
        files = runner.discover()
        
    if not files:
        print("No tests found.")
        return 1
        
    runner.run_tests(files)
    return 1 if runner.tests_failed > 0 else 0
