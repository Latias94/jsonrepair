"""
Basic tests for logging APIs and streaming interface in jsonrepair Python bindings.
"""

import json

import jsonrepair


def test_repair_json_with_log_returns_tuple():
    s, log = jsonrepair.repair_json_with_log("{a:1, /*c*/ b:2}")
    assert isinstance(s, str)
    # Python style separators add spaces after colons and commas
    assert s == '{"a": 1, "b": 2}'
    assert isinstance(log, list)


def test_loads_with_log_basic():
    obj, log = jsonrepair.loads_with_log("[{]}")
    assert obj == []
    assert isinstance(log, list)


def test_stream_repairer_push_and_flush():
    sr = jsonrepair.StreamRepairer(options=jsonrepair.RepairOptions())
    outs = []
    for part in ["{a:", "1}", "{b:", "2}"]:
        out = sr.push(part)
        if out:
            outs.append(out)
    tail = sr.flush()
    if tail:
        outs.append(tail)
    # Expect two repaired objects with Python-style separators
    assert outs == ['{"a": 1}', '{"b": 2}']


def test_stream_repairer_ndjson_aggregate():
    # When stream_ndjson_aggregate=True, expect a single array at flush
    sr = jsonrepair.StreamRepairer(
        options=jsonrepair.RepairOptions(stream_ndjson_aggregate=True)
    )
    out1 = sr.push("{a:1}")
    out2 = sr.push("{b:2}")
    # In aggregate mode, push may not emit until flush
    assert out1 is None or isinstance(out1, str)
    assert out2 is None or isinstance(out2, str)
    arr = sr.flush()
    assert isinstance(arr, str)
    parsed = json.loads(arr)
    assert parsed == [{"a": 1}, {"b": 2}]

