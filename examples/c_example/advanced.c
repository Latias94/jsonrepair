/**
 * Advanced C examples for jsonrepair library
 * Demonstrates all available options and features
 * 
 * Compile:
 *   gcc -o advanced advanced.c -I../../include -L../../target/release -ljsonrepair
 * 
 * Run (Linux/macOS):
 *   LD_LIBRARY_PATH=../../target/release ./advanced
 * 
 * Run (Windows):
 *   set PATH=%PATH%;..\..\target\release
 *   advanced.exe
 */

#include "../../include/jsonrepair.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void print_separator(const char* title) {
    printf("\n=== %s ===\n", title);
}

void example_python_keywords() {
    print_separator("Python Keywords");
    
    const char* input = "{a: True, b: False, c: None}";
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_allow_python_keywords(opts, true);
    
    char* result = jsonrepair_repair_with_options(input, opts);
    
    printf("Input:  %s\n", input);
    printf("Output: %s\n", result);
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
}

void example_hash_comments() {
    print_separator("Hash Comments");
    
    const char* input = "{\n  a: 1,  # This is a comment\n  b: 2   # Another comment\n}";
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_tolerate_hash_comments(opts, true);
    
    char* result = jsonrepair_repair_with_options(input, opts);
    
    printf("Input:\n%s\n", input);
    printf("Output: %s\n", result);
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
}

void example_fenced_code_blocks() {
    print_separator("Fenced Code Blocks");
    
    const char* input = "```json\n{a: 1, b: 'test'}\n```";
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_fenced_code_blocks(opts, true);
    
    char* result = jsonrepair_repair_with_options(input, opts);
    
    printf("Input:\n%s\n", input);
    printf("Output: %s\n", result);
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
}

void example_undefined_repair() {
    print_separator("Undefined Repair");
    
    const char* input = "{a: undefined, b: 'value'}";
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_repair_undefined(opts, true);
    
    char* result = jsonrepair_repair_with_options(input, opts);
    
    printf("Input:  %s\n", input);
    printf("Output: %s\n", result);
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
}

void example_normalize_nonfinite() {
    print_separator("Normalize Non-finite Numbers");
    
    const char* input = "{a: NaN, b: Infinity, c: -Infinity}";
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_normalize_js_nonfinite(opts, true);
    
    char* result = jsonrepair_repair_with_options(input, opts);
    
    printf("Input:  %s\n", input);
    printf("Output: %s\n", result);
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
}

void example_number_tolerance() {
    print_separator("Number Tolerance");
    
    const char* inputs[] = {
        "{a: .5}",      // Leading dot
        "{b: 1.}",      // Trailing dot
        "{c: .25, d: 3.}"
    };
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_number_tolerance_leading_dot(opts, true);
    jsonrepair_options_set_number_tolerance_trailing_dot(opts, true);
    
    for (int i = 0; i < 3; i++) {
        char* result = jsonrepair_repair_with_options(inputs[i], opts);
        printf("Input:  %s\n", inputs[i]);
        printf("Output: %s\n\n", result);
        jsonrepair_free(result);
    }
    
    jsonrepair_options_free(opts);
}

void example_python_style_separators() {
    print_separator("Python Style Separators");
    
    const char* input = "{a:1,b:2,c:3}";
    
    // Without Python style
    printf("Default separators:\n");
    char* result1 = jsonrepair_repair(input);
    printf("  %s\n\n", result1);
    jsonrepair_free(result1);
    
    // With Python style
    printf("Python style separators:\n");
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_python_style_separators(opts, true);
    
    char* result2 = jsonrepair_repair_with_options(input, opts);
    printf("  %s\n", result2);
    
    jsonrepair_free(result2);
    jsonrepair_options_free(opts);
}

void example_aggressive_truncation() {
    print_separator("Aggressive Truncation Fix");
    
    const char* input = "{a: 1, b: 'incomplete string";
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_aggressive_truncation_fix(opts, true);
    
    char* result = jsonrepair_repair_with_options(input, opts);
    
    printf("Input:  %s\n", input);
    printf("Output: %s\n", result);
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
}

void example_ndjson_streaming() {
    print_separator("NDJSON Streaming with Aggregation");
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_stream_ndjson_aggregate(opts, true);
    
    JsonRepairStream* stream = jsonrepair_stream_new(opts);
    
    const char* lines[] = {
        "{a: 1}\n",
        "{b: 2}\n",
        "{c: 3}\n",
        NULL
    };
    
    printf("Pushing NDJSON lines:\n");
    for (int i = 0; lines[i] != NULL; i++) {
        printf("  Push: %s", lines[i]);
        char* out = jsonrepair_stream_push(stream, lines[i]);
        if (out) {
            printf("  Got: %s\n", out);
            jsonrepair_free(out);
        }
    }
    
    printf("\nFlushing stream:\n");
    char* final = jsonrepair_stream_flush(stream);
    if (final) {
        printf("  Result: %s\n", final);
        jsonrepair_free(final);
    }
    
    jsonrepair_stream_free(stream);
    jsonrepair_options_free(opts);
}

void example_combined_options() {
    print_separator("Combined Options");
    
    const char* input = "```json\n{\n  name: '中文',  # User name\n  active: True,\n  score: .95\n}\n```";
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_ensure_ascii(opts, true);
    jsonrepair_options_set_allow_python_keywords(opts, true);
    jsonrepair_options_set_tolerate_hash_comments(opts, true);
    jsonrepair_options_set_fenced_code_blocks(opts, true);
    jsonrepair_options_set_number_tolerance_leading_dot(opts, true);
    jsonrepair_options_set_python_style_separators(opts, true);
    
    char* result = jsonrepair_repair_with_options(input, opts);
    
    printf("Input:\n%s\n\n", input);
    printf("Output:\n%s\n", result);
    
    jsonrepair_free(result);
    jsonrepair_options_free(opts);
}

void example_streaming_with_errors() {
    print_separator("Streaming with Error Handling");
    
    JsonRepairStream* stream = jsonrepair_stream_new(NULL);
    JsonRepairError error = {0};
    
    const char* chunks[] = {"{a:", "1,", "b:", "2}", NULL};
    
    for (int i = 0; chunks[i] != NULL; i++) {
        printf("Push: %s\n", chunks[i]);
        
        char* out = jsonrepair_stream_push_ex(stream, chunks[i], &error);
        
        if (error.code != OK) {
            printf("  Error %d: %s\n", error.code, error.message);
            free(error.message);
            error.message = NULL;
        } else if (out) {
            printf("  Got: %s\n", out);
            jsonrepair_free(out);
        } else {
            printf("  (buffering...)\n");
        }
    }
    
    char* tail = jsonrepair_stream_flush_ex(stream, &error);
    if (tail) {
        printf("Flush: %s\n", tail);
        jsonrepair_free(tail);
    }
    
    jsonrepair_stream_free(stream);
}

int main() {
    printf("jsonrepair Advanced C API Examples\n");
    printf("===================================\n");
    printf("Version: %s\n", jsonrepair_version());
    
    // Basic features
    example_python_keywords();
    example_hash_comments();
    example_fenced_code_blocks();
    example_undefined_repair();
    example_normalize_nonfinite();
    
    // Number handling
    example_number_tolerance();
    
    // Formatting
    example_python_style_separators();
    
    // Advanced features
    example_aggressive_truncation();
    example_ndjson_streaming();
    example_combined_options();
    example_streaming_with_errors();
    
    printf("\n=== All Examples Completed! ===\n");
    return 0;
}

