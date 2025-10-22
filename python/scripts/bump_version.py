#!/usr/bin/env python3
"""
Script to bump version for the Python package.

Usage:
    python scripts/bump_version.py 0.1.0
    python scripts/bump_version.py 0.2.0-beta.1
"""

import re
import sys
from pathlib import Path


def update_file(file_path: Path, pattern: str, replacement: str) -> bool:
    """Update version in a file using regex pattern."""
    try:
        content = file_path.read_text(encoding='utf-8')
        new_content = re.sub(pattern, replacement, content)

        if content != new_content:
            file_path.write_text(new_content, encoding='utf-8')
            print(f"✓ Updated {file_path}")
            return True
        else:
            print(f"⚠ No changes in {file_path}")
            return False
    except Exception as e:
        print(f"✗ Error updating {file_path}: {e}")
        return False


def validate_version(version: str) -> bool:
    """Validate version string format."""
    # Semantic versioning pattern with optional pre-release
    pattern = r'^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$'
    return bool(re.match(pattern, version))


def bump_version(new_version: str):
    """Bump version in all relevant files."""
    if not validate_version(new_version):
        print(f"✗ Invalid version format: {new_version}")
        print("  Expected format: X.Y.Z or X.Y.Z-pre.N")
        print("  Examples: 0.1.0, 1.2.3, 0.1.0-beta.1")
        sys.exit(1)

    print(f"Bumping version to: {new_version}\n")

    # Get project root (parent of python directory)
    script_dir = Path(__file__).parent
    python_dir = script_dir.parent

    # Files to update
    updates = [
        # pyproject.toml
        {
            'file': python_dir / 'pyproject.toml',
            'pattern': r'version = "[^"]+"',
            'replacement': f'version = "{new_version}"'
        },
        # __init__.py
        {
            'file': python_dir / 'python' / 'jsonrepair' / '__init__.py',
            'pattern': r'__version__ = "[^"]+"',
            'replacement': f'__version__ = "{new_version}"'
        },
    ]

    success_count = 0
    for update in updates:
        if update_file(update['file'], update['pattern'], update['replacement']):
            success_count += 1

    print(f"\n✓ Updated {success_count}/{len(updates)} files")

    # Suggest next steps
    print("\n" + "="*60)
    print("Next steps:")
    print("="*60)
    print(f"1. Update CHANGELOG.md with release notes for v{new_version}")
    print("2. Review changes: git diff")
    print(f"3. Commit: git commit -am 'chore(python): release v{new_version}'")
    print(f"4. Tag: git tag py-v{new_version}")
    print(f"5. Push: git push origin main py-v{new_version}")
    print("="*60)


def show_current_version():
    """Show current version from pyproject.toml."""
    script_dir = Path(__file__).parent
    python_dir = script_dir.parent
    pyproject = python_dir / 'pyproject.toml'

    try:
        content = pyproject.read_text(encoding='utf-8')
        match = re.search(r'version = "([^"]+)"', content)
        if match:
            print(f"Current version: {match.group(1)}")
        else:
            print("Could not find version in pyproject.toml")
    except Exception as e:
        print(f"Error reading version: {e}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python bump_version.py <new_version>")
        print("\nExamples:")
        print("  python bump_version.py 0.1.0")
        print("  python bump_version.py 0.2.0-beta.1")
        print("  python bump_version.py 1.0.0-rc.1")
        print()
        show_current_version()
        sys.exit(1)

    new_version = sys.argv[1]
    bump_version(new_version)


if __name__ == '__main__':
    main()

