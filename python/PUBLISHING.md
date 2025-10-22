# Publishing Guide for jsonrepair-rs Python Package

This guide explains how to publish the `jsonrepair-rs` Python package to PyPI.

## 📋 Overview

The Python package uses **independent versioning** from the Rust crate:
- **Rust crate**: Published to crates.io with tags like `v0.1.0`
- **Python package**: Published to PyPI with tags like `py-v0.1.0`

This allows the Python bindings to have their own release cycle.

## 🏗️ Architecture

### Build System
- **Maturin**: Builds Python wheels from Rust code using PyO3
- **GitHub Actions**: Automates cross-platform wheel building and publishing
- **PyPI Trusted Publishing**: Secure publishing without API tokens (recommended)

### Supported Platforms
- **Linux**: x86_64, aarch64 (manylinux)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x64
- **Python versions**: 3.8, 3.9, 3.10, 3.11, 3.12, 3.13, 3.14

## 🚀 Publishing Methods

### Method 1: Automated Release via GitHub Actions (Recommended)

This is the primary method for production releases.

#### Prerequisites

1. **Set up PyPI Trusted Publishing** (one-time setup):
   
   a. Go to https://pypi.org/manage/account/publishing/
   
   b. Add a new publisher with:
      - **PyPI Project Name**: `jsonrepair-rs`
      - **Owner**: `Latias94` (your GitHub username)
      - **Repository name**: `jsonrepair`
      - **Workflow name**: `python-release.yml`
      - **Environment name**: (leave empty)
   
   c. If the project doesn't exist yet, you'll need to do a manual first release (see Method 2)

2. **Verify GitHub Actions permissions**:
   - Go to repository Settings → Actions → General
   - Ensure "Read and write permissions" is enabled for workflows

#### Release Steps

1. **Update version and changelog**:
   ```bash
   # Update version in python/pyproject.toml
   # Update version in python/python/jsonrepair/__init__.py
   # Update CHANGELOG.md with release notes
   ```

2. **Commit changes**:
   ```bash
   git add python/pyproject.toml python/python/jsonrepair/__init__.py CHANGELOG.md
   git commit -m "chore(python): bump version to 0.1.0"
   git push origin main
   ```

3. **Create and push tag**:
   ```bash
   # For stable release
   git tag py-v0.1.0
   git push origin py-v0.1.0
   
   # For pre-release (will publish to TestPyPI)
   git tag py-v0.1.0-beta.1
   git push origin py-v0.1.0-beta.1
   ```

4. **Monitor the release**:
   - Go to https://github.com/Latias94/jsonrepair/actions
   - Watch the "Python Package Release" workflow
   - It will:
     - Run tests on all platforms
     - Build wheels for Linux, macOS, Windows
     - Build source distribution (sdist)
     - Publish to PyPI
     - Create GitHub release with artifacts

5. **Verify the release**:
   ```bash
   # Check PyPI
   pip install jsonrepair-rs==0.1.0

   # Test it works
   python -c "import jsonrepair; print(jsonrepair.__version__)"
   ```

### Method 2: Manual Release (First Release or Testing)

Use this for the first release or when you need manual control.

#### Prerequisites

1. **Install tools**:
   ```bash
   pip install maturin twine
   ```

2. **Set up PyPI account**:
   - Create account at https://pypi.org/
   - Generate API token at https://pypi.org/manage/account/token/
   - Save token securely

#### Release Steps

1. **Update version**:
   ```bash
   cd python
   
   # Edit pyproject.toml
   # Change: version = "0.1.0"
   
   # Edit python/jsonrepair/__init__.py
   # Change: __version__ = "0.1.0"
   ```

2. **Build wheels locally** (optional, for testing):
   ```bash
   cd python
   
   # Build for current platform
   maturin build --release
   
   # Wheels will be in target/wheels/
   ls -lh target/wheels/
   ```

3. **Build and publish to TestPyPI** (recommended first):
   ```bash
   cd python
   
   # Build and publish to TestPyPI
   maturin publish --repository testpypi
   
   # Test installation
   pip install --index-url https://test.pypi.org/simple/ jsonrepair-rs==0.1.0
   ```

4. **Publish to PyPI**:
   ```bash
   cd python
   
   # Build and publish
   maturin publish
   
   # Or if you built wheels separately:
   twine upload target/wheels/*
   ```

5. **Create Git tag**:
   ```bash
   git tag py-v0.1.0
   git push origin py-v0.1.0
   ```

### Method 3: Build Multi-Platform Wheels Locally

For advanced users who want to build all platform wheels locally.

#### Using Docker for Linux wheels:

```bash
cd python

# Build manylinux wheels
docker run --rm -v $(pwd):/io \
  ghcr.io/pyo3/maturin \
  build --release --manylinux 2014

# Build for aarch64
docker run --rm -v $(pwd):/io \
  --platform linux/arm64 \
  ghcr.io/pyo3/maturin \
  build --release --manylinux 2014
```

#### Using cross-compilation:

```bash
# Install cross-compilation tools
rustup target add aarch64-unknown-linux-gnu

# Build for specific target
maturin build --release --target aarch64-unknown-linux-gnu
```

## 📝 Version Management

### Version Number Format

Follow [Semantic Versioning](https://semver.org/):
- **Major.Minor.Patch**: `0.1.0`
- **Pre-release**: `0.1.0-beta.1`, `0.1.0-rc.1`

### Tag Format

- **Stable release**: `py-v0.1.0`
- **Pre-release**: `py-v0.1.0-beta.1`
- **Note**: The `py-` prefix distinguishes Python releases from Rust crate releases

### Files to Update

When bumping version, update these files:

1. **`python/pyproject.toml`**:
   ```toml
   [project]
   version = "0.1.0"
   ```

2. **`python/python/jsonrepair/__init__.py`**:
   ```python
   __version__ = "0.1.0"
   ```

3. **`CHANGELOG.md`**:
   ```markdown
   ## [0.1.0] - 2024-01-15
   ### Added
   - Initial Python package release
   ```

## 🔍 Pre-Release Checklist

Before creating a release tag:

- [ ] All tests pass locally: `cd python && pytest tests/`
- [ ] Version updated in `pyproject.toml` and `__init__.py`
- [ ] CHANGELOG.md updated with release notes
- [ ] README.md is up to date
- [ ] Examples work: `python examples/basic_usage.py`
- [ ] Type stubs are correct: `mypy python/jsonrepair/__init__.pyi`
- [ ] Rust crate builds: `cargo build --release`
- [ ] Python package builds: `cd python && maturin build --release`

## 🐛 Troubleshooting

### Build Fails on GitHub Actions

**Problem**: Wheel build fails for specific platform

**Solution**:
1. Check the Actions log for specific error
2. Test locally with Docker (for Linux issues)
3. Check Rust toolchain compatibility
4. Verify PyO3 version compatibility

### PyPI Upload Fails

**Problem**: `File already exists` error

**Solution**:
- You cannot re-upload the same version
- Bump the version number (even for fixes: `0.1.0` → `0.1.1`)
- Or use a post-release version: `0.1.0.post1`

### Trusted Publishing Not Working

**Problem**: Authentication fails during publish

**Solution**:
1. Verify PyPI trusted publisher settings match exactly
2. Check workflow name is `python-release.yml`
3. Ensure workflow has `id-token: write` permission
4. For first release, use manual method to create the project

### Version Mismatch

**Problem**: Installed version doesn't match expected

**Solution**:
```bash
# Clear pip cache
pip cache purge

# Reinstall with no cache
pip install --no-cache-dir jsonrepair-rs==0.1.0

# Verify version
python -c "import jsonrepair; print(jsonrepair.__version__)"
```

## 📚 Additional Resources

- [Maturin Documentation](https://www.maturin.rs/)
- [PyO3 Guide](https://pyo3.rs/)
- [PyPI Trusted Publishing](https://docs.pypi.org/trusted-publishers/)
- [Python Packaging Guide](https://packaging.python.org/)
- [Semantic Versioning](https://semver.org/)

## 🔄 Release Workflow Summary

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Update Version                                           │
│    - pyproject.toml                                         │
│    - __init__.py                                            │
│    - CHANGELOG.md                                           │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Commit & Push                                            │
│    git commit -m "chore(python): bump version to X.Y.Z"     │
│    git push origin main                                     │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Create & Push Tag                                        │
│    git tag py-vX.Y.Z                                        │
│    git push origin py-vX.Y.Z                                │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. GitHub Actions (Automatic)                               │
│    ├─ Run tests (all platforms)                             │
│    ├─ Build wheels (Linux, macOS, Windows)                  │
│    ├─ Build sdist                                           │
│    ├─ Publish to PyPI                                       │
│    └─ Create GitHub Release                                 │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Verify                                                   │
│    pip install jsonrepair-rs==X.Y.Z                         │
│    python -c "import jsonrepair; print(jsonrepair.loads(    │
│        \"{name: 'test'}\"))"                                 │
└─────────────────────────────────────────────────────────────┘
```

## 📞 Support

If you encounter issues:
1. Check this guide's troubleshooting section
2. Review GitHub Actions logs
3. Open an issue at https://github.com/Latias94/jsonrepair/issues

