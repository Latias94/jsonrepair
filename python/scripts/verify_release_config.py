#!/usr/bin/env python3
"""
Verify that the Python package release configuration is correct.

This script checks:
- Version consistency across files
- Required files exist
- Configuration files are valid
- GitHub Actions workflows are present
"""

import sys
import re
from pathlib import Path
from typing import List, Tuple


class Colors:
    """ANSI color codes for terminal output."""
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    RESET = '\033[0m'
    BOLD = '\033[1m'


def print_success(msg: str):
    print(f"{Colors.GREEN}✓{Colors.RESET} {msg}")


def print_error(msg: str):
    print(f"{Colors.RED}✗{Colors.RESET} {msg}")


def print_warning(msg: str):
    print(f"{Colors.YELLOW}⚠{Colors.RESET} {msg}")


def print_info(msg: str):
    print(f"{Colors.BLUE}ℹ{Colors.RESET} {msg}")


def print_header(msg: str):
    print(f"\n{Colors.BOLD}{msg}{Colors.RESET}")
    print("=" * 60)


def get_version_from_file(file_path: Path, pattern: str) -> str | None:
    """Extract version from a file using regex pattern."""
    try:
        content = file_path.read_text(encoding='utf-8')
        match = re.search(pattern, content)
        return match.group(1) if match else None
    except Exception as e:
        print_error(f"Error reading {file_path}: {e}")
        return None


def check_file_exists(file_path: Path, description: str) -> bool:
    """Check if a file exists."""
    if file_path.exists():
        print_success(f"{description}: {file_path.name}")
        return True
    else:
        print_error(f"{description} not found: {file_path}")
        return False


def check_version_consistency(python_dir: Path) -> Tuple[bool, str | None]:
    """Check that version is consistent across files."""
    print_header("Version Consistency Check")
    
    # Get versions from different files
    pyproject_version = get_version_from_file(
        python_dir / 'pyproject.toml',
        r'version = "([^"]+)"'
    )
    
    init_version = get_version_from_file(
        python_dir / 'python' / 'jsonrepair' / '__init__.py',
        r'__version__ = "([^"]+)"'
    )
    
    if not pyproject_version:
        print_error("Could not find version in pyproject.toml")
        return False, None
    
    if not init_version:
        print_error("Could not find version in __init__.py")
        return False, None
    
    print_info(f"pyproject.toml version: {pyproject_version}")
    print_info(f"__init__.py version: {init_version}")
    
    if pyproject_version == init_version:
        print_success(f"Versions are consistent: {pyproject_version}")
        return True, pyproject_version
    else:
        print_error("Version mismatch!")
        print_warning(f"Run: python scripts/bump_version.py {pyproject_version}")
        return False, None


def check_required_files(python_dir: Path, project_root: Path) -> bool:
    """Check that all required files exist."""
    print_header("Required Files Check")
    
    required_files = [
        (python_dir / 'pyproject.toml', 'pyproject.toml'),
        (python_dir / 'Cargo.toml', 'Cargo.toml'),
        (python_dir / 'README.md', 'README.md'),
        (python_dir / 'python' / 'jsonrepair' / '__init__.py', '__init__.py'),
        (python_dir / 'python' / 'jsonrepair' / '__init__.pyi', '__init__.pyi (type stubs)'),
        (python_dir / 'python' / 'jsonrepair' / 'py.typed', 'py.typed'),
        (python_dir / 'src' / 'lib.rs', 'src/lib.rs'),
        (python_dir / 'PUBLISHING.md', 'PUBLISHING.md'),
        (python_dir / 'RELEASE_CHECKLIST.md', 'RELEASE_CHECKLIST.md'),
        (project_root / '.github' / 'workflows' / 'python-release.yml', 'python-release.yml'),
        (project_root / '.github' / 'workflows' / 'python-ci.yml', 'python-ci.yml'),
    ]
    
    all_exist = True
    for file_path, description in required_files:
        if not check_file_exists(file_path, description):
            all_exist = False
    
    return all_exist


def check_pyproject_config(python_dir: Path) -> bool:
    """Check pyproject.toml configuration."""
    print_header("pyproject.toml Configuration Check")
    
    pyproject_path = python_dir / 'pyproject.toml'
    try:
        content = pyproject_path.read_text(encoding='utf-8')

        checks = [
            (r'\[build-system\]', 'Build system section'),
            (r'requires = \["maturin', 'Maturin requirement'),
            (r'\[project\]', 'Project section'),
            (r'name = "jsonrepair-rs"', 'Package name'),
            (r'requires-python = ">=3\.8"', 'Python version requirement'),
            (r'\[tool\.maturin\]', 'Maturin configuration'),
            (r'module-name = "jsonrepair\._jsonrepair"', 'Module name'),
        ]
        
        all_passed = True
        for pattern, description in checks:
            if re.search(pattern, content):
                print_success(description)
            else:
                print_error(f"{description} not found")
                all_passed = False
        
        return all_passed
    except Exception as e:
        print_error(f"Error reading pyproject.toml: {e}")
        return False


def check_github_workflows(project_root: Path) -> bool:
    """Check GitHub Actions workflows."""
    print_header("GitHub Actions Workflows Check")
    
    workflows_dir = project_root / '.github' / 'workflows'
    
    # Check python-release.yml
    release_workflow = workflows_dir / 'python-release.yml'
    if release_workflow.exists():
        content = release_workflow.read_text(encoding='utf-8')
        
        checks = [
            (r"py-v\[0-9\]", 'Tag trigger pattern'),
            (r'PyO3/maturin-action', 'Maturin action'),
            (r'id-token: write', 'Trusted publishing permission'),
            (r'linux:', 'Linux build job'),
            (r'macos:', 'macOS build job'),
            (r'windows:', 'Windows build job'),
            (r'sdist:', 'Source distribution job'),
            (r'publish:', 'Publish job'),
        ]
        
        all_passed = True
        for pattern, description in checks:
            if re.search(pattern, content):
                print_success(f"Release workflow: {description}")
            else:
                print_error(f"Release workflow: {description} not found")
                all_passed = False
    else:
        print_error("python-release.yml not found")
        all_passed = False
    
    # Check python-ci.yml
    ci_workflow = workflows_dir / 'python-ci.yml'
    if ci_workflow.exists():
        print_success("CI workflow exists")
    else:
        print_warning("python-ci.yml not found (optional but recommended)")
    
    return all_passed


def check_cargo_config(python_dir: Path) -> bool:
    """Check Cargo.toml configuration."""
    print_header("Cargo.toml Configuration Check")
    
    cargo_path = python_dir / 'Cargo.toml'
    try:
        content = cargo_path.read_text(encoding='utf-8')
        
        checks = [
            (r'name = "jsonrepair-python"', 'Package name'),
            (r'crate-type = \["cdylib"\]', 'Crate type'),
            (r'pyo3 = .*extension-module', 'PyO3 extension module'),
            (r'jsonrepair = \{ path = "\.\."', 'Parent crate dependency'),
        ]
        
        all_passed = True
        for pattern, description in checks:
            if re.search(pattern, content):
                print_success(description)
            else:
                print_error(f"{description} not found")
                all_passed = False
        
        return all_passed
    except Exception as e:
        print_error(f"Error reading Cargo.toml: {e}")
        return False


def main():
    """Run all verification checks."""
    # Get paths
    script_dir = Path(__file__).parent
    python_dir = script_dir.parent
    project_root = python_dir.parent
    
    print(f"{Colors.BOLD}Python Package Release Configuration Verification{Colors.RESET}")
    print(f"Python directory: {python_dir}")
    print(f"Project root: {project_root}")
    
    # Run checks
    checks = [
        ("Version Consistency", lambda: check_version_consistency(python_dir)),
        ("Required Files", lambda: check_required_files(python_dir, project_root)),
        ("pyproject.toml", lambda: check_pyproject_config(python_dir)),
        ("Cargo.toml", lambda: check_cargo_config(python_dir)),
        ("GitHub Workflows", lambda: check_github_workflows(project_root)),
    ]
    
    results = []
    for name, check_func in checks:
        result = check_func()
        # Handle tuple return (version check)
        if isinstance(result, tuple):
            result = result[0]
        results.append((name, result))
    
    # Summary
    print_header("Summary")
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for name, result in results:
        if result:
            print_success(f"{name}: PASSED")
        else:
            print_error(f"{name}: FAILED")
    
    print(f"\n{Colors.BOLD}Total: {passed}/{total} checks passed{Colors.RESET}")
    
    if passed == total:
        print_success("\nAll checks passed! Ready for release.")
        return 0
    else:
        print_error(f"\n{total - passed} check(s) failed. Please fix the issues above.")
        return 1


if __name__ == '__main__':
    sys.exit(main())

