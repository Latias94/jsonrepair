"""
Test script for new Python binding features (without pytest dependency)
"""

def test_property_access():
    """Test RepairOptions property access"""
    print("Testing RepairOptions property access...")

    # This will fail if module not built, but shows the API
    try:
        import jsonrepair

        # Create options
        opts = jsonrepair.RepairOptions()

        # Test getters
        assert isinstance(opts.ensure_ascii, bool)
        assert isinstance(opts.tolerate_hash_comments, bool)
        assert isinstance(opts.repair_undefined, bool)
        assert isinstance(opts.allow_python_keywords, bool)
        assert isinstance(opts.fenced_code_blocks, bool)
        assert isinstance(opts.normalize_js_nonfinite, bool)
        assert isinstance(opts.stream_ndjson_aggregate, bool)
        assert isinstance(opts.logging, bool)

        # Test setters
        opts.ensure_ascii = True
        assert opts.ensure_ascii == True

        opts.tolerate_hash_comments = False
        assert opts.tolerate_hash_comments == False

        opts.logging = True
        assert opts.logging == True

        print("✅ Property access works!")
        return True

    except ImportError:
        print("⚠️  Module not built yet - run 'maturin develop' first")
        return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def test_options_with_functions():
    """Test using options with repair functions"""
    print("\nTesting options with functions...")

    try:
        import jsonrepair

        # Create options
        opts = jsonrepair.RepairOptions()
        opts.ensure_ascii = True

        # Test with repair_json
        result = jsonrepair.repair_json("{'name': '中文'}", options=opts)
        assert '\\u' in result, "Should contain Unicode escapes"

        # Test with loads
        opts2 = jsonrepair.RepairOptions()
        opts2.allow_python_keywords = True
        data = jsonrepair.loads("{active: True}", options=opts2)
        assert data['active'] == True

        print("✅ Options work with functions!")
        return True

    except ImportError:
        print("⚠️  Module not built yet")
        return False
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_streaming_with_options():
    """Test StreamRepairer with options"""
    print("\nTesting StreamRepairer with options...")

    try:
        import jsonrepair

        # Create options for NDJSON aggregation
        opts = jsonrepair.RepairOptions()
        opts.stream_ndjson_aggregate = True

        # Create repairer
        repairer = jsonrepair.StreamRepairer(options=opts)

        # Feed chunks
        repairer.push('{a: 1}\n')
        repairer.push('{b: 2}\n')
        result = repairer.flush()

        # Should aggregate to array
        assert result is not None
        assert '[' in result and ']' in result

        print("✅ StreamRepairer with options works!")
        return True

    except ImportError:
        print("⚠️  Module not built yet")
        return False
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_type_hints():
    """Test that type hints file exists"""
    print("\nChecking type hints file...")

    import os
    pyi_path = os.path.join(os.path.dirname(__file__), 'python', 'jsonrepair', '__init__.pyi')

    if os.path.exists(pyi_path):
        print(f"✅ Type hints file exists: {pyi_path}")

        # Check content
        with open(pyi_path, encoding='utf-8') as f:
            content = f.read()

        # Verify key signatures
        assert 'class RepairOptions:' in content
        assert 'class StreamRepairer:' in content
        assert 'def repair_json(' in content
        assert 'def loads(' in content
        assert '@property' in content
        assert 'def ensure_ascii(self) -> bool:' in content

        print("✅ Type hints file has correct content!")
        return True
    else:
        print(f"❌ Type hints file not found: {pyi_path}")
        return False

def main():
    """Run all tests"""
    print("=" * 60)
    print("Testing New Python Binding Features")
    print("=" * 60)

    results = []

    # Test type hints (doesn't require built module)
    results.append(test_type_hints())

    # Test runtime features (requires built module)
    results.append(test_property_access())
    results.append(test_options_with_functions())
    results.append(test_streaming_with_options())

    print("\n" + "=" * 60)
    passed = sum(results)
    total = len(results)
    print(f"Results: {passed}/{total} tests passed")

    if passed == total:
        print("🎉 All tests passed!")
    else:
        print("⚠️  Some tests failed or skipped")
        print("\nTo run runtime tests, build the module first:")
        print("  cd python")
        print("  maturin develop --release")

    print("=" * 60)

if __name__ == '__main__':
    main()

