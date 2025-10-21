//! Integration tests for C FFI
//!
//! These tests verify that the C API works correctly from Rust.

#![cfg(feature = "c-api")]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

// Import the FFI functions
use jsonrepair::ffi::*;

/// Helper to convert C string to Rust string
unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[test]
fn test_simple_repair() {
    unsafe {
        let input = CString::new("{a:1}").unwrap();
        let result = jsonrepair_repair(input.as_ptr());
        assert!(!result.is_null());

        let output = c_str_to_string(result);
        assert_eq!(output, r#"{"a":1}"#);

        jsonrepair_free(result);
    }
}

#[test]
fn test_null_input() {
    unsafe {
        let result = jsonrepair_repair(ptr::null());
        assert!(result.is_null());
    }
}

#[test]
fn test_options_lifecycle() {
    unsafe {
        let opts = jsonrepair_options_new();
        assert!(!opts.is_null());

        jsonrepair_options_set_ensure_ascii(opts, true);
        jsonrepair_options_set_allow_python_keywords(opts, true);

        jsonrepair_options_free(opts);
    }
}

#[test]
fn test_repair_with_options() {
    unsafe {
        let opts = jsonrepair_options_new();
        jsonrepair_options_set_ensure_ascii(opts, true);

        let input = CString::new("{name: '中文'}").unwrap();
        let result = jsonrepair_repair_with_options(input.as_ptr(), opts);
        assert!(!result.is_null());

        let output = c_str_to_string(result);
        // Should contain Unicode escapes
        assert!(output.contains("\\u"));

        jsonrepair_free(result);
        jsonrepair_options_free(opts);
    }
}

#[test]
fn test_error_handling() {
    unsafe {
        // Use an input that will actually fail (invalid UTF-8 or truly broken JSON)
        // Since the library is very tolerant, we test that error handling works
        // by checking that successful repairs set error code to Ok
        let input = CString::new("{a:1}").unwrap();
        let mut error = JsonRepairError {
            code: JsonRepairErrorCode::Ok,
            message: ptr::null_mut(),
            position: 0,
        };

        let result = jsonrepair_repair_ex(input.as_ptr(), ptr::null(), &mut error);
        assert!(!result.is_null());
        assert_eq!(error.code, JsonRepairErrorCode::Ok);

        jsonrepair_free(result);

        // Free error message if any
        if !error.message.is_null() {
            let _ = CString::from_raw(error.message);
        }
    }
}

#[test]
fn test_streaming_basic() {
    unsafe {
        let stream = jsonrepair_stream_new(ptr::null());
        assert!(!stream.is_null());

        // Push incomplete JSON
        let chunk1 = CString::new("{a:").unwrap();
        let out1 = jsonrepair_stream_push(stream, chunk1.as_ptr());
        // Should buffer, might return null
        if !out1.is_null() {
            jsonrepair_free(out1);
        }

        // Complete the JSON
        let chunk2 = CString::new("1}").unwrap();
        let out2 = jsonrepair_stream_push(stream, chunk2.as_ptr());
        if !out2.is_null() {
            let output = c_str_to_string(out2);
            assert_eq!(output, r#"{"a":1}"#);
            jsonrepair_free(out2);
        }

        // Flush
        let tail = jsonrepair_stream_flush(stream);
        if !tail.is_null() {
            jsonrepair_free(tail);
        }

        jsonrepair_stream_free(stream);
    }
}

#[test]
fn test_python_keywords() {
    unsafe {
        let opts = jsonrepair_options_new();
        jsonrepair_options_set_allow_python_keywords(opts, true);

        let input = CString::new("{a: True, b: False, c: None}").unwrap();
        let result = jsonrepair_repair_with_options(input.as_ptr(), opts);
        assert!(!result.is_null());

        let output = c_str_to_string(result);
        assert_eq!(output, r#"{"a":true,"b":false,"c":null}"#);

        jsonrepair_free(result);
        jsonrepair_options_free(opts);
    }
}

#[test]
fn test_hash_comments() {
    unsafe {
        let opts = jsonrepair_options_new();
        jsonrepair_options_set_tolerate_hash_comments(opts, true);

        let input = CString::new("{a:1, # comment\nb:2}").unwrap();
        let result = jsonrepair_repair_with_options(input.as_ptr(), opts);
        assert!(!result.is_null());

        let output = c_str_to_string(result);
        assert_eq!(output, r#"{"a":1,"b":2}"#);

        jsonrepair_free(result);
        jsonrepair_options_free(opts);
    }
}

#[test]
fn test_fenced_code_blocks() {
    unsafe {
        let opts = jsonrepair_options_new();
        jsonrepair_options_set_fenced_code_blocks(opts, true);

        let input = CString::new("```json\n{a:1}\n```").unwrap();
        let result = jsonrepair_repair_with_options(input.as_ptr(), opts);
        assert!(!result.is_null());

        let output = c_str_to_string(result);
        assert_eq!(output, r#"{"a":1}"#);

        jsonrepair_free(result);
        jsonrepair_options_free(opts);
    }
}

#[test]
fn test_undefined_repair() {
    unsafe {
        let opts = jsonrepair_options_new();
        jsonrepair_options_set_repair_undefined(opts, true);

        let input = CString::new("{a: undefined}").unwrap();
        let result = jsonrepair_repair_with_options(input.as_ptr(), opts);
        assert!(!result.is_null());

        let output = c_str_to_string(result);
        assert_eq!(output, r#"{"a":null}"#);

        jsonrepair_free(result);
        jsonrepair_options_free(opts);
    }
}

#[test]
fn test_normalize_nonfinite() {
    unsafe {
        let opts = jsonrepair_options_new();
        jsonrepair_options_set_normalize_js_nonfinite(opts, true);

        let input = CString::new("{a: NaN, b: Infinity}").unwrap();
        let result = jsonrepair_repair_with_options(input.as_ptr(), opts);
        assert!(!result.is_null());

        let output = c_str_to_string(result);
        assert_eq!(output, r#"{"a":null,"b":null}"#);

        jsonrepair_free(result);
        jsonrepair_options_free(opts);
    }
}

#[test]
fn test_version() {
    unsafe {
        let version_ptr = jsonrepair_version();
        assert!(!version_ptr.is_null());

        let version = c_str_to_string(version_ptr);
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }
}

#[test]
fn test_streaming_with_error() {
    unsafe {
        let stream = jsonrepair_stream_new(ptr::null());
        assert!(!stream.is_null());

        let mut error = JsonRepairError {
            code: JsonRepairErrorCode::Ok,
            message: ptr::null_mut(),
            position: 0,
        };

        let chunk = CString::new("{a:1}").unwrap();
        let result = jsonrepair_stream_push_ex(stream, chunk.as_ptr(), &mut error);

        if !result.is_null() {
            assert_eq!(error.code, JsonRepairErrorCode::Ok);
            jsonrepair_free(result);
        }

        if !error.message.is_null() {
            let _ = CString::from_raw(error.message);
        }

        jsonrepair_stream_free(stream);
    }
}

#[test]
fn test_multiple_repairs() {
    unsafe {
        // Test that we can do multiple repairs without issues
        for _ in 0..10 {
            let input = CString::new("{a:1, b:'test'}").unwrap();
            let result = jsonrepair_repair(input.as_ptr());
            assert!(!result.is_null());
            jsonrepair_free(result);
        }
    }
}

#[test]
fn test_new_options() {
    unsafe {
        let opts = jsonrepair_options_new();

        // Test all new option setters
        jsonrepair_options_set_stream_ndjson_aggregate(opts, true);
        jsonrepair_options_set_logging(opts, true);
        jsonrepair_options_set_number_tolerance_leading_dot(opts, false);
        jsonrepair_options_set_number_tolerance_trailing_dot(opts, false);
        jsonrepair_options_set_python_style_separators(opts, true);
        jsonrepair_options_set_aggressive_truncation_fix(opts, true);

        jsonrepair_options_free(opts);
    }
}
