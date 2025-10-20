# jsonrepair Python Tests

This directory contains comprehensive tests for the jsonrepair Python bindings.

## Test Files

### `test_basic.py`
Basic functionality tests covering:
- `repair_json()` with various parameters
- `loads()` function
- `load()` from file objects and StringIO
- `from_file()` from file paths
- Unicode handling
- Special characters
- Nested structures
- File I/O operations

### `test_compatibility.py`
Compatibility tests ported from the original [json_repair](https://github.com/mangiucugna/json_repair) library:
- Valid JSON handling
- Multiple JSON objects
- `return_objects` parameter
- `ensure_ascii` parameter
- String parsing with mixed quotes
- Object parsing edge cases
- Array parsing edge cases
- Comment handling (// and /* */)
- Number parsing
- Boolean and null parsing
- `loads()` function compatibility

### `test_edge_cases.py`
Edge case tests for robustness:
- String edge cases (embedded quotes, missing quotes)
- Markdown links in JSON
- Object edge cases (missing commas, trailing commas, empty keys)
- Unquoted keys and values
- Nested braces in strings
- Array edge cases (ellipsis, missing commas)
- Number edge cases (fractions, leading/trailing dots, scientific notation)
- Complex nested structures
- Fenced code blocks
- Unicode handling
- Empty and whitespace input

## Running Tests

### Run All Tests

```bash
cd python
pytest tests/ -v
```

### Run Specific Test File

```bash
pytest tests/test_basic.py -v
pytest tests/test_compatibility.py -v
pytest tests/test_edge_cases.py -v
```

### Run Specific Test Class

```bash
pytest tests/test_compatibility.py::TestParseString -v
pytest tests/test_edge_cases.py::TestNumberEdgeCases -v
```

### Run Specific Test

```bash
pytest tests/test_basic.py::test_loads -v
pytest tests/test_compatibility.py::TestParseComment::test_parse_comment -v
```

### Run with Coverage

```bash
pytest tests/ --cov=jsonrepair --cov-report=html
```

Then open `htmlcov/index.html` in your browser.

### Run with Verbose Output

```bash
pytest tests/ -vv
```

### Run Failed Tests Only

```bash
pytest tests/ --lf
```

## Test Coverage

The test suite covers:

### Core Functionality
- ✅ Basic JSON repair
- ✅ Parse to Python objects
- ✅ File I/O (load, from_file)
- ✅ String I/O (StringIO)

### JSON Issues
- ✅ Missing quotes (keys and values)
- ✅ Single quotes → double quotes
- ✅ Trailing commas
- ✅ Missing commas
- ✅ Comments (// and /* */)
- ✅ Unquoted keys
- ✅ Incomplete JSON
- ✅ Python keywords (True, False, None)
- ✅ JavaScript undefined
- ✅ Markdown fenced code blocks

### Edge Cases
- ✅ Embedded quotes in strings
- ✅ Nested braces in strings
- ✅ Empty keys
- ✅ Missing values
- ✅ Ellipsis in arrays
- ✅ Number edge cases (fractions, dots, scientific notation)
- ✅ Unicode characters
- ✅ HTML in JSON
- ✅ Complex nested structures

### Parameters
- ✅ `return_objects` - return Python objects instead of string
- ✅ `skip_json_loads` - skip validation
- ✅ `ensure_ascii` - escape non-ASCII characters

## Compatibility with json_repair

The tests in `test_compatibility.py` are ported from the original Python `json_repair` library to ensure our Rust implementation is compatible.

### Known Differences

Some tests may have slight differences due to implementation details:

1. **Empty input handling**: Different implementations may return different values for empty strings
2. **Whitespace normalization**: Output formatting may differ slightly
3. **Error messages**: Error messages are implementation-specific

### Skipped Tests

Some tests from the original library are not included because:

1. **`stream_stable` parameter**: Not yet implemented in our Rust version
2. **`logging` parameter**: Different logging mechanism in Rust
3. **CLI tests**: Not applicable to Python bindings

## Adding New Tests

When adding new tests:

1. **Choose the right file**:
   - `test_basic.py` - for basic functionality
   - `test_compatibility.py` - for json_repair compatibility
   - `test_edge_cases.py` - for edge cases and robustness

2. **Use descriptive names**:
   ```python
   def test_loads_with_trailing_comma():
       """Test loads function with trailing comma"""
       assert jsonrepair.loads("[1, 2, 3,]") == [1, 2, 3]
   ```

3. **Group related tests in classes**:
   ```python
   class TestUnicode:
       """Test Unicode handling"""
       
       def test_unicode_characters(self):
           # ...
       
       def test_ensure_ascii_with_unicode(self):
           # ...
   ```

4. **Add docstrings** to explain what the test does

5. **Use assertions** that clearly show what's expected

## Continuous Integration

These tests should be run in CI/CD pipelines to ensure:
- Compatibility across Python versions (3.8-3.14)
- Compatibility across platforms (Linux, macOS, Windows)
- No regressions when updating the Rust code

## Performance Tests

For performance benchmarking, see `examples/benchmark.py` which compares our implementation with the original Python `json_repair` library.

## Contributing

When contributing:
1. Add tests for new features
2. Ensure all tests pass: `pytest tests/ -v`
3. Check coverage: `pytest tests/ --cov=jsonrepair`
4. Run the benchmark: `python examples/benchmark.py`

## License

MIT OR Apache-2.0

