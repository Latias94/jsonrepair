"""
Example: Use logging variants to inspect repair actions.

Run with:
    python -m jsonrepair.examples.logging_example
or:
    python examples/logging_example.py
"""

import jsonrepair


def main():
    broken = "{a:1, /*c*/ b:2, addr: {city: New York}}"
    # Return JSON string and log
    s, log = jsonrepair.repair_json_with_log(broken)
    print("Repaired string:\n", s)
    print("Log entries:")
    for i, e in enumerate(log):
        print(f"[{i}] {e.get('message')} | context: {e.get('context')} | path: {e.get('path')}")

    # Return Python object and log
    obj, log2 = jsonrepair.loads_with_log(broken)
    print("\nRepaired object:", obj)
    print("Log2 entries:", len(log2))


if __name__ == "__main__":
    main()

