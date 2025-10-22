"""
Compatibility tests ported from json_repair library
https://github.com/mangiucugna/json_repair

These tests ensure our Rust implementation is compatible with the Python json_repair library.
"""

import json

import jsonrepair
import pytest


def assert_json_equal(actual: str, expected: str):
    """Compare two JSON strings by parsing them, ignoring formatting differences."""
    try:
        actual_parsed = json.loads(actual)
        expected_parsed = json.loads(expected)
        assert actual_parsed == expected_parsed, f"JSON content differs:\nActual: {actual}\nExpected: {expected}"
    except json.JSONDecodeError:
        # If parsing fails, fall back to string comparison
        assert actual == expected


class TestValidJSON:
    """Test with valid JSON strings"""

    def test_valid_json(self):
        assert_json_equal(
            jsonrepair.repair_json('{"name": "John", "age": 30, "city": "New York"}'),
            '{"name": "John", "age": 30, "city": "New York"}'
        )
        assert_json_equal(
            jsonrepair.repair_json('{"employees":["John", "Anna", "Peter"]} '),
            '{"employees": ["John", "Anna", "Peter"]}'
        )
        assert_json_equal(
            jsonrepair.repair_json('{"key": "value:value"}'),
            '{"key": "value:value"}'
        )
        assert_json_equal(
            jsonrepair.repair_json('{"text": "The quick brown fox,"}'),
            '{"text": "The quick brown fox,"}'
        )
        assert_json_equal(
            jsonrepair.repair_json('{"text": "The quick brown fox won\'t jump"}'),
            '{"text": "The quick brown fox won\'t jump"}'
        )
        assert_json_equal(jsonrepair.repair_json('{"key": ""'), '{"key": ""}')
        assert_json_equal(jsonrepair.repair_json('{"key1": {"key2": [1, 2, 3]}}'), '{"key1": {"key2": [1, 2, 3]}}')
        assert_json_equal(jsonrepair.repair_json('{"key": 12345678901234567890}'), '{"key": 12345678901234567890}')


class TestMultipleJSONs:
    """Test handling multiple JSON objects"""

    def test_multiple_jsons(self):
        assert_json_equal(jsonrepair.repair_json("[]{}"), "[[], {}]")
        assert_json_equal(jsonrepair.repair_json("{}[]{}"), "[{}, [], {}]")
        assert_json_equal(jsonrepair.repair_json('{"key":"value"}[1,2,3,true]'), '[{"key": "value"}, [1, 2, 3, true]]')


class TestReturnObjects:
    """Test return_objects parameter"""

    def test_repair_json_with_objects(self):
        # Test with valid JSON strings
        assert jsonrepair.repair_json("[]", return_objects=True) == []
        assert jsonrepair.repair_json("{}", return_objects=True) == {}
        assert jsonrepair.repair_json('{"key": true, "key2": false, "key3": null}', return_objects=True) == {
            "key": True,
            "key2": False,
            "key3": None,
        }
        assert jsonrepair.repair_json('{"name": "John", "age": 30, "city": "New York"}', return_objects=True) == {
            "name": "John",
            "age": 30,
            "city": "New York",
        }
        assert jsonrepair.repair_json("[1, 2, 3, 4]", return_objects=True) == [1, 2, 3, 4]
        assert jsonrepair.repair_json('{"employees":["John", "Anna", "Peter"]} ', return_objects=True) == {
            "employees": ["John", "Anna", "Peter"]
        }


class TestEnsureASCII:
    """Test ensure_ascii parameter"""

    def test_ensure_ascii(self):
        result = jsonrepair.repair_json("{'test_中国人_ascii':'统一码'}", ensure_ascii=True)
        # Should escape Unicode characters
        assert '\\u' in result


class TestParseString:
    """Test string parsing"""

    def test_missing_and_mixed_quotes(self):
        assert_json_equal(
            jsonrepair.repair_json("{'key': 'string', 'key2': false, \"key3\": null, \"key4\": unquoted}"),
            '{"key": "string", "key2": false, "key3": null, "key4": "unquoted"}'
        )
        assert_json_equal(jsonrepair.repair_json('{"name": "John", "age": 30, "city": "New York'), '{"name": "John", "age": 30, "city": "New York"}')
        assert_json_equal(jsonrepair.repair_json('{"name": "John", "age": 30, city: "New York"}'), '{"name": "John", "age": 30, "city": "New York"}')
        assert_json_equal(jsonrepair.repair_json('{"name": "John", "age": 30, "city": New York}'), '{"name": "John", "age": 30, "city": "New York"}')
        assert_json_equal(jsonrepair.repair_json('{"name": John, "age": 30, "city": "New York"}'), '{"name": "John", "age": 30, "city": "New York"}')

    def test_leading_trailing_characters(self):
        assert_json_equal(jsonrepair.repair_json('````{ "key": "value" }```'), '{"key": "value"}')
        assert_json_equal(jsonrepair.repair_json("""{    "a": "",    "b": [ { "c": 1} ] \n}```"""), '{"a": "", "b": [{"c": 1}]}')
        assert_json_equal(jsonrepair.repair_json("Based on the information extracted, here is the filled JSON output: ```json { 'a': 'b' } ```"), '{"a": "b"}')


class TestParseObject:
    """Test object parsing"""

    def test_parse_object(self):
        assert jsonrepair.repair_json("{}", return_objects=True) == {}
        assert jsonrepair.repair_json('{ "key": "value", "key2": 1, "key3": true }', return_objects=True) == {
            "key": "value",
            "key2": 1,
            "key3": True,
        }
        assert jsonrepair.repair_json("{", return_objects=True) == {}
        assert_json_equal(jsonrepair.repair_json("   {  }   "), "{}")
        assert_json_equal(jsonrepair.repair_json("{"), "{}")

    def test_parse_object_edge_cases(self):
        assert_json_equal(jsonrepair.repair_json("{foo: [}"), '{"foo": []}')
        assert_json_equal(jsonrepair.repair_json('{"": "value"'), '{"": "value"}')
        assert_json_equal(jsonrepair.repair_json('{"" : true, "key2": "value2"}'), '{"": true, "key2": "value2"}')
        assert_json_equal(jsonrepair.repair_json("{key:value,key2:value2}"), '{"key": "value", "key2": "value2"}')


class TestParseArray:
    """Test array parsing"""

    def test_parse_array(self):
        assert jsonrepair.repair_json("[]", return_objects=True) == []
        assert jsonrepair.repair_json("[1, 2, 3, 4]", return_objects=True) == [1, 2, 3, 4]
        assert jsonrepair.repair_json("[", return_objects=True) == []
        assert_json_equal(jsonrepair.repair_json("[[1\n\n]"), "[[1]]")

    def test_parse_array_edge_cases(self):
        assert_json_equal(jsonrepair.repair_json("[{]"), "[]")
        assert_json_equal(jsonrepair.repair_json("["), "[]")
        assert_json_equal(jsonrepair.repair_json("[1, 2, 3,"), "[1, 2, 3]")
        assert_json_equal(jsonrepair.repair_json('["a" "b" "c" 1'), '["a", "b", "c", 1]')
        assert_json_equal(jsonrepair.repair_json('{"employees":["John", "Anna",'), '{"employees": ["John", "Anna"]}')
        assert_json_equal(jsonrepair.repair_json('{"employees":["John", "Anna", "Peter'), '{"employees": ["John", "Anna", "Peter"]}')
        assert_json_equal(jsonrepair.repair_json('{"key1": {"key2": [1, 2, 3'), '{"key1": {"key2": [1, 2, 3]}}')
        assert_json_equal(jsonrepair.repair_json('{"key": ["value]}'), '{"key": ["value"]}')


class TestParseComment:
    """Test comment handling"""

    def test_parse_comment(self):
        assert_json_equal(jsonrepair.repair_json('{ "key": { "key2": "value2" // comment }, "key3": "value3" }'), '{"key": {"key2": "value2"}, "key3": "value3"}')
        assert_json_equal(jsonrepair.repair_json('{ "key": { "key2": "value2" # comment }, "key3": "value3" }'), '{"key": {"key2": "value2"}, "key3": "value3"}')
        assert_json_equal(jsonrepair.repair_json('{ "key": { "key2": "value2" /* comment */ }, "key3": "value3" }'), '{"key": {"key2": "value2"}, "key3": "value3"}')
        assert_json_equal(jsonrepair.repair_json('[ "value", /* comment */ "value2" ]'), '["value", "value2"]')
        assert_json_equal(jsonrepair.repair_json('{ "key": "value" /* comment'), '{"key": "value"}')


class TestParseNumber:
    """Test number parsing"""

    def test_parse_number(self):
        assert jsonrepair.repair_json("1", return_objects=True) == 1
        assert jsonrepair.repair_json("1.2", return_objects=True) == 1.2

    def test_parse_number_edge_cases(self):
        assert_json_equal(jsonrepair.repair_json('{"key": .25}'), '{"key": 0.25}')
        assert_json_equal(jsonrepair.repair_json("[105,12"), "[105, 12]")
        assert_json_equal(jsonrepair.repair_json('{"key": 1. }'), '{"key": 1.0}')


class TestParseBooleanOrNull:
    """Test boolean and null parsing"""

    def test_parse_boolean_or_null(self):
        assert jsonrepair.repair_json("true", return_objects=True)
        assert not jsonrepair.repair_json("false", return_objects=True)
        assert jsonrepair.repair_json("null", return_objects=True) is None
        assert_json_equal(jsonrepair.repair_json('  {"key": true, "key2": false, "key3": null}'), '{"key": true, "key2": false, "key3": null}')
        assert_json_equal(jsonrepair.repair_json('{"key": True, "key2": False, "key3": None}   '), '{"key": true, "key2": false, "key3": null}')


class TestLoads:
    """Test loads function (like json.loads)"""

    def test_loads_basic(self):
        assert jsonrepair.loads("{name: 'John', age: 30}") == {'name': 'John', 'age': 30}
        assert jsonrepair.loads("[1, 2, 3,]") == [1, 2, 3]
        assert jsonrepair.loads('{"key": true, "key2": false, "key3": null}') == {
            "key": True,
            "key2": False,
            "key3": None,
        }

    def test_loads_with_comments(self):
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

    def test_loads_python_keywords(self):
        result = jsonrepair.loads('{active: True, inactive: False, value: None}')
        assert result['active'] is True
        assert result['inactive'] is False
        assert result['value'] is None


if __name__ == '__main__':
    pytest.main([__file__, '-v'])

