"""Type stubs for jsonrepair Python bindings."""

from typing import IO, Any, Dict, List, Optional, Tuple

class RepairOptions:
    """
    Options for controlling JSON repair behavior.
    
    All parameters are optional and have sensible defaults for LLM/JS outputs.
    """

    def __init__(
        self,
        tolerate_hash_comments: bool = True,
        repair_undefined: bool = True,
        fenced_code_blocks: bool = True,
        allow_python_keywords: bool = True,
        ensure_ascii: bool = False,
        normalize_js_nonfinite: bool = True,
        number_tolerance_leading_dot: bool = True,
        number_tolerance_trailing_dot: bool = True,
        number_tolerance_incomplete_exponent: bool = True,
        stream_ndjson_aggregate: bool = False,
        compat_python_friendly: bool = False,
        number_quote_suspicious: bool = True,
        aggressive_truncation_fix: bool = False,
        logging: bool = False,
        log_context_window: int = 10,
        log_json_path: bool = False,
        assume_valid_json_fastpath: bool = False,
        word_comment_markers: List[str] = [],
    ) -> None: ...

    def __repr__(self) -> str: ...

    # Property accessors
    @property
    def ensure_ascii(self) -> bool: ...
    @ensure_ascii.setter
    def ensure_ascii(self, value: bool) -> None: ...

    @property
    def tolerate_hash_comments(self) -> bool: ...
    @tolerate_hash_comments.setter
    def tolerate_hash_comments(self, value: bool) -> None: ...

    @property
    def repair_undefined(self) -> bool: ...
    @repair_undefined.setter
    def repair_undefined(self, value: bool) -> None: ...

    @property
    def allow_python_keywords(self) -> bool: ...
    @allow_python_keywords.setter
    def allow_python_keywords(self, value: bool) -> None: ...

    @property
    def fenced_code_blocks(self) -> bool: ...
    @fenced_code_blocks.setter
    def fenced_code_blocks(self, value: bool) -> None: ...

    @property
    def normalize_js_nonfinite(self) -> bool: ...
    @normalize_js_nonfinite.setter
    def normalize_js_nonfinite(self, value: bool) -> None: ...

    @property
    def stream_ndjson_aggregate(self) -> bool: ...
    @stream_ndjson_aggregate.setter
    def stream_ndjson_aggregate(self, value: bool) -> None: ...

    @property
    def logging(self) -> bool: ...
    @logging.setter
    def logging(self, value: bool) -> None: ...

class StreamRepairer:
    """
    Streaming JSON repairer that accepts chunks and emits repaired JSON pieces.
    
    Example:
        >>> repairer = StreamRepairer()
        >>> for chunk in chunks:
        ...     if output := repairer.push(chunk):
        ...         process(output)
        >>> if final := repairer.flush():
        ...     process(final)
    """

    def __init__(self, options: Optional[RepairOptions] = None) -> None: ...

    def push(self, chunk: str) -> Optional[str]:
        """
        Push a UTF-8 chunk; returns a JSON string when a complete value is produced.
        
        Args:
            chunk: Input chunk to process
            
        Returns:
            Repaired JSON string if a complete value was produced, None otherwise
        """
        ...

    def flush(self) -> Optional[str]:
        """
        Flush any buffered output.
        
        Returns:
            Remaining JSON string if any, None otherwise
        """
        ...

def repair_json(
    json_str: str,
    return_objects: bool = False,
    skip_json_loads: bool = False,
    ensure_ascii: bool = False,
    options: Optional[RepairOptions] = None,
) -> str | Dict[str, Any] | List[Any]:
    """
    Repair a broken JSON string and return the repaired JSON string or Python object.
    
    This is compatible with Python's json_repair.repair_json() function.
    
    Args:
        json_str: The broken JSON string to repair
        return_objects: If True, parse and return Python objects instead of string
        skip_json_loads: If True, skip validation (faster but less safe)
        ensure_ascii: If True, escape non-ASCII characters
        options: Optional RepairOptions instance for advanced configuration
        
    Returns:
        Repaired JSON string, or parsed object if return_objects=True
        
    Examples:
        >>> repair_json("{name: 'John', age: 30,}")
        '{"name":"John","age":30}'
        
        >>> repair_json("{name: 'John'}", return_objects=True)
        {'name': 'John'}
    """
    ...

def loads(
    json_str: str,
    skip_json_loads: bool = False,
    ensure_ascii: bool = False,
    options: Optional[RepairOptions] = None,
) -> Any:
    """
    Repair and parse a JSON string, returning Python objects.
    
    This is compatible with Python's json_repair.loads() and json.loads().
    
    Args:
        json_str: The broken JSON string to repair and parse
        skip_json_loads: If True, skip validation (faster but less safe)
        ensure_ascii: If True, escape non-ASCII characters
        options: Optional RepairOptions instance for advanced configuration
        
    Returns:
        Parsed Python object (dict, list, str, int, float, bool, or None)
        
    Examples:
        >>> loads("{name: 'John', age: 30}")
        {'name': 'John', 'age': 30}
    """
    ...

def load(
    fp: IO[str],
    skip_json_loads: bool = False,
    ensure_ascii: bool = False,
    options: Optional[RepairOptions] = None,
) -> Any:
    """
    Repair and parse JSON from a file-like object.
    
    This is compatible with Python's json_repair.load() and json.load().
    
    Args:
        fp: A file-like object with a .read() method
        skip_json_loads: If True, skip validation (faster but less safe)
        ensure_ascii: If True, escape non-ASCII characters
        options: Optional RepairOptions instance for advanced configuration
        
    Returns:
        Parsed Python object
        
    Examples:
        >>> with open('broken.json') as f:
        ...     data = load(f)
    """
    ...

def from_file(
    filename: str,
    skip_json_loads: bool = False,
    ensure_ascii: bool = False,
    options: Optional[RepairOptions] = None,
) -> Any:
    """
    Repair and parse JSON from a file path.
    
    This is compatible with Python's json_repair.from_file().
    
    Args:
        filename: Path to the JSON file
        skip_json_loads: If True, skip validation (faster but less safe)
        ensure_ascii: If True, escape non-ASCII characters
        options: Optional RepairOptions instance for advanced configuration
        
    Returns:
        Parsed Python object
        
    Examples:
        >>> data = from_file('broken.json')
    """
    ...

def repair_json_with_log(
    json_str: str,
    return_objects: bool = False,
    ensure_ascii: bool = False,
    options: Optional[RepairOptions] = None,
) -> Tuple[str | Dict[str, Any] | List[Any], List[Dict[str, Any]]]:
    """
    Repair a broken JSON string and return (result, log).
    
    Args:
        json_str: The broken JSON string to repair
        return_objects: If True, return parsed Python objects instead of string
        ensure_ascii: If True, escape non-ASCII characters
        options: Optional RepairOptions instance for advanced configuration
        
    Returns:
        Tuple of (repaired result, repair log entries)
        
    Examples:
        >>> result, log = repair_json_with_log("{a: 1}")
        >>> print(result)
        '{"a":1}'
        >>> print(log[0]['message'])
        'Added quotes around unquoted key'
    """
    ...

def loads_with_log(
    json_str: str,
    ensure_ascii: bool = False,
    options: Optional[RepairOptions] = None,
) -> Tuple[Any, List[Dict[str, Any]]]:
    """
    Repair and parse a JSON string; return (object, log).
    
    Args:
        json_str: The broken JSON string to repair and parse
        ensure_ascii: If True, escape non-ASCII characters
        options: Optional RepairOptions instance for advanced configuration
        
    Returns:
        Tuple of (parsed object, repair log entries)
    """
    ...

def from_file_with_log(
    filename: str,
    ensure_ascii: bool = False,
    options: Optional[RepairOptions] = None,
) -> Tuple[Any, List[Dict[str, Any]]]:
    """
    Repair and parse JSON from a file path; return (object, log).
    
    Args:
        filename: Path to the JSON file
        ensure_ascii: If True, escape non-ASCII characters
        options: Optional RepairOptions instance for advanced configuration
        
    Returns:
        Tuple of (parsed object, repair log entries)
    """
    ...

