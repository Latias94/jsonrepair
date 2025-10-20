#!/usr/bin/env python3
import json
import sys
from pathlib import Path
import platform
import subprocess
import os
from datetime import datetime
from typing import Dict, Tuple, List

CRIT_ROOT = Path(os.environ.get("CARGO_TARGET_DIR", "target")) / "criterion"

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

def valid_json() -> str:
    return '{"obj":{"a":1,"b":2,"arr":[1,2,3],"s":"hello","nested":{"x":true,"y":null}}}'

def valid_json() -> str:
    return '{"obj":{"a":1,"b":2,"arr":[1,2,3],"s":"hello","nested":{"x":true,"y":null}}}'

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
    m["valid_json/fixed"] = len(valid_json().encode("utf-8"))
    m["valid_json_ensure_ascii/fixed"] = m["valid_json/fixed"]
    m["valid_json_fastpath/fixed"] = m["valid_json/fixed"]
    m["valid_json_strict/fixed"] = m["valid_json/fixed"]
    m["fence_jsonp/fixed"] = len(fence_jsonp().encode("utf-8"))
    m["unicode_comments/fixed"] = len(unicode_comments().encode("utf-8"))
    m["ndjson/500"] = len(ndjson_lines(500).encode("utf-8"))
    # Additional realistic corpora
    m["flat_object/10000"] = len(("{" + ",".join([f"k{i}:{i}" for i in range(10000)]) + "}").encode("utf-8"))
    m["array_dense/100000"] = len(("[" + ",".join([str(i) for i in range(100000)]) + "]").encode("utf-8"))
    def nested(depth: int) -> str:
        return ("{" + "a:" ) * depth + "{x:1}" + ("}" * depth)
    m["nested_object/16"] = len(nested(16).encode("utf-8"))
    m["strings_unicode/1000"] = len("".join([f"{{text: '你' + '好', i:{i}}} \n" for i in range(1000)]).encode("utf-8"))
    # Trailing commas corpus (small fixed)
    m["trailing_commas/fixed"] = len(("{a:1,b:2,c:3,}\n[1,2,3,]\n").encode("utf-8"))
    # Stream bench sizes
    m["stream/ndjson_1000_lines"] = len(ndjson_lines(1000).encode("utf-8"))
    # Writer bench sizes (approximate corpus used by writer_bench.rs)
    def gen_large_object(n: int) -> str:
        parts: List[str] = ["{"]
        for i in range(n):
            if i > 0:
                parts.append(",")
            parts.append(f"k{i}:{{a:[{i}, {i+1}, {i+2}], s:'he'+ 'llo', r:/a\\/b/}}")
        parts.append("}")
        return "".join(parts)
    m["writer_vs_string/to_string"] = len(gen_large_object(5000).encode("utf-8"))
    m["writer_vs_string/to_writer_streaming"] = m["writer_vs_string/to_string"]
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
    if not CRIT_ROOT.exists():
        return rust
    groups = {"container_fast_paths", "container_llm", "stream", "writer_vs_string"}
    for est in CRIT_ROOT.rglob("new/estimates.json"):
        parts = list(est.parts)
        if "criterion" not in parts:
            continue
        try:
            cidx = parts.index("criterion")
            group = parts[cidx+1]
            if group not in groups:
                continue
            # Normalize keys and label engine
            if group in ("container_fast_paths", "container_llm"):
                bench_id = parts[cidx+2]
                bench_param = parts[cidx+3]
                case_key = f"{bench_id}/{bench_param}"
                engine = "jsonrepair" if group == "container_fast_paths" else "llm_json"
            elif group == "stream":
                bench_param = parts[cidx+2]
                case_key = f"{group}/{bench_param}"
                engine = "jsonrepair(stream)"
            else:
                bench_param = parts[cidx+2]
                case_key = f"{group}/{bench_param}"
                engine = "jsonrepair(writer)"
            js = json.loads(est.read_text(encoding="utf-8"))
            mean_ns = js.get("mean",{}).get("point_estimate")
            if isinstance(mean_ns,(int,float)):
                rust[f"{engine}||{case_key}"] = float(mean_ns) / 1e9
        except Exception:
            continue
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

    def py_key_for(case_key: str) -> str:
        parts = case_key.split("/")
        if len(parts) == 2 and parts[1] == "fixed":
            return parts[0]
        elif len(parts) == 2:
            return f"{parts[0]}_{parts[1]}"
        else:
            return case_key.replace("/","_")

    def fmt(mean_s: float, size: int) -> str:
        return f"{mean_s:.6g} ({format_throughput(size, mean_s)})"

    cases: Dict[str, dict] = {}

    def add_case(engine_key: str, key: str, mean_s: float):
        d = cases.setdefault(key, {})
        d[engine_key] = mean_s

    for full_key, mean_s in sorted(rust.items()):
        engine, key = full_key.split("||", 1)
        if engine == "jsonrepair":
            if key.startswith("valid_json_fastpath/"):
                add_case("jr_fast", key, mean_s)
            else:
                add_case("jr_strict", key, mean_s)
        elif engine == "llm_json":
            if key.startswith("valid_json_strict/"):
                key = key.replace("valid_json_strict/", "valid_json/")
                add_case("llm_strict", key, mean_s)
            elif key.startswith("valid_json_fastpath/"):
                add_case("llm_fast", key, mean_s)
            else:
                add_case("llm_strict", key, mean_s)
        elif engine == "jsonrepair(stream)":
            add_case("jr_stream", key, mean_s)
        elif engine == "jsonrepair(writer)":
            add_case("jr_writer", key, mean_s)
        else:
            pass

    # Container cases (non-streaming) — strict only (fastpaths moved to the dedicated valid JSON table)
    print("## Container Cases (strict)")
    header = ["case","size(bytes)","python","jsonrepair(strict)","llm_json(strict)"]
    print("| "+" | ".join(header)+" |")
    print("|"+"---|"*len(header))
    for key in sorted(k for k in cases.keys()
                      if not k.startswith("stream/")
                      and not k.startswith("writer_vs_string/")
                      and not k.startswith("valid_json")
                      and not k.startswith("valid_json_ensure_ascii")
                      and not k.startswith("valid_json_fastpath")):
        sz = sizes.get(key, 0)
        py_row = py.get(py_key_for(key))
        py_cell = fmt(py_row[0], sz) if py_row else "-"
        row = cases.get(key, {})
        jr_s = fmt(row["jr_strict"], sz) if "jr_strict" in row else "-"
        llm_s = fmt(row["llm_strict"], sz) if "llm_strict" in row else "-"
        print(f"| {key} | {sz} | {py_cell} | {jr_s} | {llm_s} |")

    # Valid JSON dedicated section
    print("")
    print("## Valid JSON Cases (strict vs fastpath)")
    header = ["case","size(bytes)","jsonrepair(strict)","jsonrepair(fast)","llm_json(strict)","llm_json(fast)"]
    print("| "+" | ".join(header)+" |")
    print("|"+"---|"*len(header))
    for key in ["valid_json/fixed", "valid_json_ensure_ascii/fixed", "valid_json_fastpath/fixed"]:
        if key not in cases: continue
        sz = sizes.get(key, 0)
        row = cases.get(key, {})
        jr_s = fmt(row.get("jr_strict", 0.0), sz) if "jr_strict" in row else "-"
        jr_f = fmt(row.get("jr_fast", 0.0), sz) if "jr_fast" in row else "-"
        llm_s = fmt(row.get("llm_strict", 0.0), sz) if "llm_strict" in row else "-"
        llm_f = fmt(row.get("llm_fast", 0.0), sz) if "llm_fast" in row else "-"
        print(f"| {key} | {sz} | {jr_s} | {jr_f} | {llm_s} | {llm_f} |")
    print("")
    stream_keys = sorted(k for k in cases.keys() if k.startswith("stream/"))
    if stream_keys:
        print("## Stream Cases")
        header = ["case","size(bytes)","jsonrepair(stream)"]
        print("| "+" | ".join(header)+" |")
        print("|"+"---|"*len(header))
        for key in stream_keys:
            sz = sizes.get(key, 0)
            row = cases.get(key, {})
            jr_stream = fmt(row["jr_stream"], sz) if "jr_stream" in row else "-"
            print(f"| {key} | {sz} | {jr_stream} |")

    print("")
    writer_keys = sorted(k for k in cases.keys() if k.startswith("writer_vs_string/"))
    if writer_keys:
        print("## Writer Cases")
        header = ["case","size(bytes)","jsonrepair(writer)"]
        print("| "+" | ".join(header)+" |")
        print("|"+"---|"*len(header))
        for key in writer_keys:
            sz = sizes.get(key, 0)
            row = cases.get(key, {})
            jr_w = fmt(row["jr_writer"], sz) if "jr_writer" in row else "-"
            print(f"| {key} | {sz} | {jr_w} |")

if __name__ == "__main__":
    main()

