"""
Basic usage examples for jsonrepair Python bindings
"""

import jsonrepair


def example_basic_repair():
    """Basic JSON repair"""
    print("=== Basic Repair ===")
    broken = "{name: 'John', age: 30,}"
    repaired = jsonrepair.repair_json(broken)
    print(f"Input:  {broken}")
    print(f"Output: {repaired}")
    print()

def example_loads():
    """Parse broken JSON directly"""
    print("=== Parse with loads() ===")
    broken = "{name: 'John', age: 30}"
    data = jsonrepair.loads(broken)
    print(f"Input:  {broken}")
    print(f"Output: {data}")
    print(f"Type:   {type(data)}")
    print()

def example_comments():
    """Handle comments in JSON"""
    print("=== Handle Comments ===")
    broken = """
    {
        // This is a comment
        "name": "John",
        /* Block comment */
        "age": 30
    }
    """
    data = jsonrepair.loads(broken)
    print(f"Input:  {broken}")
    print(f"Output: {data}")
    print()

def example_python_keywords():
    """Handle Python keywords"""
    print("=== Python Keywords ===")
    broken = "{active: True, inactive: False, value: None}"
    data = jsonrepair.loads(broken)
    print(f"Input:  {broken}")
    print(f"Output: {data}")
    print()

def example_incomplete():
    """Handle incomplete JSON"""
    print("=== Incomplete JSON ===")
    broken = '{"name": "John", "age": 30'
    data = jsonrepair.loads(broken)
    print(f"Input:  {broken}")
    print(f"Output: {data}")
    print()

def example_fenced_code():
    """Handle markdown fenced code blocks"""
    print("=== Fenced Code Block ===")
    broken = """
    ```json
    {
        "name": "John",
        "age": 30
    }
    ```
    """
    data = jsonrepair.loads(broken)
    print(f"Input:  {broken}")
    print(f"Output: {data}")
    print()

def example_unicode():
    """Handle Unicode characters"""
    print("=== Unicode Support ===")
    broken = "{'name': '统一码', 'emoji': '😀'}"
    data = jsonrepair.loads(broken)
    print(f"Input:  {broken}")
    print(f"Output: {data}")
    print()

def example_nested():
    """Handle nested structures"""
    print("=== Nested Structures ===")
    broken = """
    {
        name: 'John',
        address: {
            street: '123 Main St',
            city: 'New York'
        },
        hobbies: ['reading', 'swimming']
    }
    """
    data = jsonrepair.loads(broken)
    print(f"Input:  {broken}")
    print(f"Output: {data}")
    print()

def example_return_objects():
    """Use return_objects parameter"""
    print("=== Return Objects ===")
    broken = "{name: 'John', age: 30}"

    # Return string
    string_result = jsonrepair.repair_json(broken)
    print(f"String result: {string_result} (type: {type(string_result)})")

    # Return object
    object_result = jsonrepair.repair_json(broken, return_objects=True)
    print(f"Object result: {object_result} (type: {type(object_result)})")
    print()

def example_ensure_ascii():
    """Use ensure_ascii parameter"""
    print("=== Ensure ASCII ===")
    broken = "{'name': '统一码'}"

    # Without ensure_ascii
    normal = jsonrepair.repair_json(broken)
    print(f"Normal:       {normal}")

    # With ensure_ascii
    ascii_only = jsonrepair.repair_json(broken, ensure_ascii=True)
    print(f"ASCII only:   {ascii_only}")
    print()

if __name__ == '__main__':
    example_basic_repair()
    example_loads()
    example_comments()
    example_python_keywords()
    example_incomplete()
    example_fenced_code()
    example_unicode()
    example_nested()
    example_return_objects()
    example_ensure_ascii()

    print("✅ All examples completed successfully!")

