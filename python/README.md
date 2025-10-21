# jsonrepair - Fast JSON Repair for Python

A high-performance JSON repair library for Python, powered by Rust. This library provides a drop-in replacement for the popular [json_repair](https://github.com/mangiucugna/json_repair) library with better performance for most use cases.

## 🚀 Features

- **⚡ High Performance**: Rust-powered implementation, significantly faster than pure Python
- **🔄 Drop-in Replacement**: Compatible API with `json_repair`
- **🛠️ Comprehensive Repair**: Fixes common JSON issues:
  - Missing quotes around keys and values
  - Single quotes instead of double quotes
  - Trailing commas
  - Comments (// and /* */)
  - Unquoted keys
  - Missing commas between elements
  - Incomplete JSON (missing closing brackets)
  - Python keywords (True, False, None)
  - JavaScript undefined
  - Markdown fenced code blocks
- **🌍 Unicode Support**: Properly handles Unicode characters
- **📦 Zero Python Dependencies**: Pure Rust implementation

## 📦 Installation

```bash
pip install jsonrepair
```

Or install from source:

```bash
cd python
pip install maturin
maturin develop
```

## 🎯 Quick Start

### Basic Usage

```python
import jsonrepair

# Repair and parse broken JSON
broken_json = "{name: 'John', age: 30,}"
data = jsonrepair.loads(broken_json)
print(data)  # {'name': 'John', 'age': 30}

# Just repair, return string
repaired = jsonrepair.repair_json(broken_json)
print(repaired)  # '{"name":"John","age":30}'
```

### Drop-in Replacement for json_repair

```python
# Replace this:
# from json_repair import repair_json, loads, load, from_file

# With this:
from jsonrepair import repair_json, loads, load, from_file

# All functions work the same way!
data = loads("{name: 'John', age: 30}")
```

### Load from File

```python
import jsonrepair

# From file path
data = jsonrepair.from_file('broken.json')

# From file object
with open('broken.json') as f:
    data = jsonrepair.load(f)
```

## 📚 API Reference

### `RepairOptions`

Advanced configuration class for fine-tuning repair behavior.

**Example:**
```python
# Create with constructor
opts = jsonrepair.RepairOptions(
    ensure_ascii=True,
    allow_python_keywords=True,
    tolerate_hash_comments=False
)

# Or modify using properties
opts = jsonrepair.RepairOptions()
opts.ensure_ascii = True
opts.logging = True

# Use with any function
result = jsonrepair.loads("{a: 1}", options=opts)
```

**Available properties:**
- `ensure_ascii` (bool): Escape non-ASCII characters
- `tolerate_hash_comments` (bool): Allow # comments
- `repair_undefined` (bool): Convert undefined → null
- `allow_python_keywords` (bool): Convert True/False/None
- `fenced_code_blocks` (bool): Extract from ```...```
- `normalize_js_nonfinite` (bool): Convert NaN/Infinity → null
- `stream_ndjson_aggregate` (bool): Aggregate NDJSON to array
- `logging` (bool): Enable repair logging

See [examples/options_example.py](examples/options_example.py) for more examples.

### `repair_json(json_str, return_objects=False, skip_json_loads=False, ensure_ascii=False, options=None)`

Repair a broken JSON string.

**Parameters:**
- `json_str` (str): The broken JSON string to repair
- `return_objects` (bool): If True, return parsed Python objects instead of string
- `skip_json_loads` (bool): If True, skip validation (faster but less safe)
- `ensure_ascii` (bool): If True, escape non-ASCII characters
- `options` (RepairOptions): Optional advanced configuration

**Returns:**
- str or dict/list: Repaired JSON string, or parsed object if `return_objects=True`

**Example:**
```python
>>> repair_json("{name: 'John'}")
'{"name":"John"}'

>>> repair_json("{name: 'John'}", return_objects=True)
{'name': 'John'}

>>> opts = RepairOptions(ensure_ascii=True)
>>> repair_json("{'name': '中文'}", options=opts)
'{"name":"\\u4e2d\\u6587"}'
```

### `loads(json_str, skip_json_loads=False, ensure_ascii=False, options=None)`

Repair and parse a JSON string (like `json.loads`).

**Parameters:**
- `json_str` (str): The broken JSON string to repair and parse
- `skip_json_loads` (bool): If True, skip validation
- `ensure_ascii` (bool): If True, escape non-ASCII characters
- `options` (RepairOptions): Optional advanced configuration

**Returns:**
- dict or list: Parsed Python object

**Example:**
```python
>>> loads("{name: 'John', age: 30}")
{'name': 'John', 'age': 30}
```

### `load(fp, skip_json_loads=False, ensure_ascii=False, options=None)`

Repair and parse JSON from a file object (like `json.load`).

**Parameters:**
- `fp`: A file-like object with a `.read()` method
- `skip_json_loads` (bool): If True, skip validation
- `ensure_ascii` (bool): If True, escape non-ASCII characters
- `options` (RepairOptions): Optional advanced configuration

**Returns:**
- dict or list: Parsed Python object

**Example:**
```python
>>> with open('broken.json') as f:
...     data = load(f)
```

### `from_file(filename, skip_json_loads=False, ensure_ascii=False, options=None)`

Repair and parse JSON from a file path.

**Parameters:**
- `filename` (str): Path to the JSON file
- `skip_json_loads` (bool): If True, skip validation
- `ensure_ascii` (bool): If True, escape non-ASCII characters
- `options` (RepairOptions): Optional advanced configuration

**Returns:**
- dict or list: Parsed Python object

**Example:**
```python
>>> data = from_file('broken.json')
```

### `StreamRepairer(options=None)`

Streaming JSON repairer for processing large files or chunked input.

**Parameters:**
- `options` (RepairOptions): Optional advanced configuration

**Methods:**
- `push(chunk: str) -> Optional[str]`: Feed a chunk, returns output when complete
- `flush() -> Optional[str]`: Flush remaining buffered output

**Example:**
```python
>>> repairer = StreamRepairer()
>>> for chunk in chunks:
...     if output := repairer.push(chunk):
...         process(output)
>>> if final := repairer.flush():
...     process(final)
```

## 🔧 Supported Repairs

### Missing Quotes

```python
>>> loads("{name: John, age: 30}")
{'name': 'John', 'age': 30}
```

### Single Quotes

```python
>>> loads("{'name': 'John'}")
{'name': 'John'}
```

### Trailing Commas

```python
>>> loads('{"name": "John", "age": 30,}')
{'name': 'John', 'age': 30}
```

### Comments

```python
>>> loads('''
... {
...     // This is a comment
...     "name": "John",
...     /* Block comment */
...     "age": 30
... }
... ''')
{'name': 'John', 'age': 30}
```

### Python Keywords

```python
>>> loads('{active: True, inactive: False, value: None}')
{'active': True, 'inactive': False, 'value': None}
```

### Incomplete JSON

```python
>>> loads('{"name": "John", "age": 30')
{'name': 'John', 'age': 30}
```

### Markdown Fenced Code Blocks

```python
>>> loads('''
... ```json
... {"name": "John"}
... ```
... ''')
{'name': 'John'}
```

## ⚡ Performance

This library provides better performance than pure Python implementations for most use cases, especially for:

- **Medium to large JSON** (1KB - 1MB+): Our zero-copy architecture shines here
- **Batch processing**: Processing many files or API responses
- **Complex repairs**: Multiple repair operations on the same input

### Architecture

- **Zero-copy parsing**: Hand-written recursive descent parser using `&str` slicing
- **Minimal allocations**: Reuses input string where possible
- **Native code**: Compiled Rust for CPU-intensive parsing

### When to Use This Library

✅ **Good fit:**
- Processing LLM outputs (medium-large JSON with various issues)
- API response repair (1KB - 1MB range)
- Batch file processing
- Production systems needing consistent performance

⚠️ **Consider alternatives:**
- **Very small JSON** (< 100 bytes): Pure Python overhead may be negligible
- **Already valid JSON**: Use standard `json.loads()` instead
- **Simple string fixes**: Regex might be faster for single-issue repairs

### Benchmark It Yourself

We provide a benchmark script to compare with `json_repair`:

```bash
# Install both libraries
pip install jsonrepair json-repair

# Run benchmark
python python/examples/benchmark.py
```

**Quick test:**

```python
import timeit
import jsonrepair
import json_repair

# Test with your actual data
broken = "{name: 'John', age: 30,}" * 100

time_rust = timeit.timeit(lambda: jsonrepair.loads(broken), number=1000)
time_python = timeit.timeit(lambda: json_repair.loads(broken), number=1000)

print(f"jsonrepair: {time_rust:.4f}s")
print(f"json_repair: {time_python:.4f}s")
print(f"Speedup: {time_python / time_rust:.1f}x")
```

**Note:** Performance varies significantly by input size, repair complexity, and system. Always benchmark with your actual data.

## 🤝 Compatibility

This library aims to be 100% compatible with [json_repair](https://github.com/mangiucugna/json_repair).

**Compatibility Testing:**
- We've ported the test suite from the original `json_repair` library
- 100+ compatibility tests ensure API compatibility
- See `tests/test_compatibility.py` for details

**Known Differences:**
- Performance characteristics differ (Rust vs Python)
- Error messages may vary slightly
- Internal implementation is completely different

If you find any functional incompatibilities, please [open an issue](https://github.com/Latias94/jsonrepair/issues).

## 🎯 Choosing the Right Library

### Use `jsonrepair` (this library) if:
- You process medium to large JSON files (1KB+)
- You need consistent performance in production
- You're doing batch processing or high-throughput repairs
- You want additional features (streaming API, logging, type hints)

### Use `json_repair` (pure Python) if:
- You need a pure Python solution (no compiled dependencies)
- You're processing very small JSON snippets (< 100 bytes)
- You need maximum portability across platforms
- You prefer simpler installation (no Rust toolchain needed for development)

Both libraries are excellent—choose based on your specific needs. We're grateful to the `json_repair` project for the inspiration and API design.

## 📄 License

MIT OR Apache-2.0

## � Related Projects

- **[json_repair](https://github.com/mangiucugna/json_repair)** - The original pure Python implementation that inspired this project
- **[llm_json](https://github.com/oramasearch/llm_json)** - Alternative Rust-based library, also based on json_repair
- **[PyO3](https://github.com/PyO3/pyo3)** - The Rust-Python binding framework powering this library

## 🙏 Credits

This project is inspired by and aims to be compatible with [json_repair](https://github.com/mangiucugna/json_repair) by Stefano Baccianella. We're grateful for the excellent API design and comprehensive test suite that made this Rust implementation possible.
