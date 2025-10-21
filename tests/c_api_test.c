/**
 * C API Tests for jsonrepair
 * 
 * Compile:
 *   gcc -o c_api_test c_api_test.c -I../include -L../target/release -ljsonrepair
 * 
 * Run (Linux/macOS):
 *   LD_LIBRARY_PATH=../target/release ./c_api_test
 * 
 * Run (Windows):
 *   set PATH=%PATH%;..\target\release
 *   c_api_test.exe
 */

#include "../include/jsonrepair.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

// Test counter
static int tests_run = 0;
static int tests_passed = 0;

// NOTE: Guard multi-statement macros with do { ... } while (0)
// to avoid unguarded statements when used in if/else contexts.
#define TEST(name) \
    do { \
        printf("Testing: %s ... ", name); \
        tests_run++; \
    } while (0)

#define PASS() \
    do { \
        printf("PASS\n"); \
        tests_passed++; \
    } while (0)

#define FAIL(msg) \
    do { \
        printf("FAIL: %s\n", msg); \
        exit(1); \
    } while (0)

#define ASSERT_NOT_NULL(ptr) \
    do { \
        if ((ptr) == NULL) FAIL("Expected non-NULL pointer"); \
    } while (0)

#define ASSERT_NULL(ptr) \
    do { \
        if ((ptr) != NULL) FAIL("Expected NULL pointer"); \
    } while (0)

#define ASSERT_STR_EQ(actual, expected) \
    do { \
        if (strcmp((actual), (expected)) != 0) { \
            printf("\n  Expected: %s\n  Got: %s\n", (expected), (actual)); \
            FAIL("String mismatch"); \
        } \
    } while (0)

void test_simple_repair() {
    TEST("simple repair");
    
    char* result = jsonrepair_repair("{a:1}");
    ASSERT_NOT_NULL(result);
    ASSERT_STR_EQ(result, "{\"a\":1}");
    jsonrepair_free(result);
    
    PASS();
}

void test_null_input() {
    TEST("null input");
    
    char* result = jsonrepair_repair(NULL);
    ASSERT_NULL(result);
    
    PASS();
}

void test_with_options() {
    TEST("repair with options");
    
    struct Options* opts = jsonrepair_options_new();
    ASSERT_NOT_NULL(opts);
    
    jsonrepair_options_set_ensure_ascii(opts, true);
    
    char* result = jsonrepair_repair_with_options("{name: '中文'}", opts);
    ASSERT_NOT_NULL(result);
    
    // Should contain \u escapes for Chinese characters
    if (strstr(result, "\\u") == NULL) {
        FAIL("Expected Unicode escapes");
    }
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
    
    PASS();
}

void test_error_handling() {
    TEST("error handling");

    struct JsonRepairError error = {0};
    // The library is very tolerant, so test successful repair with error tracking
    char* result = jsonrepair_repair_ex("{a:1}", NULL, &error);

    // Should succeed
    ASSERT_NOT_NULL(result);

    // Error code should be OK for successful repair
    if (error.code != OK) {
        FAIL("Expected OK error code for successful repair");
    }

    jsonrepair_free(result);

    if (error.message != NULL) {
        free(error.message);
    }

    PASS();
}

void test_streaming_basic() {
    TEST("streaming basic");
    
    struct StreamRepairer* stream = jsonrepair_stream_new(NULL);
    ASSERT_NOT_NULL(stream);
    
    // Push incomplete JSON
    char* out1 = jsonrepair_stream_push(stream, "{a:");
    ASSERT_NULL(out1);  // Should buffer
    
    // Complete the JSON
    char* out2 = jsonrepair_stream_push(stream, "1}");
    if (out2 != NULL) {
        ASSERT_STR_EQ(out2, "{\"a\":1}");
        jsonrepair_free(out2);
    }
    
    // Flush remaining
    char* tail = jsonrepair_stream_flush(stream);
    if (tail != NULL) {
        jsonrepair_free(tail);
    }
    
    jsonrepair_stream_free(stream);
    
    PASS();
}

void test_streaming_multiple_values() {
    TEST("streaming multiple values");
    
    struct StreamRepairer* stream = jsonrepair_stream_new(NULL);
    ASSERT_NOT_NULL(stream);
    
    // Push first value
    char* out1 = jsonrepair_stream_push(stream, "{a:1}");
    if (out1 != NULL) {
        ASSERT_STR_EQ(out1, "{\"a\":1}");
        jsonrepair_free(out1);
    }
    
    // Push second value
    char* out2 = jsonrepair_stream_push(stream, "{b:2}");
    if (out2 != NULL) {
        ASSERT_STR_EQ(out2, "{\"b\":2}");
        jsonrepair_free(out2);
    }
    
    jsonrepair_stream_free(stream);
    
    PASS();
}

void test_python_keywords() {
    TEST("Python keywords");
    
    struct Options* opts = jsonrepair_options_new();
    jsonrepair_options_set_allow_python_keywords(opts, true);
    
    char* result = jsonrepair_repair_with_options("{a: True, b: False, c: None}", opts);
    ASSERT_NOT_NULL(result);
    ASSERT_STR_EQ(result, "{\"a\":true,\"b\":false,\"c\":null}");
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
    
    PASS();
}

void test_hash_comments() {
    TEST("hash comments");
    
    struct Options* opts = jsonrepair_options_new();
    jsonrepair_options_set_tolerate_hash_comments(opts, true);
    
    char* result = jsonrepair_repair_with_options("{a:1, # comment\nb:2}", opts);
    ASSERT_NOT_NULL(result);
    ASSERT_STR_EQ(result, "{\"a\":1,\"b\":2}");
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
    
    PASS();
}

void test_fenced_code_blocks() {
    TEST("fenced code blocks");
    
    struct Options* opts = jsonrepair_options_new();
    jsonrepair_options_set_fenced_code_blocks(opts, true);
    
    char* result = jsonrepair_repair_with_options("```json\n{a:1}\n```", opts);
    ASSERT_NOT_NULL(result);
    ASSERT_STR_EQ(result, "{\"a\":1}");
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
    
    PASS();
}

void test_undefined_repair() {
    TEST("undefined repair");
    
    struct Options* opts = jsonrepair_options_new();
    jsonrepair_options_set_repair_undefined(opts, true);
    
    char* result = jsonrepair_repair_with_options("{a: undefined}", opts);
    ASSERT_NOT_NULL(result);
    ASSERT_STR_EQ(result, "{\"a\":null}");
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
    
    PASS();
}

void test_normalize_nonfinite() {
    TEST("normalize non-finite numbers");
    
    struct Options* opts = jsonrepair_options_new();
    jsonrepair_options_set_normalize_js_nonfinite(opts, true);
    
    char* result = jsonrepair_repair_with_options("{a: NaN, b: Infinity}", opts);
    ASSERT_NOT_NULL(result);
    ASSERT_STR_EQ(result, "{\"a\":null,\"b\":null}");
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
    
    PASS();
}

void test_version() {
    TEST("version info");
    
    const char* version = jsonrepair_version();
    ASSERT_NOT_NULL(version);
    
    // Version should not be empty
    if (strlen(version) == 0) {
        FAIL("Version string is empty");
    }
    
    printf("(version: %s) ", version);
    
    PASS();
}

void test_complex_repair() {
    TEST("complex repair");
    
    const char* input = "{name: 'John', age: 30, active: True, data: undefined}";
    char* result = jsonrepair_repair(input);
    ASSERT_NOT_NULL(result);
    
    // Should have proper quotes and conversions
    if (strstr(result, "\"name\"") == NULL) FAIL("Missing quoted key");
    if (strstr(result, "\"John\"") == NULL) FAIL("Missing quoted value");
    if (strstr(result, "true") == NULL) FAIL("Missing Python keyword conversion");
    if (strstr(result, "null") == NULL) FAIL("Missing undefined conversion");
    
    jsonrepair_free(result);
    
    PASS();
}

int main() {
    printf("=================================\n");
    printf("jsonrepair C API Test Suite\n");
    printf("=================================\n\n");
    
    // Run all tests
    test_version();
    test_simple_repair();
    test_null_input();
    test_with_options();
    test_error_handling();
    test_streaming_basic();
    test_streaming_multiple_values();
    test_python_keywords();
    test_hash_comments();
    test_fenced_code_blocks();
    test_undefined_repair();
    test_normalize_nonfinite();
    test_complex_repair();
    
    // Summary
    printf("\n=================================\n");
    printf("Tests run: %d\n", tests_run);
    printf("Tests passed: %d\n", tests_passed);
    printf("Tests failed: %d\n", tests_run - tests_passed);
    printf("=================================\n");
    
    if (tests_passed == tests_run) {
        printf("✓ All tests passed!\n");
        return 0;
    } else {
        printf("✗ Some tests failed!\n");
        return 1;
    }
}
