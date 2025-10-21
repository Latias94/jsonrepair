/**
 * Basic C example for jsonrepair library
 * 
 * Compile:
 *   gcc -o basic basic.c -L../../target/release -ljsonrepair
 * 
 * Run (Linux/macOS):
 *   LD_LIBRARY_PATH=../../target/release ./basic
 * 
 * Run (Windows):
 *   set PATH=%PATH%;..\..\target\release
 *   basic.exe
 */

#include "../../include/jsonrepair.h"
#include <stdio.h>
#include <stdlib.h>

void example_simple() {
    printf("=== Simple Repair ===\n");
    
    const char* broken = "{a:1, b:'hello'}";
    char* repaired = jsonrepair_repair(broken);
    
    if (repaired) {
        printf("Input:  %s\n", broken);
        printf("Output: %s\n", repaired);
        jsonrepair_free(repaired);
    } else {
        printf("Repair failed!\n");
    }
    printf("\n");
}

void example_with_options() {
    printf("=== With Options ===\n");
    
    const char* broken = "{name: '统一码', age: 30}";
    
    JsonRepairOptions* opts = jsonrepair_options_new();
    jsonrepair_options_set_ensure_ascii(opts, true);
    
    char* repaired = jsonrepair_repair_with_options(broken, opts);
    
    if (repaired) {
        printf("Input:  %s\n", broken);
        printf("Output: %s\n", repaired);
        jsonrepair_free(repaired);
    }
    
    jsonrepair_options_free(opts);
    printf("\n");
}

void example_error_handling() {
    printf("=== Error Handling ===\n");
    
    const char* broken = "{a:1, b:";  // Incomplete JSON
    JsonRepairError error = {0};
    
    char* repaired = jsonrepair_repair_ex(broken, NULL, &error);
    
    if (!repaired) {
        printf("Input:  %s\n", broken);
        printf("Error:  Code %d at position %zu\n", error.code, error.position);
        printf("        %s\n", error.message);
        free(error.message);
    } else {
        printf("Unexpectedly succeeded: %s\n", repaired);
        jsonrepair_free(repaired);
    }
    printf("\n");
}

void example_streaming() {
    printf("=== Streaming ===\n");
    
    const char* chunks[] = {"{a:", "1}", "{b:", "2}", NULL};
    
    JsonRepairStream* stream = jsonrepair_stream_new(NULL);
    
    for (int i = 0; chunks[i] != NULL; i++) {
        printf("Push: %s\n", chunks[i]);
        char* out = jsonrepair_stream_push(stream, chunks[i]);
        if (out) {
            printf("  -> Got: %s\n", out);
            jsonrepair_free(out);
        } else {
            printf("  -> (buffering...)\n");
        }
    }
    
    char* tail = jsonrepair_stream_flush(stream);
    if (tail) {
        printf("Flush -> %s\n", tail);
        jsonrepair_free(tail);
    }
    
    jsonrepair_stream_free(stream);
    printf("\n");
}

void example_version() {
    printf("=== Version Info ===\n");
    printf("jsonrepair version: %s\n", jsonrepair_version());
    printf("\n");
}

int main() {
    printf("jsonrepair C API Examples\n");
    printf("=========================\n\n");
    
    example_version();
    example_simple();
    example_with_options();
    example_error_handling();
    example_streaming();
    
    printf("All examples completed!\n");
    return 0;
}

