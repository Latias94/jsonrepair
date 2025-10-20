use pyo3::prelude::*;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::types::PyString;
use pyo3::types::PyDict;
use ::jsonrepair::{Options, repair_json as rust_repair_json, from_file as rust_from_file, repair_to_string_with_log as rust_repair_with_log, StreamRepairer};

/// Python wrapper for jsonrepair::Options
///
/// RepairOptions control how the Rust engine tolerates and repairs malformed JSON.
/// Most options default to pragmatic values for LLM/JS outputs while keeping safety.
///
/// Highlights (defaults in parentheses):
/// - tolerate_hash_comments (True): treat `#` as line comment (like //)
/// - repair_undefined (True): convert JS `undefined` to `null`
/// - fenced_code_blocks (True): extract JSON from ```lang ... ```; multiple blocks aggregate to an array
/// - allow_python_keywords (True): normalize True/False/None to true/false/null
/// - ensure_ascii (False): when True, escape non‑ASCII as \uXXXX
/// - normalize_js_nonfinite (True): NaN/Infinity become null
/// - number_tolerance_leading_dot (True): .25 -> 0.25
/// - number_tolerance_trailing_dot (True): 1. -> 1.0
/// - number_tolerance_incomplete_exponent (True): 1e -> 1
/// - number_quote_suspicious (True): quote suspicious numeric‑like tokens (e.g., 1/3)
/// - stream_ndjson_aggregate (False): aggregate NDJSON stream to one array (streaming API)
/// - aggressive_truncation_fix (False): best‑effort close on severe truncation
/// - word_comment_markers ([]): words to strip when found before keys (e.g., ["COMMENT"]) 
/// - logging (False): enable repair logging; use Rust API repair_to_string_with_log to retrieve entries
/// - log_context_window (10), log_json_path (False): control logging detail
/// - assume_valid_json_fastpath (False): skip full validation for already valid JSON (ensure_ascii must be False)
///
/// Tip: You can pass an instance to functions via the `options=` parameter; function‑level flags
/// like `ensure_ascii=` and `skip_json_loads=` still override for convenience.
#[pyclass(name = "RepairOptions")]
#[derive(Clone)]
pub struct PyRepairOptions {
    inner: Options,
}

#[pymethods]
impl PyRepairOptions {
    #[new]
    /// Create a new RepairOptions.
    ///
    /// Parameters mirror Rust jsonrepair::Options; see class docstring for details.
    /// Providing an option is optional; unspecified values take pragmatic defaults.
    #[pyo3(signature = (
        tolerate_hash_comments=true,
        repair_undefined=true,
        fenced_code_blocks=true,
        allow_python_keywords=true,
        ensure_ascii=false,
        normalize_js_nonfinite=true,
        number_tolerance_leading_dot=true,
        number_tolerance_trailing_dot=true,
        number_tolerance_incomplete_exponent=true,
        // extra toggles exposed from Rust Options (optional)
        stream_ndjson_aggregate=false,
        compat_python_friendly=false,
        number_quote_suspicious=true,
        aggressive_truncation_fix=false,
        logging=false,
        log_context_window=10,
        log_json_path=false,
        assume_valid_json_fastpath=false,
        word_comment_markers=Vec::new(),
    ))]
    fn new(
        tolerate_hash_comments: bool,
        repair_undefined: bool,
        fenced_code_blocks: bool,
        allow_python_keywords: bool,
        ensure_ascii: bool,
        normalize_js_nonfinite: bool,
        number_tolerance_leading_dot: bool,
        number_tolerance_trailing_dot: bool,
        number_tolerance_incomplete_exponent: bool,
        stream_ndjson_aggregate: bool,
        compat_python_friendly: bool,
        number_quote_suspicious: bool,
        aggressive_truncation_fix: bool,
        logging: bool,
        log_context_window: usize,
        log_json_path: bool,
        assume_valid_json_fastpath: bool,
        word_comment_markers: Vec<String>,
    ) -> Self {
        let mut opts = Options::default();
        opts.tolerate_hash_comments = tolerate_hash_comments;
        opts.repair_undefined = repair_undefined;
        opts.fenced_code_blocks = fenced_code_blocks;
        opts.allow_python_keywords = allow_python_keywords;
        opts.ensure_ascii = ensure_ascii;
    opts.python_style_separators = true;
        opts.normalize_js_nonfinite = normalize_js_nonfinite;
        opts.number_tolerance_leading_dot = number_tolerance_leading_dot;
        opts.number_tolerance_trailing_dot = number_tolerance_trailing_dot;
        opts.number_tolerance_incomplete_exponent = number_tolerance_incomplete_exponent;
        // new fields
        opts.stream_ndjson_aggregate = stream_ndjson_aggregate;
        opts.compat_python_friendly = compat_python_friendly;
        opts.number_quote_suspicious = number_quote_suspicious;
        opts.aggressive_truncation_fix = aggressive_truncation_fix;
        opts.logging = logging;
        opts.log_context_window = log_context_window;
        opts.log_json_path = log_json_path;
        opts.assume_valid_json_fastpath = assume_valid_json_fastpath;
        opts.word_comment_markers = word_comment_markers;

        PyRepairOptions { inner: opts }
    }

    fn __repr__(&self) -> String {
        format!("RepairOptions(ensure_ascii={}, allow_python_keywords={})",
                self.inner.ensure_ascii, self.inner.allow_python_keywords)
    }
}

impl Default for PyRepairOptions {
    fn default() -> Self {
        PyRepairOptions {
            inner: Options::default(),
        }
    }
}

/// Repair a broken JSON string and return the repaired JSON string.
///
/// This is compatible with Python's json_repair.repair_json() function.
///
/// Args:
///     json_str: The broken JSON string to repair
///     return_objects: If True, parse and return Python objects instead of string
///     skip_json_loads: If True, skip validation (faster but less safe)
///     ensure_ascii: If True, escape non-ASCII characters
///     **kwargs: Additional options (currently ignored for compatibility)
///
/// Returns:
///     str or dict/list: Repaired JSON string, or parsed object if return_objects=True
///
/// Examples:
///     >>> repair_json("{name: 'John', age: 30,}")
///     '{"name":"John","age":30}'
///     
///     >>> repair_json("{name: 'John'}", return_objects=True)
///     {'name': 'John'}
#[pyfunction]
/// Repair a broken JSON string and return a JSON string or Python object.
///
/// Arguments:
/// - json_str (str): input possibly malformed JSON text
/// - return_objects (bool): when True, parse and return Python objects; otherwise return JSON str
/// - _skip_json_loads (bool): when True, assume valid JSON and skip full validation (faster)
/// - ensure_ascii (bool): when True, escape non‑ASCII characters in output
/// - options (RepairOptions | None): engine options (class doc has details); function flags override
#[pyo3(signature = (json_str, return_objects=false, _skip_json_loads=false, ensure_ascii=false, options=None))]
fn repair_json(
    py: Python,
    json_str: &str,
    return_objects: bool,
    _skip_json_loads: bool,
    ensure_ascii: bool,
    options: Option<PyRepairOptions>,
) -> PyResult<Py<PyAny>> {
    // Handle empty input - return empty string like json_repair does
    if json_str.trim().is_empty() {
        return Ok(PyString::new(py, "").into());
    }

    let mut opts = if let Some(o) = options { o.inner.clone() } else { Options::default() };
    // function-level override keeps backward compatibility
    opts.ensure_ascii = ensure_ascii;
    opts.python_style_separators = true;\n    opts.python_style_separators = true;
    if _skip_json_loads {
        // Use fast path for already-valid JSON when caller signals skipping validation
        opts.assume_valid_json_fastpath = true;
    }

    // Repair the JSON
    let repaired = rust_repair_json(json_str, &opts)
        .map_err(|e| PyValueError::new_err(format!("Failed to repair JSON: {}", e)))?;

    // Handle empty result from repair
    if repaired.trim().is_empty() {
        return Ok(PyString::new(py, "").into());
    }

    if return_objects {
        let value: serde_json::Value = serde_json::from_str(&repaired)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse JSON: {}", e)))?;
        Ok(pythonize::pythonize(py, &value)
            .map_err(|e| PyValueError::new_err(format!("Failed to convert to Python: {}", e)))?
            .into())
    } else {
        Ok(PyString::new(py, &repaired).into())
    }
}

/// Repair and parse a JSON string, returning Python objects.
///
/// This is compatible with Python's json_repair.loads() and json.loads().
///
/// Args:
///     json_str: The broken JSON string to repair and parse
///     skip_json_loads: If True, skip validation (faster but less safe)
///     ensure_ascii: If True, escape non-ASCII characters
///
/// Returns:
///     dict or list: Parsed Python object
///
/// Examples:
///     >>> loads("{name: 'John', age: 30}")
///     {'name': 'John', 'age': 30}
#[pyfunction]
/// Repair a JSON string and parse into Python objects (like json.loads).
///
/// Arguments:
/// - json_str (str): input possibly malformed JSON text
/// - skip_json_loads (bool): when True, assume valid JSON and skip full validation (faster)
/// - ensure_ascii (bool): when True, escape non‑ASCII characters in output
/// - options (RepairOptions | None): engine options; function flags override
#[pyo3(signature = (json_str, skip_json_loads=false, ensure_ascii=false, options=None))]
fn loads(
    py: Python,
    json_str: &str,
    skip_json_loads: bool,
    ensure_ascii: bool,
    options: Option<PyRepairOptions>,
) -> PyResult<Py<PyAny>> {
    repair_json(py, json_str, true, skip_json_loads, ensure_ascii, options)
}

/// Repair and parse JSON from a file-like object.
///
/// This is compatible with Python's json_repair.load() and json.load().
///
/// Args:
///     fp: A file-like object with a .read() method
///     skip_json_loads: If True, skip validation (faster but less safe)
///     ensure_ascii: If True, escape non-ASCII characters
///
/// Returns:
///     dict or list: Parsed Python object
///
/// Examples:
///     >>> with open('broken.json') as f:
///     ...     data = load(f)
#[pyfunction]
/// Repair and parse from a file‑like object (has .read()).
///
/// Arguments:
/// - fp: file‑like object supporting .read()
/// - skip_json_loads (bool): assume valid JSON and skip full validation
/// - ensure_ascii (bool): when True, escape non‑ASCII characters
/// - options (RepairOptions | None): engine options; function flags override
#[pyo3(signature = (fp, skip_json_loads=false, ensure_ascii=false, options=None))]
fn load(
    py: Python,
    fp: &Bound<'_, PyAny>,
    skip_json_loads: bool,
    ensure_ascii: bool,
    options: Option<PyRepairOptions>,
) -> PyResult<Py<PyAny>> {
    // Call fp.read() to get the content
    let content: String = fp.call_method0("read")?.extract()?;
    loads(py, &content, skip_json_loads, ensure_ascii, options)
}

/// Repair and parse JSON from a file path.
///
/// This is compatible with Python's json_repair.from_file().
///
/// Args:
///     filename: Path to the JSON file
///     skip_json_loads: If True, skip validation (faster but less safe)
///     ensure_ascii: If True, escape non-ASCII characters
///
/// Returns:
///     dict or list: Parsed Python object
///
/// Examples:
///     >>> data = from_file('broken.json')
#[pyfunction]
/// Repair and parse from a filename path (like json.load/json_repair.from_file).
///
/// Arguments:
/// - filename (str): path to file
/// - _skip_json_loads (bool): assume valid JSON and skip validation
/// - ensure_ascii (bool): when True, escape non‑ASCII characters
/// - options (RepairOptions | None): engine options; function flags override
#[pyo3(signature = (filename, _skip_json_loads=false, ensure_ascii=false, options=None))]
fn from_file(
    py: Python,
    filename: &str,
    _skip_json_loads: bool,
    ensure_ascii: bool,
    options: Option<PyRepairOptions>,
) -> PyResult<Py<PyAny>> {
    let mut opts = if let Some(o) = options { o.inner.clone() } else { Options::default() };
    opts.ensure_ascii = ensure_ascii;
    opts.python_style_separators = true;
    if _skip_json_loads {
        opts.assume_valid_json_fastpath = true;
    }

    // Use jsonrepair's from_file function
    let value = rust_from_file(filename, &opts)
        .map_err(|e| PyIOError::new_err(format!("Failed to read or repair file: {}", e)))?;

    Ok(pythonize::pythonize(py, &value)
        .map_err(|e| PyValueError::new_err(format!("Failed to convert to Python: {}", e)))?
        .into())
}

fn make_log_list(py: Python<'_>, log: Vec<jsonrepair::RepairLogEntry>) -> Py<PyAny> {
    let mut vec: Vec<Py<PyAny>> = Vec::with_capacity(log.len());
    for e in log.into_iter() {
        let d = PyDict::new(py);
        let _ = d.set_item("position", e.position);
        let _ = d.set_item("message", e.message);
        let _ = d.set_item("context", e.context);
        if let Some(p) = e.path { let _ = d.set_item("path", p); } else { let _ = d.set_item("path", py.None()); }
        let pd: Py<PyDict> = d.into();
        vec.push(pd.into());
    }
    let list = pyo3::types::PyList::new(py, &vec).expect("build log list");
    let pl: Py<pyo3::types::PyList> = list.into();
    pl.into()
}

#[pyfunction]
/// Repair a broken JSON string and return (result, log).
/// When `return_objects=True`, the first element is a Python object, otherwise a JSON string.
#[pyo3(signature = (json_str, return_objects=false, ensure_ascii=false, options=None))]
fn repair_json_with_log(
    py: Python,
    json_str: &str,
    return_objects: bool,
    ensure_ascii: bool,
    options: Option<PyRepairOptions>,
) -> PyResult<(Py<PyAny>, Py<PyAny>)> {
    let mut opts = if let Some(o) = options { o.inner.clone() } else { Options::default() };
    opts.ensure_ascii = ensure_ascii;
    opts.python_style_separators = true;\n    opts.python_style_separators = true;
    let (repaired, log) = rust_repair_with_log(json_str, &opts)
        .map_err(|e| PyValueError::new_err(format!("Failed to repair JSON: {}", e)))?;
    let pylog = make_log_list(py, log);
    if return_objects {
        let value: serde_json::Value = serde_json::from_str(&repaired)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse JSON: {}", e)))?;
        let obj: Py<PyAny> = pythonize::pythonize(py, &value)
            .map_err(|e| PyValueError::new_err(format!("Failed to convert to Python: {}", e)))?
            .into();
        Ok((obj, pylog))
    } else {
        let s_obj: Py<PyAny> = PyString::new(py, &repaired).into();
        Ok((s_obj, pylog))
    }
}

#[pyfunction]
/// Repair and parse a JSON string; return (object, log).
#[pyo3(signature = (json_str, ensure_ascii=false, options=None))]
fn loads_with_log(
    py: Python,
    json_str: &str,
    ensure_ascii: bool,
    options: Option<PyRepairOptions>,
) -> PyResult<(Py<PyAny>, Py<PyAny>)> {
    repair_json_with_log(py, json_str, true, ensure_ascii, options)
}

#[pyfunction]
/// Repair and parse JSON from a file path; return (object, log).
#[pyo3(signature = (filename, ensure_ascii=false, options=None))]
fn from_file_with_log(
    py: Python,
    filename: &str,
    ensure_ascii: bool,
    options: Option<PyRepairOptions>,
) -> PyResult<(Py<PyAny>, Py<PyAny>)> {
    let content = std::fs::read_to_string(filename)
        .map_err(|e| PyIOError::new_err(format!("Failed to open file: {}", e)))?;
    repair_json_with_log(py, &content, true, ensure_ascii, options)
}

#[pyclass(name = "StreamRepairer")]
/// Streaming JSON repairer that accepts chunks and emits repaired JSON pieces.
/// Methods:
/// - push(chunk: str) -> Optional[str]: feed a chunk; may emit a JSON string when a value completes
/// - flush() -> Optional[str]: flush any remaining buffered output
///
/// If `options.stream_ndjson_aggregate=True`, the output will be a single JSON array.
pub struct PyStreamRepairer {
    inner: StreamRepairer,
}

#[pymethods]
impl PyStreamRepairer {
    #[new]
    #[pyo3(signature = (options=None))]
    fn new(options: Option<PyRepairOptions>) -> Self {
        let opts = if let Some(o) = options { o.inner.clone() } else { Options::default() };
        Self { inner: StreamRepairer::new(opts) }
    }

    /// Push a UTF‑8 chunk; returns a JSON string when a complete value is produced, otherwise None.
    fn push(&mut self, chunk: &str) -> PyResult<Option<String>> {
        self.inner.push(chunk).map_err(|e| PyValueError::new_err(format!("stream push error: {}", e)))
    }

    /// Flush any buffered output; returns a JSON string if something remains, otherwise None.
    fn flush(&mut self) -> PyResult<Option<String>> {
        self.inner.flush().map_err(|e| PyValueError::new_err(format!("stream flush error: {}", e)))
    }
}

/// Fast JSON repair library - Rust-powered Python bindings
///
/// This module provides a drop-in replacement for Python's json_repair library,
/// with significantly better performance thanks to Rust implementation.
///
/// Main functions:
///     - repair_json(): Repair broken JSON and return string or object
///     - loads(): Repair and parse JSON string (like json.loads)
///     - load(): Repair and parse from file object (like json.load)
///     - from_file(): Repair and parse from file path
///
/// Example:
///     >>> import jsonrepair
///     >>> jsonrepair.loads("{name: 'John', age: 30}")
///     {'name': 'John', 'age': 30}
#[pymodule]
fn _jsonrepair(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(repair_json, m)?)?;
    m.add_function(wrap_pyfunction!(loads, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(from_file, m)?)?;
    m.add_function(wrap_pyfunction!(repair_json_with_log, m)?)?;
    m.add_function(wrap_pyfunction!(loads_with_log, m)?)?;
    m.add_function(wrap_pyfunction!(from_file_with_log, m)?)?;
    m.add_class::<PyStreamRepairer>()?;
    m.add_class::<PyRepairOptions>()?;
    Ok(())
}




