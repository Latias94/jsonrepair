"""
Basic tests for jsonrepair Python bindings

These tests cover the basic functionality and API surface.
"""

import pytest
import jsonrepair
import tempfile
import os
import io


def test_repair_json_basic():
    """Test basic JSON repair returning string"""
    broken = "{name: 'John', age: 30,}"
    repaired = jsonrepair.repair_json(broken)
    assert repaired == '{"name":"John","age":30}'


def test_repair_json_return_objects():
    """Test JSON repair returning Python objects"""
    broken = "{name: 'John', age: 30}"
    result = jsonrepair.repair_json(broken, return_objects=True)
    assert isinstance(result, dict)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_loads():
    """Test loads function (like json.loads)"""
    broken = "{name: 'John', age: 30}"
    result = jsonrepair.loads(broken)
    assert isinstance(result, dict)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_loads_array():
    """Test loads with array"""
    broken = "[1, 2, 3,]"
    result = jsonrepair.loads(broken)
    assert result == [1, 2, 3]


def test_loads_with_comments():
    """Test loads with comments"""
    broken = """
    {
        // This is a comment
        "name": "John",
        /* Block comment */
        "age": 30
    }
    """
    result = jsonrepair.loads(broken)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_loads_single_quotes():
    """Test loads with single quotes"""
    broken = "{'name': 'John', 'active': true}"
    result = jsonrepair.loads(broken)
    assert result['name'] == 'John'
    assert result['active'] is True


def test_loads_unquoted_keys():
    """Test loads with unquoted keys"""
    broken = "{name: 'John', age: 30}"
    result = jsonrepair.loads(broken)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_loads_trailing_comma():
    """Test loads with trailing comma"""
    broken = '{"name": "John", "age": 30,}'
    result = jsonrepair.loads(broken)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_loads_missing_quotes():
    """Test loads with missing quotes"""
    broken = '{name: John, age: 30}'
    result = jsonrepair.loads(broken)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_loads_python_keywords():
    """Test loads with Python keywords (True, False, None)"""
    broken = '{active: True, inactive: False, value: None}'
    result = jsonrepair.loads(broken)
    assert result['active'] is True
    assert result['inactive'] is False
    assert result['value'] is None


def test_loads_undefined():
    """Test loads with undefined (converts to null)"""
    broken = '{value: undefined}'
    result = jsonrepair.loads(broken)
    assert result['value'] is None


def test_loads_fenced_code_block():
    """Test loads with markdown fenced code block"""
    broken = """
    ```json
    {
        "name": "John",
        "age": 30
    }
    ```
    """
    result = jsonrepair.loads(broken)
    # Our library extracts the JSON from fenced code blocks
    assert isinstance(result, dict)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_loads_incomplete_json():
    """Test loads with incomplete JSON"""
    broken = '{"name": "John", "age": 30'
    result = jsonrepair.loads(broken)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_loads_unicode():
    """Test loads with Unicode characters"""
    broken = "{'name': '统一码', 'emoji': '😀'}"
    result = jsonrepair.loads(broken)
    assert result['name'] == '统一码'
    assert result['emoji'] == '😀'


def test_ensure_ascii():
    """Test ensure_ascii option"""
    broken = "{'name': '统一码'}"
    repaired = jsonrepair.repair_json(broken, ensure_ascii=True)
    # Should escape Unicode characters
    assert '\\u' in repaired


def test_from_file():
    """Test from_file function"""
    # Create a temporary file with broken JSON
    with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.json') as f:
        f.write("{name: 'John', age: 30}")
        temp_path = f.name
    
    try:
        result = jsonrepair.from_file(temp_path)
        assert result['name'] == 'John'
        assert result['age'] == 30
    finally:
        os.unlink(temp_path)


def test_load():
    """Test load function with file object"""
    # Create a temporary file with broken JSON
    with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.json') as f:
        f.write("{name: 'John', age: 30}")
        temp_path = f.name

    try:
        with open(temp_path, 'r') as f:
            result = jsonrepair.load(f)
        assert result['name'] == 'John'
        assert result['age'] == 30
    finally:
        os.unlink(temp_path)


def test_load_from_stringio():
    """Test load function with StringIO"""
    broken = "{name: 'John', age: 30}"
    string_io = io.StringIO(broken)
    result = jsonrepair.load(string_io)
    assert result['name'] == 'John'
    assert result['age'] == 30


def test_load_from_bytesio():
    """Test load function with BytesIO (should fail gracefully)"""
    broken = b"{name: 'John', age: 30}"
    bytes_io = io.BytesIO(broken)
    # This should fail because we expect text mode
    with pytest.raises(Exception):
        jsonrepair.load(bytes_io)


def test_nested_objects():
    """Test with nested objects"""
    broken = """
    {
        name: 'John',
        address: {
            street: '123 Main St',
            city: 'New York'
        },
        hobbies: ['reading', 'swimming']
    }
    """
    result = jsonrepair.loads(broken)
    assert result['name'] == 'John'
    assert result['address']['street'] == '123 Main St'
    assert result['address']['city'] == 'New York'
    assert result['hobbies'] == ['reading', 'swimming']


def test_empty_input():
    """Test with empty input"""
    result = jsonrepair.repair_json("")
    # Empty input returns empty string (like json_repair)
    assert result == ""


def test_numbers():
    """Test with various number formats"""
    broken = '{int: 42, float: 3.14, negative: -10, exp: 1e5}'
    result = jsonrepair.loads(broken)
    assert result['int'] == 42
    assert result['float'] == 3.14
    assert result['negative'] == -10
    assert result['exp'] == 1e5


def test_special_characters():
    """Test with special characters in strings"""
    broken = r'{"message": "He said \"Hello\""}'
    result = jsonrepair.loads(broken)
    assert result['message'] == 'He said "Hello"'


def test_mixed_quotes():
    """Test with mixed quotes"""
    broken = """{"name": 'John', 'age': "30"}"""
    result = jsonrepair.loads(broken)
    assert result['name'] == 'John'
    assert result['age'] == '30'


if __name__ == '__main__':
    pytest.main([__file__, '-v'])

