import subprocess
import time
import os

IFA_BIN = os.path.abspath("target/release/ifa.exe")
ITERATIONS = 3

def compile_ifa():
    print("Building Ifá-Lang in release mode...")
    subprocess.run(["cargo", "build", "-p", "ifa-cli", "--release"], check=True)
    print("Build complete.")

def run_command(cmd):
    times = []
    for _ in range(ITERATIONS):
        start = time.perf_counter()
        try:
            subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        except subprocess.CalledProcessError as e:
            print(f"Error running {' '.join(cmd)}:\n{e.stderr.decode('utf-8')}")
            return None
        end = time.perf_counter()
        times.append((end - start) * 1000)
    
    return sum(times) / len(times)

def main():
    if not os.path.exists(IFA_BIN):
        compile_ifa()
        
    benchmarks = {
        "Fibonacci (n=30)": {
            "Ifá-Lang (Interpreter)": [IFA_BIN, "run", "benchmarks/cross_language/fib.ifa"],
            "Ifá-Lang (Bytecode VM)": [IFA_BIN, "runb", "benchmarks/cross_language/fib.ifab"],
            "Python 3": ["python", "benchmarks/cross_language/fib.py"],
            "Node.js": ["node", "benchmarks/cross_language/fib.js"],
        },
        "Loop Sum (10M)": {
            "Ifá-Lang (Interpreter)": [IFA_BIN, "run", "benchmarks/cross_language/loop_sum.ifa"],
            "Ifá-Lang (Bytecode VM)": [IFA_BIN, "runb", "benchmarks/cross_language/loop_sum.ifab"],
            "Python 3": ["python", "benchmarks/cross_language/loop_sum.py"],
            "Node.js": ["node", "benchmarks/cross_language/loop_sum.js"],
        }
    }
    
    # Pre-compile to bytecode
    for bench_name in ["fib", "loop_sum"]:
        subprocess.run([IFA_BIN, "bytecode", f"benchmarks/cross_language/{bench_name}.ifa"], check=True)
        
    print(f"{'Benchmark':<20} | {'Language/Runtime':<25} | {'Avg Time (ms)':<15}")
    print("-" * 65)
    
    for bench_name, langs in benchmarks.items():
        for lang_name, cmd in langs.items():
            avg_time = run_command(cmd)
            if avg_time is not None:
                print(f"{bench_name:<20} | {lang_name:<25} | {avg_time:.2f} ms")
            else:
                print(f"{bench_name:<20} | {lang_name:<25} | ERROR")
        print("-" * 65)

if __name__ == "__main__":
    main()
