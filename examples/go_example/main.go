package main

/*
#cgo LDFLAGS: -L../../target/release -ljsonrepair
#include "../../include/jsonrepair.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"unsafe"
)

// RepairJSON repairs a broken JSON string using default options
func RepairJSON(input string) (string, error) {
	cInput := C.CString(input)
	defer C.free(unsafe.Pointer(cInput))

	cResult := C.jsonrepair_repair(cInput)
	if cResult == nil {
		return "", fmt.Errorf("repair failed")
	}
	defer C.jsonrepair_free(cResult)

	return C.GoString(cResult), nil
}

// RepairJSONWithOptions repairs JSON with custom options
func RepairJSONWithOptions(input string, ensureASCII bool) (string, error) {
	cInput := C.CString(input)
	defer C.free(unsafe.Pointer(cInput))

	opts := C.jsonrepair_options_new()
	defer C.jsonrepair_options_free(opts)

	C.jsonrepair_options_set_ensure_ascii(opts, C.bool(ensureASCII))

	cResult := C.jsonrepair_repair_with_options(cInput, opts)
	if cResult == nil {
		return "", fmt.Errorf("repair failed")
	}
	defer C.jsonrepair_free(cResult)

	return C.GoString(cResult), nil
}

// StreamRepairer wraps the C streaming API
type StreamRepairer struct {
	stream *C.JsonRepairStream
}

// NewStreamRepairer creates a new streaming repairer
func NewStreamRepairer() *StreamRepairer {
	return &StreamRepairer{
		stream: C.jsonrepair_stream_new(nil),
	}
}

// Push pushes a chunk and returns completed JSON if any
func (s *StreamRepairer) Push(chunk string) (string, error) {
	cChunk := C.CString(chunk)
	defer C.free(unsafe.Pointer(cChunk))

	cResult := C.jsonrepair_stream_push(s.stream, cChunk)
	if cResult == nil {
		return "", nil // No complete value yet
	}
	defer C.jsonrepair_free(cResult)

	return C.GoString(cResult), nil
}

// Flush flushes remaining data
func (s *StreamRepairer) Flush() (string, error) {
	cResult := C.jsonrepair_stream_flush(s.stream)
	if cResult == nil {
		return "", nil
	}
	defer C.jsonrepair_free(cResult)

	return C.GoString(cResult), nil
}

// Close frees the stream
func (s *StreamRepairer) Close() {
	if s.stream != nil {
		C.jsonrepair_stream_free(s.stream)
		s.stream = nil
	}
}

func main() {
	fmt.Println("jsonrepair Go Example")
	fmt.Println("=====================\n")

	// Example 1: Simple repair
	fmt.Println("=== Simple Repair ===")
	broken := "{a:1, b:'hello'}"
	repaired, err := RepairJSON(broken)
	if err != nil {
		fmt.Printf("Error: %v\n", err)
	} else {
		fmt.Printf("Input:  %s\n", broken)
		fmt.Printf("Output: %s\n", repaired)
	}
	fmt.Println()

	// Example 2: With options
	fmt.Println("=== With Options (ensure_ascii) ===")
	broken = "{name: '统一码'}"
	repaired, err = RepairJSONWithOptions(broken, true)
	if err != nil {
		fmt.Printf("Error: %v\n", err)
	} else {
		fmt.Printf("Input:  %s\n", broken)
		fmt.Printf("Output: %s\n", repaired)
	}
	fmt.Println()

	// Example 3: Streaming
	fmt.Println("=== Streaming ===")
	stream := NewStreamRepairer()
	defer stream.Close()

	chunks := []string{"{a:", "1}", "{b:", "2}"}
	for _, chunk := range chunks {
		fmt.Printf("Push: %s\n", chunk)
		out, err := stream.Push(chunk)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
		} else if out != "" {
			fmt.Printf("  -> Got: %s\n", out)
		} else {
			fmt.Printf("  -> (buffering...)\n")
		}
	}

	tail, err := stream.Flush()
	if err != nil {
		fmt.Printf("Error: %v\n", err)
	} else if tail != "" {
		fmt.Printf("Flush -> %s\n", tail)
	}
	fmt.Println()

	// Example 4: Version
	fmt.Println("=== Version ===")
	version := C.GoString(C.jsonrepair_version())
	fmt.Printf("jsonrepair version: %s\n", version)
	fmt.Println()

	fmt.Println("All examples completed!")
}

