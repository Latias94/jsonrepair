#!/usr/bin/env python3
import json
import time
import math
import argparse
import sys
from typing import List, Dict, Any

def gen_array_with_spaces(n: int, spaces: int) -> str:
    """Generate array with spaces before commas, matching Rust implementation."""
    parts = ["["]
    for i in range(n):
        if i > 0:
            parts.append(" " * spaces)
            parts.append(",")
        parts.append(str(i))
    parts.append("]")
    return "".join(parts)

def gen_object_with_newlines(n: int, lines: int) -> str:
    """Generate object with newlines between members, matching Rust implementation."""
    parts = ["{"]
    for i in range(n):
        if i > 0:
            parts.append(",")
            parts.append("\n" * lines)
        parts.append(f"a{i}:{i}")
    parts.append("}")
    return "".join(parts)

def gen_mixed_comments(size: int) -> str:
    """Generate mixed comments corpus, matching Rust implementation."""
    parts: List[str] = []
    for i in range(size):
        parts.append("/*c*/[1,2,3] //x\n")
        parts.append(f"{{k{i}:{i}}}\n")
        parts.append("#y\n\n")
    return "".join(parts)

def corpuses() -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    # Align with Rust cases and keys used by aggregator
    out.append({"name":"typical","data":"{a:1, 'b': 'x', c: /re+/, d: 'he' + 'llo'}"})
    # Valid JSON baseline (must match benches and aggregator)
    out.append({"name":"valid_json","data":"{\"obj\":{\"a\":1,\"b\":2,\"arr\":[1,2,3],\"s\":\"hello\",\"nested\":{\"x\":true,\"y\":null}}}"})

    # Array with large spaces before commas (200 elements, matching Rust)
    for spaces in (64, 1024, 8192):
        out.append({"name":f"array_spaces_{spaces}", "data": gen_array_with_spaces(200, spaces)})

    # Object with many newlines between members (200 members, matching Rust)
    for lines in (1, 8, 64):
        out.append({"name":f"object_newlines_{lines}", "data": gen_object_with_newlines(200, lines)})

    # Mixed comments + whitespace corpus
    for rep in (50, 200):
        out.append({"name":f"mixed_comments_{rep}", "data": gen_mixed_comments(rep)})

    out.append({"name":"fence_jsonp","data":"cb(```json\n{a:1}\n```);"})
    out.append({"name":"unicode_comments","data":"{'中':/*c*/'文', note: '你' + '好'}"})
    out.append({"name":"ndjson_500","data":"".join([f"{{a:{i}}}\n" for i in range(500)])})
    return out

def main():
    ap = argparse.ArgumentParser(description="Benchmark python json_repair on representative corpuses.")
    ap.add_argument("--target-sec", type=float, default=1.0, help="Target seconds per case (steady-state)")
    ap.add_argument("--min-bytes", type=int, default=1<<20, help="Min bytes per iteration by scaling input (default 1 MiB)")
    ap.add_argument("--warmup", type=int, default=5, help="Warmup iterations per case")
    args = ap.parse_args()
    try:
        from json_repair.json_repair import repair_json as py_repair
    except ImportError:
        try:
            from json_repair import repair_json as py_repair
        except ImportError as e:
            raise SystemExit("Please `pip install json_repair` before running this script")

    results = []
    for item in corpuses():
        name = item["name"]; data = item["data"]
        base_size = len(data.encode("utf-8"))
        # scale input until at least min-bytes
        repeat = max(1, math.ceil(args.min_bytes / max(1, base_size)))
        data_scaled = data * repeat
        size = len(data_scaled.encode("utf-8"))
        # warmup
        for _ in range(args.warmup):
            py_repair(data_scaled)
        # calibrate
        c_iters = 3
        c0 = time.perf_counter()
        for _ in range(c_iters):
            py_repair(data_scaled)
        c1 = time.perf_counter()
        per_iter = max(1e-9, (c1 - c0) / c_iters)
        # steady state iters to meet target-sec
        iters = max(3, int(args.target_sec / per_iter))
        t0 = time.perf_counter()
        for _ in range(iters):
            py_repair(data_scaled)
        t1 = time.perf_counter()
        secs = (t1 - t0) / iters
        mibs = (size / secs) / (1024*1024)
        # logging
        print(f"[pybench] case={name} base_bytes={base_size} repeat={repeat} scaled_bytes={size} iters={iters} mean_s={secs:.6g} mib_s={mibs:.2f}", file=sys.stderr)
        results.append({
            "name": name,
            "size_bytes": size,
            "mean_sec_per_iter": secs,
            "throughput_mib_s": mibs,
            "iters": iters,
            "scaled_repeat": repeat,
        })

    print(json.dumps({"python_bench":results}, ensure_ascii=False, indent=2))

if __name__ == "__main__":
    main()
