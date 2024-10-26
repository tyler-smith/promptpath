#!/usr/bin/python3
import subprocess
import time
import statistics
import os
from pathlib import Path

def compile_rust():
    """Compile the Rust version with optimizations."""
    subprocess.run(["rustc", "-O", "promptpath.rs"], check=True)

def run_command(cmd, warmup=False):
    """Run a command and return its execution time in milliseconds."""
    start = time.perf_counter_ns()
    subprocess.run(cmd, capture_output=True, check=True)
    duration = (time.perf_counter_ns() - start) / 1_000_000  # Convert to ms

    if not warmup:
        return duration

def benchmark_command(cmd, name, iterations=1000):
    """Benchmark a command multiple times and return statistics."""
    # Warm-up runs
    for _ in range(5):
        run_command(cmd, warmup=True)

    # Actual benchmark runs
    times = []
    for _ in range(iterations):
        duration = run_command(cmd)
        times.append(duration)

    return {
        "name": name,
        "min": min(times),
        "max": max(times),
        "mean": statistics.mean(times),
        "median": statistics.median(times),
        "stddev": statistics.stdev(times)
    }

def print_results(results):
    """Print benchmark results in a formatted table."""
    print("\nBenchmark Results (times in milliseconds):")
    print("-" * 80)
    print(f"{'Program':<15} {'Min':>10} {'Max':>10} {'Mean':>10} {'Median':>10} {'StdDev':>10}")
    print("-" * 80)

    for result in results:
        print(f"{result['name']:<15} {result['min']:>10.3f} {result['max']:>10.3f} "
              f"{result['mean']:>10.3f} {result['median']:>10.3f} {result['stddev']:>10.3f}")

    # Calculate and print speedup
    py_mean = next(r["mean"] for r in results if r["name"] == "Python")
    rust_mean = next(r["mean"] for r in results if r["name"] == "Rust")
    speedup = py_mean / rust_mean
    print("\nRust is {:.1f}x faster than Python (based on mean times)".format(speedup))

def main():
    # Get absolute paths to executables
    script_dir = Path.cwd()
    py_path = script_dir / "promptpath.py"
    rust_path = script_dir / "target/release/promptpath"

    # Ensure the Python script is executable
    py_path.chmod(py_path.stat().st_mode | 0o755)

    # Test directories
    test_dirs = [
        "~/code/github.com/ethereum-optimism/optimism",
        "~/code/github.com/ethereum-optimism/optimism/op-node",
        "~/code"
    ]

    # print("Compiling Rust version...")
    # compile_rust()

    results = []

    for test_dir in test_dirs:
        expanded_dir = os.path.expanduser(test_dir)
        print(f"\nBenchmarking in directory: {test_dir}")

        # Change to test directory
        os.chdir(expanded_dir)

        # Benchmark both versions using absolute paths
        py_cmd = [str(py_path)]
        rust_cmd = [str(rust_path)]

        py_results = benchmark_command(py_cmd, "Python")
        rust_results = benchmark_command(rust_cmd, "Rust")

        print_results([py_results, rust_results])

if __name__ == "__main__":
    main()