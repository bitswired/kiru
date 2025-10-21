# justfile - Run commands from workspace root

# Default recipe (shows help)
default:
    @just --list

# ============================================================================
# Rust Core Library (kiru-core)
# ============================================================================

# Build the core Rust library
build-rust:
    cargo build -p kiru

# Build core library in release mode
build-rust-release:
    cargo build -p kiru --release

# Run tests for core library
test-rust:
    cargo test -p kiru

# Run benchmarks for core library
bench-rust:
    cd kiru-core && cargo bench

# Check core library (faster than build)
check-rust:
    cargo check -p kiru

# Format Rust code
fmt-rust:
    cargo fmt --all

# Lint Rust code
lint-rust:
    cargo clippy --all -- -D warnings

# ============================================================================
# Python Bindings (kiru-py)
# ============================================================================

# Build Python package in development mode (installs locally)
build-py:
    cd kiru-py && maturin develop

# Build Python package in release mode
build-py-release:
    cd kiru-py && maturin develop --release

# Build Python wheel for distribution
build-wheel:
    cd kiru-py && maturin build --release

# Build Python wheels for all platforms (via maturin)
build-wheel-all:
    cd kiru-py && maturin build --release --sdist

# Publish Python package to PyPI
publish-py:
    cd kiru-py && maturin publish

# Publish to test PyPI
publish-py-test:
    cd kiru-py && maturin publish --repository testpypi

# Run Python tests
test-py:
    cd kiru-py && uv pytest python/t.py

# Run Python benchmarks
bench-py:
    cd kiru-py && uv run python/test.py

# ============================================================================
# Combined Operations
# ============================================================================

# Build everything (Rust + Python)
build-all: build-rust build-py

# Test everything (Rust + Python)
test-all: test-rust test-py

# Clean all build artifacts
clean:
    cargo clean
    rm -rf kiru-py/target
    find . -type d -name "__pycache__" -exec rm -r {} +
    find . -type f -name "*.pyc" -delete
    find . -type d -name "*.egg-info" -exec rm -r {} +

# Format all code (Rust + Python)
fmt: fmt-rust
    cd kiru-py && black python/
    cd kiru-py && isort python/

# Lint all code
lint: lint-rust
    cd kiru-py && ruff check python/

# Check everything (fast validation)
check: check-rust
    @echo "✓ All checks passed"

# ============================================================================
# Development Helpers
# ============================================================================