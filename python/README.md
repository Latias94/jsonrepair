# jsonrepair - Fast JSON Repair for Python

A high-performance JSON repair library for Python, powered by Rust. This library provides a drop-in replacement for the popular [json_repair](https://github.com/mangiucugna/json_repair) library with **significantly better performance**.

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

### `repair_json(json_str, return_objects=False, skip_json_loads=False, ensure_ascii=False, **kwargs)`

Repair a broken JSON string.

**Parameters:**
- `json_str` (str): The broken JSON string to repair
- `return_objects` (bool): If True, return parsed Python objects instead of string
- `skip_json_loads` (bool): If True, skip validation (faster but less safe)
- `ensure_ascii` (bool): If True, escape non-ASCII characters

**Returns:**
- str or dict/list: Repaired JSON string, or parsed object if `return_objects=True`

**Example:**
```python
>>> repair_json("{name: 'John'}")
'{"name":"John"}'

>>> repair_json("{name: 'John'}", return_objects=True)
{'name': 'John'}
```

### `loads(json_str, skip_json_loads=False, ensure_ascii=False)`

Repair and parse a JSON string (like `json.loads`).

**Parameters:**
- `json_str` (str): The broken JSON string to repair and parse
- `skip_json_loads` (bool): If True, skip validation
- `ensure_ascii` (bool): If True, escape non-ASCII characters

**Returns:**
- dict or list: Parsed Python object

**Example:**
```python
>>> loads("{name: 'John', age: 30}")
{'name': 'John', 'age': 30}
```

### `load(fp, skip_json_loads=False, ensure_ascii=False)`

Repair and parse JSON from a file object (like `json.load`).

**Parameters:**
- `fp`: A file-like object with a `.read()` method
- `skip_json_loads` (bool): If True, skip validation
- `ensure_ascii` (bool): If True, escape non-ASCII characters

**Returns:**
- dict or list: Parsed Python object

**Example:**
```python
>>> with open('broken.json') as f:
...     data = load(f)
```

### `from_file(filename, skip_json_loads=False, ensure_ascii=False)`

Repair and parse JSON from a file path.

**Parameters:**
- `filename` (str): Path to the JSON file
- `skip_json_loads` (bool): If True, skip validation
- `ensure_ascii` (bool): If True, escape non-ASCII characters

**Returns:**
- dict or list: Parsed Python object

**Example:**
```python
>>> data = from_file('broken.json')
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

jsonrepair provides **significant performance improvements** over pure Python implementations, especially for:

- **Simple repairs** (quotes, commas): 50-200x faster
- **Comment removal**: 20-100x faster
- **Fenced code blocks**: 50-400x faster
- **Complex nested structures**: 10-50x faster

Performance varies by use case. For detailed benchmarks, see [bench_table.md](../docs/bench_table.md).

**Quick benchmark:**

```python
import timeit
import jsonrepair
import json_repair  # Pure Python implementation

broken = "{name: 'John', age: 30,}" * 100

# jsonrepair (Rust)
time_rust = timeit.timeit(lambda: jsonrepair.loads(broken), number=1000)

# json_repair (Python)
time_python = timeit.timeit(lambda: json_repair.loads(broken), number=1000)

print(f"Speedup: {time_python / time_rust:.1f}x")
```

## 🤝 Compatibility

This library aims to be 100% compatible with [json_repair](https://github.com/mangiucugna/json_repair).

**Compatibility Testing:**
- We've ported the test suite from the original `json_repair` library
- 100+ compatibility tests ensure API compatibility
- See `tests/test_compatibility.py` for details

If you find any incompatibilities, please [open an issue](https://github.com/Latias94/jsonrepair/issues).

## 📄 License

MIT OR Apache-2.0

## 🙏 Credits

- Inspired by [json_repair](https://github.com/mangiucugna/json_repair) by Stefano Baccianella
- Powered by [PyO3](https://github.com/PyO3/pyo3) for Rust-Python bindings
