# Python Package Release Checklist

Quick reference for releasing the `jsonrepair-rs` Python package.

## 🎯 Quick Release (Automated)

For experienced maintainers who have already set up PyPI Trusted Publishing:

```bash
# 1. Update version
vim python/pyproject.toml              # version = "X.Y.Z"
vim python/python/jsonrepair/__init__.py  # __version__ = "X.Y.Z"
vim CHANGELOG.md                       # Add release notes

# 2. Commit and push
git add python/pyproject.toml python/python/jsonrepair/__init__.py CHANGELOG.md
git commit -m "chore(python): release v0.1.0"
git push origin main

# 3. Create and push tag
git tag py-v0.1.0
git push origin py-v0.1.0

# 4. Wait for GitHub Actions to complete
# 5. Verify: pip install jsonrepair-rs==0.1.0
```

## ✅ Pre-Release Checklist

Run these checks before creating a release tag:

### Code Quality
- [ ] All tests pass: `cd python && pytest tests/ -v`
- [ ] Examples work: `python examples/basic_usage.py`
- [ ] Type stubs valid: `mypy python/jsonrepair/__init__.pyi --strict`
- [ ] Linting passes: `ruff check python/ tests/ examples/`
- [ ] Rust crate builds: `cargo build --release`
- [ ] Python package builds: `cd python && maturin build --release`

### Documentation
- [ ] README.md is up to date
- [ ] CHANGELOG.md has entry for this version
- [ ] Version numbers match in all files:
  - [ ] `python/pyproject.toml`
  - [ ] `python/python/jsonrepair/__init__.py`
- [ ] Examples are tested and working

### Testing
- [ ] Compatibility tests pass: `pytest tests/test_compatibility.py -v`
- [ ] All platforms tested (if possible):
  - [ ] Linux
  - [ ] macOS
  - [ ] Windows
- [ ] Multiple Python versions tested (3.8, 3.9, 3.10, 3.11, 3.12)

### Release Preparation
- [ ] PyPI Trusted Publishing configured (for first release)
- [ ] GitHub Actions permissions verified
- [ ] Release notes prepared in CHANGELOG.md
- [ ] No uncommitted changes: `git status`

## 📋 Version Bump Guide

### Files to Update

1. **`python/pyproject.toml`** (line 7):
   ```toml
   version = "0.1.0"  # Change this
   ```

2. **`python/python/jsonrepair/__init__.py`** (line 36):
   ```python
   __version__ = "0.1.0"  # Change this
   ```

3. **`CHANGELOG.md`**:
   ```markdown
   ## [0.1.0] - 2024-01-15
   
   ### Added
   - New feature X
   
   ### Fixed
   - Bug fix Y
   ```

### Version Number Rules

Follow [Semantic Versioning](https://semver.org/):

- **Patch** (0.1.0 → 0.1.1): Bug fixes, no API changes
- **Minor** (0.1.0 → 0.2.0): New features, backward compatible
- **Major** (0.1.0 → 1.0.0): Breaking changes

Pre-release versions:
- **Alpha**: `0.1.0-alpha.1`
- **Beta**: `0.1.0-beta.1`
- **RC**: `0.1.0-rc.1`

## 🏷️ Tag Format

- **Stable**: `py-v0.1.0`
- **Pre-release**: `py-v0.1.0-beta.1`

**Important**: Always use the `py-` prefix to distinguish from Rust crate releases!

## 🚀 Release Types

### Stable Release

```bash
git tag py-v0.1.0
git push origin py-v0.1.0
```

- Publishes to PyPI
- Creates GitHub Release
- Builds all platform wheels

### Pre-Release (Beta/RC)

```bash
git tag py-v0.1.0-beta.1
git push origin py-v0.1.0-beta.1
```

- Publishes to TestPyPI (for testing)
- Creates GitHub Release marked as pre-release
- Builds all platform wheels

### Hotfix Release

For urgent bug fixes:

```bash
# Create hotfix branch
git checkout -b hotfix/0.1.1 py-v0.1.0

# Make fixes
# Update version to 0.1.1
# Commit changes

# Tag and release
git tag py-v0.1.1
git push origin py-v0.1.1

# Merge back to main
git checkout main
git merge hotfix/0.1.1
git push origin main
```

## 🔍 Post-Release Verification

After GitHub Actions completes:

### 1. Check PyPI
```bash
# Visit PyPI page
open https://pypi.org/project/jsonrepair-rs/

# Check version is live
pip index versions jsonrepair
```

### 2. Test Installation
```bash
# Create fresh virtual environment
python -m venv test_env
source test_env/bin/activate  # or `test_env\Scripts\activate` on Windows

# Install from PyPI
pip install jsonrepair-rs==0.1.0

# Test basic functionality
python -c "import jsonrepair; print(jsonrepair.__version__)"
python -c "import jsonrepair; print(jsonrepair.loads('{name: \"test\"}'))"
```

### 3. Verify GitHub Release
- Check https://github.com/Latias94/jsonrepair/releases
- Verify release notes are correct
- Verify wheels are attached

### 4. Test on Different Platforms (Optional)
```bash
# Linux
docker run --rm -it python:3.11 bash -c "pip install jsonrepair-rs==0.1.0 && python -c 'import jsonrepair; print(jsonrepair.__version__)'"

# macOS (if available)
pip install jsonrepair-rs==0.1.0

# Windows (if available)
pip install jsonrepair-rs==0.1.0
```

## 🐛 Troubleshooting

### Build Fails on GitHub Actions

1. Check the Actions log: https://github.com/Latias94/jsonrepair/actions
2. Look for specific error messages
3. Common issues:
   - Rust compilation errors → Check Rust code compatibility
   - PyO3 version mismatch → Update `python/Cargo.toml`
   - Missing dependencies → Check `pyproject.toml`

### PyPI Upload Fails

**Error: File already exists**
- Cannot re-upload same version
- Bump version number (even for fixes)
- Use post-release: `0.1.0.post1`

**Error: Authentication failed**
- Verify PyPI Trusted Publishing setup
- Check workflow has `id-token: write` permission
- For first release, use manual upload

### Version Mismatch

If installed version doesn't match:
```bash
pip cache purge
pip install --no-cache-dir --force-reinstall jsonrepair-rs==0.1.0
python -c "import jsonrepair; print(jsonrepair.__version__)"
```

### Tag Already Exists

If you need to re-tag:
```bash
# Delete local tag
git tag -d py-v0.1.0

# Delete remote tag
git push origin :refs/tags/py-v0.1.0

# Create new tag
git tag py-v0.1.0
git push origin py-v0.1.0
```

## 📞 First-Time Setup

For the first release, you need to:

### 1. Create PyPI Account
- Sign up at https://pypi.org/account/register/
- Verify email address

### 2. Set Up Trusted Publishing
- Go to https://pypi.org/manage/account/publishing/
- Add publisher:
  - **PyPI Project Name**: `jsonrepair-rs`
  - **Owner**: `Latias94`
  - **Repository**: `jsonrepair`
  - **Workflow**: `python-release.yml`
  - **Environment**: (leave empty)

### 3. First Manual Release (if needed)

If Trusted Publishing requires the project to exist first:

```bash
cd python

# Install maturin
pip install maturin

# Build and publish manually
maturin publish

# Enter PyPI credentials when prompted
```

After first manual release, Trusted Publishing will work for future releases.

## 📚 Resources

- [Full Publishing Guide](./PUBLISHING.md)
- [Maturin Documentation](https://www.maturin.rs/)
- [PyPI Trusted Publishing](https://docs.pypi.org/trusted-publishers/)
- [Semantic Versioning](https://semver.org/)

## 🎉 Release Announcement Template

After successful release, announce on:

### GitHub Discussions
```markdown
# 🎉 Python Package v0.1.0 Released!

We're excited to announce the release of `jsonrepair-rs` v0.1.0 for Python!

## Installation
pip install jsonrepair-rs==0.1.0

## What's New
- Feature X
- Improvement Y
- Bug fix Z

## Links
- [PyPI](https://pypi.org/project/jsonrepair-rs/0.1.0/)
- [Documentation](https://github.com/Latias94/jsonrepair/tree/main/python)
- [Changelog](https://github.com/Latias94/jsonrepair/blob/main/CHANGELOG.md)

Try it out and let us know what you think!
```

### Twitter/Social Media
```
🚀 jsonrepair-rs v0.1.0 is now on PyPI!

Fast JSON repair for Python, powered by Rust 🦀

pip install jsonrepair-rs

✨ Features:
- Drop-in replacement for json_repair
- 10x faster for large JSON
- Zero Python dependencies

https://pypi.org/project/jsonrepair-rs/
```

