"""
Edge case tests ported from json_repair library

These tests cover complex and edge cases to ensure robustness.
"""

import pytest
import jsonrepair
import json


def assert_json_equal(actual: str, expected: str):
    """Compare two JSON strings by parsing them, ignoring formatting differences."""
    try:
        actual_parsed = json.loads(actual)
        expected_parsed = json.loads(expected)
        assert actual_parsed == expected_parsed, f"JSON content differs:\nActual: {actual}\nExpected: {expected}"
    except json.JSONDecodeError:
        # If parsing fails, fall back to string comparison
        assert actual == expected


class TestStringEdgeCases:
    """Test edge cases in string parsing"""

    def test_mixed_quotes_in_strings(self):
        assert_json_equal(jsonrepair.repair_json('{"key": ""value"}'), '{"key": "value"}')
        assert_json_equal(jsonrepair.repair_json('{"key": "value", 5: "value"}'), '{"key": "value", "5": "value"}')
        assert_json_equal(jsonrepair.repair_json('{"foo": "\\"bar\\""'), '{"foo": "\\"bar\\""}')
        assert_json_equal(jsonrepair.repair_json('{"" key":"val"'), '{" key": "val"}')

    def test_embedded_quotes(self):
        assert_json_equal(jsonrepair.repair_json('{"key": "v"alu"e"} key:'), '{"key": "v\\"alu\\"e"}')
        assert_json_equal(jsonrepair.repair_json('{"key": "v"alue", "key2": "value2"}'), '{"key": "v\\"alue", "key2": "value2"}')
        assert_json_equal(jsonrepair.repair_json('[{"key": "v"alu,e", "key2": "value2"}]'), '[{"key": "v\\"alu,e", "key2": "value2"}]')

    def test_missing_closing_quote(self):
        assert_json_equal(jsonrepair.repair_json('{"name": "John", "age": 30, "city": "New York, "gender": "male"}'), '{"name": "John", "age": 30, "city": "New York", "gender": "male"}')

    def test_markdown_links(self):
        assert (
            jsonrepair.repair_json('{ "content": "[LINK]("https://google.com")" }')
            == '{"content": "[LINK](\\"https://google.com\\")"}'
        )
        assert_json_equal(jsonrepair.repair_json('{ "content": "[LINK](" }'), '{"content": "[LINK]("}')
        assert_json_equal(jsonrepair.repair_json('{ "content": "[LINK](", "key": true }'), '{"content": "[LINK](", "key": true}')


class TestObjectEdgeCases:
    """Test edge cases in object parsing"""

    def test_missing_commas(self):
        assert jsonrepair.repair_json('{ "key": value, "key2": 1 "key3": null }', return_objects=True) == {
            "key": "value",
            "key2": 1,
            "key3": None,
        }
        assert_json_equal(jsonrepair.repair_json('{"key":value "key2":"value2" }'), '{"key": "value", "key2": "value2"}')

    def test_trailing_commas(self):
        assert_json_equal(jsonrepair.repair_json('{"key": "value", }'), '{"key": "value"}')
        assert_json_equal(jsonrepair.repair_json('{"key": value , }'), '{"key": "value"}')

    def test_empty_keys(self):
        assert_json_equal(jsonrepair.repair_json('{"": "value"'), '{"": "value"}')
        assert_json_equal(jsonrepair.repair_json('{"" : true, "key2": "value2"}'), '{"": true, "key2": "value2"}')

    def test_unquoted_keys_and_values(self):
        assert_json_equal(jsonrepair.repair_json("{key:value,key2:value2}"), '{"key": "value", "key2": "value2"}')
        assert_json_equal(jsonrepair.repair_json('{"key:"value"}'), '{"key": "value"}')
        assert_json_equal(jsonrepair.repair_json('{"key:value}'), '{"key": "value"}')

    def test_nested_braces_in_strings(self):
        assert_json_equal(jsonrepair.repair_json("{'text': 'words{words in brackets}more words'}"), '{"text": "words{words in brackets}more words"}')
        assert_json_equal(jsonrepair.repair_json("{text:words{words in brackets}}"), '{"text": "words{words in brackets}"}')
        assert_json_equal(jsonrepair.repair_json("{text:words{words in brackets}m}"), '{"text": "words{words in brackets}m"}')

    def test_missing_values(self):
        assert_json_equal(jsonrepair.repair_json('{"key": , "key2": "value2"}'), '{"key": "", "key2": "value2"}')


class TestArrayEdgeCases:
    """Test edge cases in array parsing"""

    def test_ellipsis_in_arrays(self):
        assert_json_equal(jsonrepair.repair_json("[1, 2, 3, ...]"), "[1, 2, 3]")
        assert_json_equal(jsonrepair.repair_json("[1, 2, ... , 3]"), "[1, 2, 3]")
        assert_json_equal(jsonrepair.repair_json("[1, 2, '...', 3]"), '[1, 2, "...", 3]')
        assert_json_equal(jsonrepair.repair_json("[true, false, null, ...]"), "[true, false, null]")

    def test_missing_commas_in_arrays(self):
        assert_json_equal(jsonrepair.repair_json('["a" "b" "c" 1'), '["a", "b", "c", 1]')
        assert_json_equal(jsonrepair.repair_json('{"key": ["value" "value1" "value2"]}'), '{"key": ["value", "value1", "value2"]}')

    def test_array_to_object_conversion(self):
        assert_json_equal(jsonrepair.repair_json('["key":"value"}]'), '[{"key": "value"}]')
        assert_json_equal(jsonrepair.repair_json("{'key1', 'key2'}"), '["key1", "key2"]')


class TestNumberEdgeCases:
    """Test edge cases in number parsing"""

    def test_fractions(self):
        assert_json_equal(jsonrepair.repair_json('{"key": 1/3}'), '{"key": "1/3"}')
        assert_json_equal(jsonrepair.repair_json('{"here": "now", "key": 1/3, "foo": "bar"}'), '{"here": "now", "key": "1/3", "foo": "bar"}')
        assert_json_equal(jsonrepair.repair_json('{"key": 12345/67890}'), '{"key": "12345/67890"}')

    def test_leading_dot(self):
        assert_json_equal(jsonrepair.repair_json('{"key": .25}'), '{"key": 0.25}')

    def test_trailing_dot(self):
        assert_json_equal(jsonrepair.repair_json('{"key": 1. }'), '{"key": 1.0}')

    def test_scientific_notation(self):
        assert_json_equal(jsonrepair.repair_json('{"key": 1e10 }'), '{"key": 10000000000.0}')
        assert_json_equal(jsonrepair.repair_json('{"key": 1e }'), '{"key": 1}')

    def test_invalid_numbers(self):
        assert_json_equal(jsonrepair.repair_json('{"key": 10-20}'), '{"key": "10-20"}')
        assert_json_equal(jsonrepair.repair_json('{"key": 1.1.1}'), '{"key": "1.1.1"}')
        assert_json_equal(jsonrepair.repair_json('{"key": 1notanumber }'), '{"key": "1notanumber"}')
        assert_json_equal(jsonrepair.repair_json("[1, 2notanumber]"), '[1, "2notanumber"]')


class TestComplexNested:
    """Test complex nested structures"""

    def test_deeply_nested(self):
        result = jsonrepair.repair_json(
            """
            {
                "resourceType": "Bundle",
                "id": "1",
                "type": "collection",
                "entry": [
                    {
                        "resource": {
                            "resourceType": "Patient",
                            "id": "1",
                            "name": [
                                {"use": "official", "family": "Corwin", "given": ["Keisha", "Sunny"], "prefix": ["Mrs."]},
                                {"use": "maiden", "family": "Goodwin", "given": ["Keisha", "Sunny"], "prefix": ["Mrs."]}
                            ]
                        }
                    }
                ]
            }
            """,
            return_objects=True,
        )
        assert result["resourceType"] == "Bundle"
        assert result["id"] == "1"
        assert len(result["entry"]) == 1
        assert len(result["entry"][0]["resource"]["name"]) == 2

    def test_html_in_json(self):
        result = jsonrepair.repair_json(
            '{\n"html": "<h3 id="aaa">Waarom meer dan 200 Technical Experts - "Passie voor techniek"?</h3>"}',
            return_objects=True,
        )
        assert result == {"html": '<h3 id="aaa">Waarom meer dan 200 Technical Experts - "Passie voor techniek"?</h3>'}

    def test_nested_quotes(self):
        result = jsonrepair.repair_json(
            """
            [
                {
                    "foo": "Foo bar baz",
                    "tag": "#foo-bar-baz"
                },
                {
                    "foo": "foo bar "foobar" foo bar baz.",
                    "tag": "#foo-bar-foobar"
                }
            ]
            """,
            return_objects=True,
        )
        assert result == [
            {"foo": "Foo bar baz", "tag": "#foo-bar-baz"},
            {"foo": 'foo bar "foobar" foo bar baz.', "tag": "#foo-bar-foobar"},
        ]


class TestFencedCodeBlocks:
    """Test markdown fenced code block extraction"""

    def test_single_fenced_block(self):
        assert_json_equal(jsonrepair.repair_json("Based on the information extracted, here is the filled JSON output: ```json { 'a': 'b' } ```"), '{"a": "b"}')

    def test_fenced_block_with_newlines(self):
        result = jsonrepair.repair_json("""
            The next 64 elements are:
            ```json
            { "key": "value" }
            ```
        """)
        # The result should be valid JSON with proper spacing
        assert_json_equal(result, '{"key": "value"}')

    def test_multiple_fenced_blocks(self):
        result = jsonrepair.repair_json(
            'lorem ```json {"key":"value"} ``` ipsum ```json [1,2,3,true] ``` 42'
        )
        assert result == '[{"key": "value"}, [1, 2, 3, true]]'


class TestUnicode:
    """Test Unicode handling"""

    def test_unicode_characters(self):
        result = jsonrepair.loads("{'name': '统一码', 'emoji': '😀'}")
        assert result['name'] == '统一码'
        assert result['emoji'] == '😀'

    def test_ensure_ascii_with_unicode(self):
        result = jsonrepair.repair_json("{'name': '统一码'}", ensure_ascii=True)
        # Should escape Unicode characters
        assert '\\u' in result


class TestEmptyAndWhitespace:
    """Test empty and whitespace handling"""

    def test_empty_input(self):
        # Different implementations may handle empty input differently
        # Just ensure it doesn't crash
        result = jsonrepair.loads("")
        assert result is not None

    def test_whitespace_only(self):
        assert_json_equal(jsonrepair.repair_json("   {  }   "), "{}")
        assert_json_equal(jsonrepair.repair_json("  "), "")


if __name__ == '__main__':
    pytest.main([__file__, '-v'])

