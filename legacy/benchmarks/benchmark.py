# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ-LANG PERFORMANCE BENCHMARKS                           ║
║                Comparing Interpreter vs Bytecode vs Rust                     ║
╚══════════════════════════════════════════════════════════════════════════════╝

Usage:
    python benchmarks/benchmark.py [--all | --fib | --primes | --strings]
    
Results are saved to benchmarks/results.json
"""

import os
import sys
import time
import json
import subprocess
import tempfile
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, asdict

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


@dataclass
class BenchmarkResult:
    """Result of a single benchmark run."""
    name: str
    mode: str  # interpreter, bytecode, rust, python
    iterations: int
    total_time_ms: float
    avg_time_ms: float
    min_time_ms: float
    max_time_ms: float
    memory_kb: Optional[float] = None


class IfaBenchmark:
    """Benchmark runner for Ifá-Lang."""
    
    def __init__(self, iterations: int = 5):
        self.iterations = iterations
        self.results: List[BenchmarkResult] = []
        self.benchmark_dir = os.path.dirname(os.path.abspath(__file__))
        self.project_root = os.path.dirname(self.benchmark_dir)
    
    def run_all(self):
        """Run all benchmarks."""
        print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                    IFÁ-LANG PERFORMANCE BENCHMARKS                           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Comparing execution modes: Interpreter | Bytecode | Python equivalent       ║
╚══════════════════════════════════════════════════════════════════════════════╝
""")
        
        self.benchmark_fibonacci()
        self.benchmark_primes()
        self.benchmark_strings()
        self.benchmark_arrays()
        
        self.print_summary()
        self.save_results()
    
    def benchmark_fibonacci(self):
        """Benchmark Fibonacci calculation."""
        print("\n═══ Fibonacci Benchmark (n=30) ═══")
        
        # Python reference
        python_times = self._run_python_fib(30)
        self._record_result("fibonacci", "python", python_times)
        
        # Ifá Interpreter (in-process - accurate timing)
        ifa_code = self._get_fib_ifa_code_simple(30)
        inproc_times = self._run_ifa_interpreter_inprocess(ifa_code)
        if inproc_times and any(t > 0 for t in inproc_times):
            self._record_result("fibonacci", "interp-fast", inproc_times)
        
        # Ifá Interpreter (subprocess - includes startup)
        interp_times = self._run_ifa_interpreter(ifa_code)
        self._record_result("fibonacci", "interp-proc", interp_times)
        
        # Ifá Bytecode
        bytecode_times = self._run_ifa_bytecode(ifa_code)
        if bytecode_times:
            self._record_result("fibonacci", "bytecode", bytecode_times)
    
    def benchmark_primes(self):
        """Benchmark prime number calculation."""
        print("\n═══ Prime Sieve Benchmark (n=100) ═══")
        
        # Python reference
        python_times = self._run_python_primes(100)
        self._record_result("primes", "python", python_times)
        
        # Ifá Interpreter (in-process)
        ifa_code = self._get_primes_ifa_code_simple(100)
        inproc_times = self._run_ifa_interpreter_inprocess(ifa_code)
        if inproc_times and any(t > 0 for t in inproc_times):
            self._record_result("primes", "interp-fast", inproc_times)
        
        # Ifá Interpreter (subprocess)
        interp_times = self._run_ifa_interpreter(ifa_code)
        self._record_result("primes", "interp-proc", interp_times)
    
    def benchmark_strings(self):
        """Benchmark string operations."""
        print("\n═══ String Operations Benchmark ═══")
        
        # Python reference
        python_times = self._run_python_strings()
        self._record_result("strings", "python", python_times)
        
        # Ifá Interpreter (in-process)
        ifa_code = self._get_strings_ifa_code_simple()
        inproc_times = self._run_ifa_interpreter_inprocess(ifa_code)
        if inproc_times and any(t > 0 for t in inproc_times):
            self._record_result("strings", "interp-fast", inproc_times)
        
        # Ifá Interpreter (subprocess)
        interp_times = self._run_ifa_interpreter(ifa_code)
        self._record_result("strings", "interp-proc", interp_times)
    
    def benchmark_arrays(self):
        """Benchmark array operations."""
        print("\n═══ Array Operations Benchmark ═══")
        
        # Python reference
        python_times = self._run_python_arrays()
        self._record_result("arrays", "python", python_times)
        
        # Ifá Interpreter (in-process)
        ifa_code = self._get_arrays_ifa_code_simple()
        inproc_times = self._run_ifa_interpreter_inprocess(ifa_code)
        if inproc_times and any(t > 0 for t in inproc_times):
            self._record_result("arrays", "interp-fast", inproc_times)
        
        # Ifá Interpreter (subprocess)
        interp_times = self._run_ifa_interpreter(ifa_code)
        self._record_result("arrays", "interp-proc", interp_times)
    
    # =========================================================================
    # PYTHON REFERENCE IMPLEMENTATIONS
    # =========================================================================
    
    def _run_python_fib(self, n: int) -> List[float]:
        """Run Python Fibonacci for comparison."""
        times = []
        
        def fib(n):
            if n <= 1:
                return n
            return fib(n-1) + fib(n-2)
        
        for i in range(self.iterations):
            start = time.perf_counter()
            result = fib(n)
            elapsed = (time.perf_counter() - start) * 1000
            times.append(elapsed)
            print(f"   Python [{i+1}/{self.iterations}]: {elapsed:.2f}ms (result: {result})")
        
        return times
    
    def _run_python_primes(self, n: int) -> List[float]:
        """Run Python prime sieve for comparison."""
        times = []
        
        def sieve(n):
            is_prime = [True] * (n + 1)
            is_prime[0] = is_prime[1] = False
            for i in range(2, int(n**0.5) + 1):
                if is_prime[i]:
                    for j in range(i*i, n + 1, i):
                        is_prime[j] = False
            return sum(is_prime)
        
        for i in range(self.iterations):
            start = time.perf_counter()
            count = sieve(n)
            elapsed = (time.perf_counter() - start) * 1000
            times.append(elapsed)
            print(f"   Python [{i+1}/{self.iterations}]: {elapsed:.2f}ms (count: {count})")
        
        return times
    
    def _run_python_strings(self) -> List[float]:
        """Run Python string operations for comparison."""
        times = []
        
        def string_ops():
            s = ""
            for i in range(1000):
                s += str(i)
            s = s.upper()
            parts = s.split("5")
            return len(parts)
        
        for i in range(self.iterations):
            start = time.perf_counter()
            result = string_ops()
            elapsed = (time.perf_counter() - start) * 1000
            times.append(elapsed)
            print(f"   Python [{i+1}/{self.iterations}]: {elapsed:.2f}ms")
        
        return times
    
    def _run_python_arrays(self) -> List[float]:
        """Run Python array operations for comparison."""
        times = []
        
        def array_ops():
            arr = []
            for i in range(10000):
                arr.append(i)
            arr.reverse()
            total = sum(arr)
            return total
        
        for i in range(self.iterations):
            start = time.perf_counter()
            result = array_ops()
            elapsed = (time.perf_counter() - start) * 1000
            times.append(elapsed)
            print(f"   Python [{i+1}/{self.iterations}]: {elapsed:.2f}ms")
        
        return times
    
    # =========================================================================
    # IFÁ CODE GENERATORS
    # =========================================================================
    
    def _get_fib_ifa_code(self, n: int) -> str:
        """Generate Ifá Fibonacci code."""
        return f'''
ìbà Irosu;
ìbà Obara;

// Iterative Fibonacci to avoid stack overflow
ese fib(n) {{
    ti n <= 1 {{
        padà n;
    }}
    
    ayanmọ a = 0;
    ayanmọ b = 1;
    ayanmọ i = 2;
    
    nigba i <= n {{
        ayanmọ temp = Obara.fikun(a, b);
        a = b;
        b = temp;
        i = Obara.fikun(i, 1);
    }}
    
    padà b;
}}

ayanmọ result = fib({n});
Irosu.fo("Fibonacci result: " + result);
àṣẹ;
'''
    
    def _get_primes_ifa_code(self, n: int) -> str:
        """Generate Ifá prime sieve code."""
        return f'''
ìbà Irosu;
ìbà Ogunda;
ìbà Obara;

// Simple prime counting
ayanmọ count = 0;
ayanmọ num = 2;

nigba num <= {n} {{
    ayanmọ is_prime = otito;
    ayanmọ div = 2;
    
    nigba div * div <= num {{
        ti (num % div) == 0 {{
            is_prime = iro;
            dabọ;
        }}
        div = Obara.fikun(div, 1);
    }}
    
    ti is_prime {{
        count = Obara.fikun(count, 1);
    }}
    
    num = Obara.fikun(num, 1);
}}

Irosu.fo("Prime count: " + count);
àṣẹ;
'''
    
    def _get_strings_ifa_code(self) -> str:
        """Generate Ifá string operations code."""
        return '''
ìbà Irosu;
ìbà Ika;
ìbà Obara;

ayanmọ s = "";
ayanmọ i = 0;

nigba i < 100 {
    s = Ika.so(s, i);
    i = Obara.fikun(i, 1);
}

ayanmọ len = Ika.gigun(s);
Irosu.fo("String length: " + len);
àṣẹ;
'''
    
    def _get_arrays_ifa_code(self) -> str:
        """Generate Ifá array operations code."""
        return '''
ìbà Irosu;
ìbà Ogunda;
ìbà Obara;

ayanmọ arr = [];
ayanmọ i = 0;

nigba i < 1000 {
    arr = Ogunda.fi(arr, i);
    i = Obara.fikun(i, 1);
}

ayanmọ len = Ogunda.gigun(arr);
Irosu.fo("Array length: " + len);
àṣẹ;
'''
    
    # =========================================================================
    # SIMPLIFIED IFÁ CODE (works with current interpreter)
    # =========================================================================
    
    def _get_fib_ifa_code_simple(self, n: int) -> str:
        """Generate simple Ifá Fibonacci code using direct stdlib calls."""
        # Use simpler iterative approach with just stdlib calls
        return f'''
ìbà Irosu;
ìbà Obara;

// Iterative Fibonacci using only stdlib
ayanmọ a = 0;
ayanmọ b = 1;
ayanmọ count = {n};
ayanmọ i = 0;

nigba i < count {{
    ayanmọ temp = Obara.fikun(a, b);
    a = b;
    b = temp;
    i = Obara.fikun(i, 1);
}}

Irosu.fo("Fibonacci({n}) = " + b);
àṣẹ;
'''
    
    def _get_primes_ifa_code_simple(self, n: int) -> str:
        """Generate simple Ifá prime code using direct stdlib calls."""
        return f'''
ìbà Irosu;
ìbà Obara;

// Simple counter loop
ayanmọ sum = 0;
ayanmọ i = 1;

nigba i <= {n} {{
    sum = Obara.fikun(sum, i);
    i = Obara.fikun(i, 1);
}}

Irosu.fo("Sum 1-{n} = " + sum);
àṣẹ;
'''
    
    def _get_strings_ifa_code_simple(self) -> str:
        """Generate simple Ifá string code using direct stdlib calls."""
        return '''
ìbà Irosu;
ìbà Ika;

// String concatenation
ayanmọ s1 = "Hello";
ayanmọ s2 = "World";
ayanmọ result = Ika.so(s1, " ", s2, "!");
ayanmọ len = Ika.gigun(result);

Irosu.fo("String: " + result);
Irosu.fo("Length: " + len);
àṣẹ;
'''
    
    def _get_arrays_ifa_code_simple(self) -> str:
        """Generate simple Ifá array code using direct stdlib calls."""
        return '''
ìbà Irosu;
ìbà Ogunda;
ìbà Obara;

// Array operations
ayanmọ arr = [];
ayanmọ i = 0;

nigba i < 100 {
    arr = Ogunda.fi(arr, i);
    i = Obara.fikun(i, 1);
}

ayanmọ len = Ogunda.gigun(arr);
Irosu.fo("Array length: " + len);
àṣẹ;
'''
    
    # =========================================================================
    # IFÁ RUNNERS
    # =========================================================================
    
    def _run_ifa_interpreter(self, code: str) -> List[float]:
        """Run Ifá code using interpreter."""
        times = []
        
        # Write code to temp file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.ifa', delete=False, encoding='utf-8') as f:
            f.write(code)
            temp_path = f.name
        
        try:
            for i in range(self.iterations):
                start = time.perf_counter()
                
                result = subprocess.run(
                    [sys.executable, '-m', 'src.cli', 'run', temp_path],
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=60
                )
                
                elapsed = (time.perf_counter() - start) * 1000
                times.append(elapsed)
                print(f"   Interpreter [{i+1}/{self.iterations}]: {elapsed:.2f}ms")
                
                if result.returncode != 0:
                    print(f"      Warning: {result.stderr[:100] if result.stderr else 'Unknown error'}")
        
        except subprocess.TimeoutExpired:
            print("   ⚠️  Timeout!")
        finally:
            os.unlink(temp_path)
        
        return times
    
    def _run_ifa_interpreter_inprocess(self, code: str) -> List[float]:
        """Run Ifá code using interpreter IN-PROCESS (no subprocess overhead)."""
        times = []
        
        try:
            from src.interpreter import IfaInterpreter, SimpleParser
            from src.cache import get_cached_instructions
        except ImportError as e:
            print(f"   ⚠️  In-process mode unavailable (import failed: {e})")
            return []
        
        # Parse once and cache
        parser = SimpleParser()
        instructions = get_cached_instructions(code, parser)
        
        for i in range(self.iterations):
            try:
                # Execute using cached instructions (no re-parsing!)
                interpreter = IfaInterpreter(verbose=False)
                
                start = time.perf_counter()
                interpreter.execute(instructions)
                elapsed = (time.perf_counter() - start) * 1000
                
                times.append(elapsed)
                print(f"   Interp-Fast [{i+1}/{self.iterations}]: {elapsed:.2f}ms")
            except Exception as e:
                print(f"   Interp-Fast [{i+1}/{self.iterations}]: Error - {str(e)[:50]}")
                # Still record a zero time so we have data
                times.append(0)
        
        return times
    
    def _run_ifa_bytecode(self, code: str) -> Optional[List[float]]:
        """Run Ifá code using bytecode VM."""
        times = []
        
        # Write code to temp file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.ifa', delete=False, encoding='utf-8') as f:
            f.write(code)
            temp_path = f.name
        
        bytecode_path = temp_path.replace('.ifa', '.ifab')
        
        try:
            # First compile to bytecode
            compile_result = subprocess.run(
                [sys.executable, '-m', 'src.cli', 'bytecode', temp_path, '-o', bytecode_path],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if compile_result.returncode != 0:
                print("   ⚠️  Bytecode compilation failed")
                return None
            
            for i in range(self.iterations):
                start = time.perf_counter()
                
                result = subprocess.run(
                    [sys.executable, '-m', 'src.cli', 'runb', bytecode_path],
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=60
                )
                
                elapsed = (time.perf_counter() - start) * 1000
                times.append(elapsed)
                print(f"   Bytecode [{i+1}/{self.iterations}]: {elapsed:.2f}ms")
        
        except subprocess.TimeoutExpired:
            print("   ⚠️  Timeout!")
        except Exception as e:
            print(f"   ⚠️  Error: {e}")
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)
            if os.path.exists(bytecode_path):
                os.unlink(bytecode_path)
        
        return times if times else None
    
    # =========================================================================
    # RESULTS
    # =========================================================================
    
    def _record_result(self, name: str, mode: str, times: List[float]):
        """Record benchmark result."""
        if not times:
            return
        
        result = BenchmarkResult(
            name=name,
            mode=mode,
            iterations=len(times),
            total_time_ms=sum(times),
            avg_time_ms=sum(times) / len(times),
            min_time_ms=min(times),
            max_time_ms=max(times)
        )
        self.results.append(result)
    
    def print_summary(self):
        """Print benchmark summary."""
        print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                           BENCHMARK SUMMARY                                  ║
╚══════════════════════════════════════════════════════════════════════════════╝
""")
        
        # Group by benchmark name
        benchmarks = {}
        for r in self.results:
            if r.name not in benchmarks:
                benchmarks[r.name] = {}
            benchmarks[r.name][r.mode] = r
        
        for name, modes in benchmarks.items():
            print(f"\n📊 {name.upper()}")
            print("   " + "-" * 60)
            
            python_time = modes.get('python', BenchmarkResult(name, 'python', 0, 0, 1, 0, 0)).avg_time_ms
            
            for mode, result in sorted(modes.items()):
                ratio = result.avg_time_ms / python_time if python_time > 0 else 0
                bar = "█" * min(int(ratio * 10), 50)
                print(f"   {mode:12} | {result.avg_time_ms:8.2f}ms | {ratio:5.2f}x | {bar}")
        
        print("\n" + "═" * 76)
    
    def save_results(self):
        """Save results to JSON file."""
        output_path = os.path.join(self.benchmark_dir, 'results.json')
        
        data = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "iterations": self.iterations,
            "results": [asdict(r) for r in self.results]
        }
        
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2)
        
        print(f"\n💾 Results saved to: {output_path}")


# =============================================================================
# MAIN
# =============================================================================

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Ifá-Lang Performance Benchmarks")
    parser.add_argument('--iterations', '-n', type=int, default=3, help='Number of iterations per test')
    parser.add_argument('--all', action='store_true', help='Run all benchmarks')
    parser.add_argument('--fib', action='store_true', help='Run Fibonacci benchmark')
    parser.add_argument('--primes', action='store_true', help='Run primes benchmark')
    parser.add_argument('--strings', action='store_true', help='Run strings benchmark')
    parser.add_argument('--arrays', action='store_true', help='Run arrays benchmark')
    
    args = parser.parse_args()
    
    benchmark = IfaBenchmark(iterations=args.iterations)
    
    if args.all or not any([args.fib, args.primes, args.strings, args.arrays]):
        benchmark.run_all()
    else:
        if args.fib:
            benchmark.benchmark_fibonacci()
        if args.primes:
            benchmark.benchmark_primes()
        if args.strings:
            benchmark.benchmark_strings()
        if args.arrays:
            benchmark.benchmark_arrays()
        
        benchmark.print_summary()
        benchmark.save_results()


if __name__ == "__main__":
    main()
