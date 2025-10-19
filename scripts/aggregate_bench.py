#!/usr/bin/env python3
import json
import sys
from pathlib import Path
import platform
import subprocess
import os
from datetime import datetime
from typing import Dict, Tuple, List

TARGET = Path("target/criterion/container_fast_paths")

def format_throughput(bytes_val: int, time_s: float) -> str:
    """
    Format throughput with adaptive units (B/s, KiB/s, or MiB/s).
    Returns a string like "1.23 MiB/s" or "456 KiB/s" or "789 B/s".
    """
    if time_s <= 0 or bytes_val <= 0:
        return "0 B/s"

    bytes_per_sec = bytes_val / time_s

    # Try MiB/s
    mib_s = bytes_per_sec / (1024 * 1024)
    if mib_s >= 1.0:
        return f"{mib_s:.2f} MiB/s"

    # Try KiB/s
    kib_s = bytes_per_sec / 1024
    if kib_s >= 1.0:
        return f"{kib_s:.2f} KiB/s"

    # Use B/s
    return f"{bytes_per_sec:.2f} B/s"

def gen_array_with_spaces(n: int, spaces: int) -> str:
    parts = ["["]
    for i in range(n):
        if i > 0:
            parts.append(" " * spaces)
            parts.append(",")
        parts.append(str(i))
    parts.append("]")
    return "".join(parts)

def gen_object_with_newlines(n: int, lines: int) -> str:
    parts = ["{"]
    for i in range(n):
        if i > 0:
            parts.append(",")
            parts.append("\n" * lines)
        parts.append(f"a{i}:{i}")
    parts.append("}")
    return "".join(parts)

def gen_mixed_comments(size: int) -> str:
    parts: List[str] = []
    for i in range(size):
        parts.append("/*c*/[1,2,3] //x\n")
        parts.append(f"{{k{i}:{i}}}\n")
        parts.append("#y\n\n")
    return "".join(parts)

def typical() -> str:
    return "{a:1, 'b': 'x', c: /re+/, d: 'he' + 'llo'}"

def fence_jsonp() -> str:
    return "cb(```json\n{a:1}\n```);"

def unicode_comments() -> str:
    return "{'汉':/*c*/'字', note: '你' + '好'}"

def ndjson_lines(n: int) -> str:
    return "".join([f"{{a:{i}}}\n" for i in range(n)])

def size_map() -> Dict[str, int]:
    m: Dict[str,int] = {}
    for spaces in (64, 1024, 8192):
        key = f"array_spaces/{spaces}"
        m[key] = len(gen_array_with_spaces(200, spaces).encode("utf-8"))
    for lines in (1, 8, 64):
        key = f"object_newlines/{lines}"
        m[key] = len(gen_object_with_newlines(200, lines).encode("utf-8"))
    for rep in (50, 200):
        key = f"mixed_comments/{rep}"
        m[key] = len(gen_mixed_comments(rep).encode("utf-8"))
    m["typical/fixed"] = len(typical().encode("utf-8"))
    m["fence_jsonp/fixed"] = len(fence_jsonp().encode("utf-8"))
    m["unicode_comments/fixed"] = len(unicode_comments().encode("utf-8"))
    m["ndjson/500"] = len(ndjson_lines(500).encode("utf-8"))
    # If JR_MIN_BYTES is set, report at least that many bytes (since benches scale input)
    try:
        min_bytes = int(os.environ.get("JR_MIN_BYTES", "0"))
    except Exception:
        min_bytes = 0
    if min_bytes > 0:
        for k in list(m.keys()):
            if m[k] < min_bytes:
                m[k] = min_bytes
    return m

def load_python(path: Path) -> Dict[str, Tuple[float,float,int]]:
    if not path.exists():
        return {}
    data = json.loads(path.read_text(encoding="utf-8"))
    out: Dict[str, Tuple[float,float,int]] = {}
    for item in data.get("python_bench", []):
        name = item["name"]
        out[name] = (
            float(item["mean_sec_per_iter"]),
            float(item["throughput_mib_s"]),
            int(item.get("size_bytes",0)),
        )
    return out

def scan_rust() -> Dict[str, float]:
    rust: Dict[str,float] = {}
    if not TARGET.exists():
        return rust
    for est in TARGET.rglob("new/estimates.json"):
        # path like .../container_fast_paths/<id>/<param>/new/estimates.json
        try:
            parts = est.parts
            idx = parts.index("container_fast_paths")
            bench_id = parts[idx+1]
            bench_param = parts[idx+2]
            key = f"{bench_id}/{bench_param}"
        except Exception:
            continue
        js = json.loads(est.read_text(encoding="utf-8"))
        mean_ns = js.get("mean",{}).get("point_estimate")
        if isinstance(mean_ns,(int,float)):
            rust[key] = float(mean_ns) / 1e9
    return rust

def main():
    py_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("python_bench.json")
    py = load_python(py_path)
    rust = scan_rust()
    sizes = size_map()

    # Environment header
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    host = platform.uname()
    try:
        rustc = subprocess.check_output(["rustc","--version"], text=True).strip()
    except Exception:
        rustc = "rustc --version: unavailable"
    try:
        pyver = subprocess.check_output([sys.executable,"--version"], text=True).strip()
    except Exception:
        pyver = f"python: {sys.version.splitlines()[0]}"

    jr_min = os.environ.get("JR_MIN_BYTES","(unset)")
    jr_meas = os.environ.get("JR_MEAS_SEC","(unset)")
    jr_warm = os.environ.get("JR_WARMUP_SEC","(unset)")
    jr_samp = os.environ.get("JR_SAMPLE_SIZE","(unset)")

    print(f"# Benchmark Report")
    print(f"- Timestamp: {ts}")
    print(f"- Host: {host.system} {host.release} ({host.machine}), CPU: {host.processor}")
    print(f"- Rust: {rustc}")
    print(f"- Python: {pyver}")
    print(f"- Rust bench env: JR_MIN_BYTES={jr_min}, JR_MEAS_SEC={jr_meas}, JR_WARMUP_SEC={jr_warm}, JR_SAMPLE_SIZE={jr_samp}")
    print(f"- Metrics: mean(s) (lower is better), throughput (higher is better)")
    print("")

    rows = []
    header = ["case","size(bytes)","rust mean(s)","rust throughput","python mean(s)","python throughput","speedup"]
    for key, mean_s in sorted(rust.items()):
        sz = sizes.get(key, 0)
        rust_thrpt = format_throughput(sz, mean_s)
        # Map python key names: convert `a/b` -> `a_b`, drop `/fixed`, and encode numbers
        parts = key.split("/")
        if len(parts) == 2 and parts[1] == "fixed":
            py_key = parts[0]
        elif len(parts) == 2:
            py_key = f"{parts[0]}_{parts[1]}"
        else:
            py_key = key.replace("/","_")
        py_row = py.get(py_key)
        if py_row:
            py_mean, py_mibs, _ = py_row
            # Convert Python MiB/s to adaptive format
            py_thrpt = format_throughput(sz, py_mean)
            # Calculate speedup (positive = Rust faster, negative = Python faster)
            speedup = py_mean / mean_s if mean_s > 0 else 0.0
            if speedup >= 1.0:
                speedup_str = f"Rust {speedup:.2f}x"
            else:
                speedup_str = f"Python {1/speedup:.2f}x" if speedup > 0 else "-"
            rows.append([key, sz, f"{mean_s:.6g}", rust_thrpt, f"{py_mean:.6g}", py_thrpt, speedup_str])
        else:
            rows.append([key, sz, f"{mean_s:.6g}", rust_thrpt, "-", "-", "-"])

    # Print markdown table
    print("| "+" | ".join(header)+" |")
    print("|"+"---|"*len(header))
    for r in rows:
        print("| "+" | ".join(map(str,r))+" |")

if __name__ == "__main__":
    main()
