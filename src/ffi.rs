//! C FFI bindings for jsonrepair
//!
//! This module provides a C-compatible API for the jsonrepair library.
//! Enable with the `c-api` feature.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::{Options, RepairError, RepairErrorKind, StreamRepairer};

// ============================================================================
// Error Handling
// ============================================================================

/// Error codes for C API
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonRepairErrorCode {
    Ok = 0,
    UnexpectedEnd = 1,
    UnexpectedChar = 2,
    ObjectKeyExpected = 3,
    ColonExpected = 4,
    InvalidUnicode = 5,
    Parse = 6,
}

/// Error structure for C API
#[repr(C)]
pub struct JsonRepairError {
    pub code: JsonRepairErrorCode,
    pub message: *mut c_char,
    pub position: usize,
}

impl JsonRepairError {
    fn from_repair_error(err: RepairError) -> Self {
        let code = match err.kind {
            RepairErrorKind::UnexpectedEnd => JsonRepairErrorCode::UnexpectedEnd,
            RepairErrorKind::UnexpectedChar(_) => JsonRepairErrorCode::UnexpectedChar,
            RepairErrorKind::ObjectKeyExpected => JsonRepairErrorCode::ObjectKeyExpected,
            RepairErrorKind::ColonExpected => JsonRepairErrorCode::ColonExpected,
            RepairErrorKind::InvalidUnicodeEscape => JsonRepairErrorCode::InvalidUnicode,
            RepairErrorKind::Parse(_) => JsonRepairErrorCode::Parse,
        };

        let message = CString::new(err.to_string())
            .unwrap_or_else(|_| CString::new("Unknown error").unwrap())
            .into_raw();

        JsonRepairError {
            code,
            message,
            position: err.position,
        }
    }

    fn ok() -> Self {
        JsonRepairError {
            code: JsonRepairErrorCode::Ok,
            message: ptr::null_mut(),
            position: 0,
        }
    }
}

// ============================================================================
// Simple API
// ============================================================================

/// Repair a JSON string using default options.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - The returned string must be freed with `jsonrepair_free()`
/// - Returns NULL on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_repair(input: *const c_char) -> *mut c_char {
    unsafe {
        if input.is_null() {
            return ptr::null_mut();
        }

        let c_str = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        match crate::repair_json(c_str, &Options::default()) {
            Ok(result) => CString::new(result)
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Free a string allocated by the library.
///
/// # Safety
/// - `str` must be a string returned by this library, or NULL
/// - Do not use `str` after calling this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_free(str: *mut c_char) {
    unsafe {
        if !str.is_null() {
            drop(CString::from_raw(str));
        }
    }
}

// ============================================================================
// Options API
// ============================================================================

/// Create a new options object with default values.
///
/// Must be freed with `jsonrepair_options_free()`.
#[unsafe(no_mangle)]
pub extern "C" fn jsonrepair_options_new() -> *mut Options {
    Box::into_raw(Box::new(Options::default()))
}

/// Free an options object.
///
/// # Safety
/// - `opts` must be a pointer returned by `jsonrepair_options_new()`, or NULL
/// - Do not use `opts` after calling this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_free(opts: *mut Options) {
    unsafe {
        if !opts.is_null() {
            drop(Box::from_raw(opts));
        }
    }
}

/// Set the ensure_ascii option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_ensure_ascii(opts: *mut Options, value: bool) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.ensure_ascii = value;
        }
    }
}

/// Set the allow_python_keywords option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_allow_python_keywords(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.allow_python_keywords = value;
        }
    }
}

/// Set the tolerate_hash_comments option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_tolerate_hash_comments(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.tolerate_hash_comments = value;
        }
    }
}

/// Set the repair_undefined option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_repair_undefined(opts: *mut Options, value: bool) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.repair_undefined = value;
        }
    }
}

/// Set the fenced_code_blocks option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_fenced_code_blocks(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.fenced_code_blocks = value;
        }
    }
}

/// Set the normalize_js_nonfinite option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_normalize_js_nonfinite(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.normalize_js_nonfinite = value;
        }
    }
}

/// Set the stream_ndjson_aggregate option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_stream_ndjson_aggregate(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.stream_ndjson_aggregate = value;
        }
    }
}

/// Set the logging option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_logging(opts: *mut Options, value: bool) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.logging = value;
        }
    }
}

/// Set the number_tolerance_leading_dot option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_number_tolerance_leading_dot(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.number_tolerance_leading_dot = value;
        }
    }
}

/// Set the number_tolerance_trailing_dot option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_number_tolerance_trailing_dot(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.number_tolerance_trailing_dot = value;
        }
    }
}

/// Set the python_style_separators option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_python_style_separators(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.python_style_separators = value;
        }
    }
}

/// Set the aggressive_truncation_fix option.
///
/// # Safety
/// - `opts` must be a valid pointer to Options
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_options_set_aggressive_truncation_fix(
    opts: *mut Options,
    value: bool,
) {
    unsafe {
        if let Some(opts) = opts.as_mut() {
            opts.aggressive_truncation_fix = value;
        }
    }
}

/// Repair a JSON string with custom options.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - `opts` must be a valid pointer to Options, or NULL for defaults
/// - The returned string must be freed with `jsonrepair_free()`
/// - Returns NULL on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_repair_with_options(
    input: *const c_char,
    opts: *const Options,
) -> *mut c_char {
    unsafe {
        if input.is_null() {
            return ptr::null_mut();
        }

        let c_str = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        let options = if opts.is_null() {
            &Options::default()
        } else {
            &*opts
        };

        match crate::repair_json(c_str, options) {
            Ok(result) => CString::new(result)
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Repair a JSON string with error details.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - `opts` must be a valid pointer to Options, or NULL for defaults
/// - `error` can be NULL to ignore error details
/// - If `error` is not NULL and an error occurs, `error.message` must be freed with `free()`
/// - The returned string must be freed with `jsonrepair_free()`
/// - Returns NULL on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_repair_ex(
    input: *const c_char,
    opts: *const Options,
    error: *mut JsonRepairError,
) -> *mut c_char {
    unsafe {
        if input.is_null() {
            if !error.is_null() {
                *error = JsonRepairError::from_repair_error(RepairError::new(
                    RepairErrorKind::Parse("Input is NULL".to_string()),
                    0,
                ));
            }
            return ptr::null_mut();
        }

        let c_str = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(e) => {
                if !error.is_null() {
                    *error = JsonRepairError::from_repair_error(RepairError::new(
                        RepairErrorKind::Parse(format!("Invalid UTF-8: {}", e)),
                        0,
                    ));
                }
                return ptr::null_mut();
            }
        };

        let options = if opts.is_null() {
            &Options::default()
        } else {
            &*opts
        };

        match crate::repair_json(c_str, options) {
            Ok(result) => {
                if !error.is_null() {
                    *error = JsonRepairError::ok();
                }
                CString::new(result)
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw()
            }
            Err(e) => {
                if !error.is_null() {
                    *error = JsonRepairError::from_repair_error(e);
                }
                ptr::null_mut()
            }
        }
    }
}

// ============================================================================
// Streaming API
// ============================================================================

/// Create a new streaming repairer.
///
/// # Safety
/// - `opts` can be NULL for default options
/// - Must be freed with `jsonrepair_stream_free()`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_stream_new(opts: *const Options) -> *mut StreamRepairer {
    unsafe {
        let options = if opts.is_null() {
            Options::default()
        } else {
            (*opts).clone()
        };

        Box::into_raw(Box::new(StreamRepairer::new(options)))
    }
}

/// Free a streaming repairer.
///
/// # Safety
/// - `stream` must be a pointer returned by `jsonrepair_stream_new()`, or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_stream_free(stream: *mut StreamRepairer) {
    unsafe {
        if !stream.is_null() {
            drop(Box::from_raw(stream));
        }
    }
}

/// Push a chunk to the streaming repairer.
///
/// # Safety
/// - `stream` must be a valid pointer to StreamRepairer
/// - `chunk` must be a valid null-terminated UTF-8 string
/// - Returns NULL if no complete value yet, or a string that must be freed with `jsonrepair_free()`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_stream_push(
    stream: *mut StreamRepairer,
    chunk: *const c_char,
) -> *mut c_char {
    unsafe {
        if stream.is_null() || chunk.is_null() {
            return ptr::null_mut();
        }

        let stream = match stream.as_mut() {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        let c_str = match CStr::from_ptr(chunk).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        match stream.push(c_str) {
            Ok(Some(result)) => CString::new(result)
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            Ok(None) => ptr::null_mut(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Flush the streaming repairer.
///
/// # Safety
/// - `stream` must be a valid pointer to StreamRepairer
/// - Returns NULL if no data, or a string that must be freed with `jsonrepair_free()`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_stream_flush(stream: *mut StreamRepairer) -> *mut c_char {
    unsafe {
        if stream.is_null() {
            return ptr::null_mut();
        }

        let stream = match stream.as_mut() {
            Some(s) => s,
            None => return ptr::null_mut(),
        };

        match stream.flush() {
            Ok(Some(result)) => CString::new(result)
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            Ok(None) => ptr::null_mut(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Push a chunk with error handling.
///
/// # Safety
/// - `stream` must be a valid pointer to StreamRepairer
/// - `chunk` must be a valid null-terminated UTF-8 string
/// - `error` can be NULL to ignore error details
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_stream_push_ex(
    stream: *mut StreamRepairer,
    chunk: *const c_char,
    error: *mut JsonRepairError,
) -> *mut c_char {
    unsafe {
        if stream.is_null() || chunk.is_null() {
            if !error.is_null() {
                *error = JsonRepairError::from_repair_error(RepairError::new(
                    RepairErrorKind::Parse("NULL pointer".to_string()),
                    0,
                ));
            }
            return ptr::null_mut();
        }

        let stream = match stream.as_mut() {
            Some(s) => s,
            None => {
                if !error.is_null() {
                    *error = JsonRepairError::from_repair_error(RepairError::new(
                        RepairErrorKind::Parse("Invalid stream pointer".to_string()),
                        0,
                    ));
                }
                return ptr::null_mut();
            }
        };

        let c_str = match CStr::from_ptr(chunk).to_str() {
            Ok(s) => s,
            Err(e) => {
                if !error.is_null() {
                    *error = JsonRepairError::from_repair_error(RepairError::new(
                        RepairErrorKind::Parse(format!("Invalid UTF-8: {}", e)),
                        0,
                    ));
                }
                return ptr::null_mut();
            }
        };

        match stream.push(c_str) {
            Ok(Some(result)) => {
                if !error.is_null() {
                    *error = JsonRepairError::ok();
                }
                CString::new(result)
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw()
            }
            Ok(None) => {
                if !error.is_null() {
                    *error = JsonRepairError::ok();
                }
                ptr::null_mut()
            }
            Err(e) => {
                if !error.is_null() {
                    *error = JsonRepairError::from_repair_error(e);
                }
                ptr::null_mut()
            }
        }
    }
}

/// Flush with error handling.
///
/// # Safety
/// - `stream` must be a valid pointer to StreamRepairer
/// - `error` can be NULL to ignore error details
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jsonrepair_stream_flush_ex(
    stream: *mut StreamRepairer,
    error: *mut JsonRepairError,
) -> *mut c_char {
    unsafe {
        if stream.is_null() {
            if !error.is_null() {
                *error = JsonRepairError::from_repair_error(RepairError::new(
                    RepairErrorKind::Parse("NULL pointer".to_string()),
                    0,
                ));
            }
            return ptr::null_mut();
        }

        let stream = match stream.as_mut() {
            Some(s) => s,
            None => {
                if !error.is_null() {
                    *error = JsonRepairError::from_repair_error(RepairError::new(
                        RepairErrorKind::Parse("Invalid stream pointer".to_string()),
                        0,
                    ));
                }
                return ptr::null_mut();
            }
        };

        match stream.flush() {
            Ok(Some(result)) => {
                if !error.is_null() {
                    *error = JsonRepairError::ok();
                }
                CString::new(result)
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw()
            }
            Ok(None) => {
                if !error.is_null() {
                    *error = JsonRepairError::ok();
                }
                ptr::null_mut()
            }
            Err(e) => {
                if !error.is_null() {
                    *error = JsonRepairError::from_repair_error(e);
                }
                ptr::null_mut()
            }
        }
    }
}

// ============================================================================
// Version Info
// ============================================================================

/// Get the library version string.
///
/// Returns a static string, do not free.
#[unsafe(no_mangle)]
pub extern "C" fn jsonrepair_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}
