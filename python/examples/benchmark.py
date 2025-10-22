"""
Performance benchmark comparing jsonrepair (Rust) with json_repair (Python)

This benchmark compares our Rust implementation with the pure Python json_repair library.
Note: Our implementation uses &str with gradient descent, which provides good performance
while offering additional features like streaming and Writer APIs.
"""

import sys
import time

try:
    import jsonrepair
    HAS_JSONREPAIR = True
except ImportError:
    HAS_JSONREPAIR = False
    print("⚠️  jsonrepair not installed. Run: pip install jsonrepair")

try:
    import json_repair
    HAS_JSON_REPAIR = True
except ImportError:
    HAS_JSON_REPAIR = False
    print("⚠️  json_repair not installed. Run: pip install json-repair")

if not HAS_JSONREPAIR or not HAS_JSON_REPAIR:
    print("\nPlease install both libraries to run the benchmark.")
    sys.exit(1)


def benchmark_case(name, broken_json, iterations=1000):
    """Benchmark a single test case"""
    print(f"\n{'='*60}")
    print(f"Test: {name}")
    print(f"Iterations: {iterations}")
    print(f"Input length: {len(broken_json)} chars")
    print(f"{'='*60}")

    # Warm up
    for _ in range(10):
        jsonrepair.loads(broken_json)
        json_repair.loads(broken_json)

    # Benchmark jsonrepair (Rust)
    start = time.perf_counter()
    for _ in range(iterations):
        result_rust = jsonrepair.loads(broken_json)
    time_rust = time.perf_counter() - start

    # Benchmark json_repair (Python)
    start = time.perf_counter()
    for _ in range(iterations):
        result_python = json_repair.loads(broken_json)
    time_python = time.perf_counter() - start

    # Verify results match
    assert result_rust == result_python, "Results don't match!"

    # Calculate speedup
    speedup = time_python / time_rust

    print("\nResults:")
    print(f"  jsonrepair (Rust):   {time_rust:.4f}s ({time_rust/iterations*1000:.3f}ms per iteration)")
    print(f"  json_repair (Python): {time_python:.4f}s ({time_python/iterations*1000:.3f}ms per iteration)")
    print(f"  Speedup: {speedup:.1f}x faster 🚀")

    return speedup


def main():
    print("🏁 jsonrepair Performance Benchmark")
    print("=" * 60)

    speedups = []

    # Test 1: Simple object
    speedup = benchmark_case(
        "Simple Object",
        "{name: 'John', age: 30,}",
        iterations=10000
    )
    speedups.append(speedup)

    # Test 2: Array with trailing comma
    speedup = benchmark_case(
        "Array with Trailing Comma",
        "[1, 2, 3, 4, 5,]",
        iterations=10000
    )
    speedups.append(speedup)

    # Test 3: Comments
    speedup = benchmark_case(
        "JSON with Comments",
        """
        {
            // Name field
            "name": "John",
            /* Age field */
            "age": 30
        }
        """,
        iterations=5000
    )
    speedups.append(speedup)

    # Test 4: Nested structure
    speedup = benchmark_case(
        "Nested Structure",
        """
        {
            name: 'John',
            address: {
                street: '123 Main St',
                city: 'New York',
                zip: 10001
            },
            hobbies: ['reading', 'swimming', 'coding']
        }
        """,
        iterations=5000
    )
    speedups.append(speedup)

    # Test 5: Large array
    large_array = "[" + ", ".join([f"{{id: {i}, name: 'Item {i}'}}" for i in range(100)]) + "]"
    speedup = benchmark_case(
        "Large Array (100 objects)",
        large_array,
        iterations=1000
    )
    speedups.append(speedup)

    # Test 6: Unicode
    speedup = benchmark_case(
        "Unicode Characters",
        "{'name': '统一码', 'emoji': '😀', 'text': 'Hello 世界'}",
        iterations=5000
    )
    speedups.append(speedup)

    # Test 7: Incomplete JSON
    speedup = benchmark_case(
        "Incomplete JSON",
        '{"name": "John", "age": 30, "address": {"street": "123 Main',
        iterations=5000
    )
    speedups.append(speedup)

    # Test 8: Fenced code block
    speedup = benchmark_case(
        "Fenced Code Block",
        """
        ```json
        {
            "name": "John",
            "age": 30,
            "active": true
        }
        ```
        """,
        iterations=5000
    )
    speedups.append(speedup)

    # Summary
    print(f"\n{'='*60}")
    print("📊 SUMMARY")
    print(f"{'='*60}")
    avg_speedup = sum(speedups) / len(speedups)
    min_speedup = min(speedups)
    max_speedup = max(speedups)

    print(f"Average speedup: {avg_speedup:.1f}x")
    print(f"Min speedup:     {min_speedup:.1f}x")
    print(f"Max speedup:     {max_speedup:.1f}x")
    print(f"\n✅ jsonrepair is {avg_speedup:.1f}x faster on average!")
    print("\nNote: Performance varies by use case. Our implementation prioritizes")
    print("feature richness (streaming, Writer API, logging) while maintaining")
    print("excellent performance compared to pure Python implementations.")


if __name__ == '__main__':
    main()

