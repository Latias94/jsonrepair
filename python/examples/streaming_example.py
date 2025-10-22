"""
Example: Streaming repair with push/flush and NDJSON aggregation.

Run with:
    python -m jsonrepair.examples.streaming_example
or:
    python examples/streaming_example.py
"""

import json

import jsonrepair


def basic_stream():
    print("-- Basic stream --")
    sr = jsonrepair.StreamRepairer(options=jsonrepair.RepairOptions())
    chunks = ["{a:", "1}", "{b:", "2}"]
    outs = []
    for ch in chunks:
        if (o := sr.push(ch)):
            outs.append(o)
    if (tail := sr.flush()):
        outs.append(tail)
    print("chunks:", chunks)
    print("outs:", outs)


def ndjson_aggregate():
    print("-- NDJSON aggregate --")
    sr = jsonrepair.StreamRepairer(
        options=jsonrepair.RepairOptions(stream_ndjson_aggregate=True)
    )
    sr.push("{a:1}")
    sr.push("{b:2}")
    out = sr.flush()
    print("aggregated:", out)
    print("parsed:", json.loads(out))


def main():
    basic_stream()
    ndjson_aggregate()


if __name__ == "__main__":
    main()

