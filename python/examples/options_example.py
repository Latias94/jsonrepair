"""
Examples of using RepairOptions with property access
"""

import jsonrepair

def example_basic_options():
    """Create options and use them"""
    print("=== Basic Options ===")
    
    # Create options with constructor
    opts = jsonrepair.RepairOptions(
        ensure_ascii=True,
        allow_python_keywords=True,
        tolerate_hash_comments=True
    )
    
    broken = "{'name': '统一码', 'active': True}"
    result = jsonrepair.repair_json(broken, options=opts)
    print(f"Input:  {broken}")
    print(f"Output: {result}")
    print()

def example_property_access():
    """Modify options using property access"""
    print("=== Property Access ===")
    
    # Create default options
    opts = jsonrepair.RepairOptions()
    
    # Modify using properties
    opts.ensure_ascii = True
    opts.tolerate_hash_comments = False
    opts.fenced_code_blocks = True
    
    print(f"Options: {opts}")
    print(f"ensure_ascii: {opts.ensure_ascii}")
    print(f"tolerate_hash_comments: {opts.tolerate_hash_comments}")
    print()

def example_reuse_options():
    """Reuse options for multiple repairs"""
    print("=== Reuse Options ===")
    
    # Create options once
    opts = jsonrepair.RepairOptions(
        allow_python_keywords=True,
        normalize_js_nonfinite=True
    )
    
    # Use for multiple repairs
    inputs = [
        "{active: True, value: None}",
        "{number: NaN, infinity: Infinity}",
        "{name: 'test', undefined: undefined}"
    ]
    
    for broken in inputs:
        result = jsonrepair.loads(broken, options=opts)
        print(f"Input:  {broken}")
        print(f"Output: {result}")
        print()

def example_streaming_with_options():
    """Use options with streaming API"""
    print("=== Streaming with Options ===")
    
    # Create options for NDJSON aggregation
    opts = jsonrepair.RepairOptions(stream_ndjson_aggregate=True)
    
    # Create repairer with options
    repairer = jsonrepair.StreamRepairer(options=opts)
    
    # Feed NDJSON chunks
    chunks = [
        '{name: "Alice"}\n',
        '{name: "Bob"}\n',
        '{name: "Charlie"}'
    ]
    
    for chunk in chunks:
        if output := repairer.push(chunk):
            print(f"Chunk output: {output}")
    
    if final := repairer.flush():
        print(f"Final output: {final}")
    print()

def example_logging_options():
    """Use logging options"""
    print("=== Logging Options ===")
    
    # Enable logging
    opts = jsonrepair.RepairOptions(logging=True)
    
    broken = "{name: 'John', age: 30,}"
    result, log = jsonrepair.repair_json_with_log(broken, options=opts)
    
    print(f"Input:  {broken}")
    print(f"Output: {result}")
    print(f"\nRepair log ({len(log)} entries):")
    for entry in log:
        print(f"  - Position {entry['position']}: {entry['message']}")
        if entry['context']:
            print(f"    Context: {entry['context']}")
    print()

def example_modify_existing_options():
    """Modify existing options"""
    print("=== Modify Existing Options ===")
    
    # Start with default options
    opts = jsonrepair.RepairOptions()
    
    # First repair with defaults
    broken = "{'name': '中文'}"
    result1 = jsonrepair.repair_json(broken, options=opts)
    print(f"Default: {result1}")
    
    # Modify option
    opts.ensure_ascii = True
    
    # Second repair with modified options
    result2 = jsonrepair.repair_json(broken, options=opts)
    print(f"ASCII:   {result2}")
    print()

def example_all_boolean_options():
    """Show all boolean options"""
    print("=== All Boolean Options ===")
    
    opts = jsonrepair.RepairOptions()
    
    # Access all boolean properties
    print(f"ensure_ascii:              {opts.ensure_ascii}")
    print(f"tolerate_hash_comments:    {opts.tolerate_hash_comments}")
    print(f"repair_undefined:          {opts.repair_undefined}")
    print(f"allow_python_keywords:     {opts.allow_python_keywords}")
    print(f"fenced_code_blocks:        {opts.fenced_code_blocks}")
    print(f"normalize_js_nonfinite:    {opts.normalize_js_nonfinite}")
    print(f"stream_ndjson_aggregate:   {opts.stream_ndjson_aggregate}")
    print(f"logging:                   {opts.logging}")
    print()

if __name__ == '__main__':
    example_basic_options()
    example_property_access()
    example_reuse_options()
    example_streaming_with_options()
    example_logging_options()
    example_modify_existing_options()
    example_all_boolean_options()
    
    print("✅ All options examples completed successfully!")

