"""
jsonrepair - Fast JSON repair library

A high-performance JSON repair library powered by Rust, providing a drop-in
replacement for the Python json_repair library with significantly better performance.

Main functions:
    - repair_json(): Repair broken JSON and return string or object
    - loads(): Repair and parse JSON string (like json.loads)
    - load(): Repair and parse from file object (like json.load)
    - from_file(): Repair and parse from file path

Example:
    >>> import jsonrepair
    >>> jsonrepair.loads("{name: 'John', age: 30}")
    {'name': 'John', 'age': 30}
    
    >>> jsonrepair.repair_json("{name: 'John'}")
    '{"name":"John"}'
"""

from ._jsonrepair import (
    repair_json,
    loads,
    load,
    from_file,
    # logging variants
    repair_json_with_log,
    loads_with_log,
    from_file_with_log,
    # streaming
    StreamRepairer,
    RepairOptions,
)

__version__ = "0.1.0"
__all__ = [
    "repair_json",
    "loads",
    "load",
    "from_file",
    "RepairOptions",
    "repair_json_with_log",
    "loads_with_log",
    "from_file_with_log",
    "StreamRepairer",
]
