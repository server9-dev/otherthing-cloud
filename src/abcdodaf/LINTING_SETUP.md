# Rust Linting Configuration for ABCDODAF

This document describes the comprehensive linting setup for the ABCDODAF project, including formatting rules (rustfmt) and linting rules (clippy).

## Overview

The project uses two primary linting tools:

1. **rustfmt** - Automatic code formatting
2. **clippy** - Comprehensive linting and best practices enforcement

## Files Created

### 1. `.rustfmt.toml`

Location: `/mnt/nvme1n1p3/Projects/server9-dev/otherthing-cloud/src/abcdodaf/.rustfmt.toml`

This file configures rustfmt with the following settings:

- **Edition**: 2021 (matches Cargo.toml)
- **Line width**: 100 characters max
- **Indentation**: 4 spaces (no tabs)
- **Import organization**: Reorders imports and modules
- **Heuristics**: Uses "Max" for smaller lines where possible
- **Struct formatting**: Uses field init shorthand
- **Array/Chain width**: 80 characters
- **Newline style**: Unix (LF)

Note: Only stable rustfmt features are used for maximum compatibility across Rust versions.

### 2. `clippy.toml`

Location: `/mnt/nvme1n1p3/Projects/server9-dev/otherthing-cloud/src/abcdodaf/clippy.toml`

This file configures clippy with the following thresholds:

- **MSRV**: 1.70.0 (Minimum Supported Rust Version)
- **Cognitive complexity**: 25 (functions should be refactored above this)
- **Type complexity**: 250
- **Max function lines**: 150
- **Max struct bools**: 3
- **Stack size threshold**: 512,000 bytes
- **Pass by value size**: 256 bytes

### 3. Cargo.toml Lint Configuration

Added comprehensive lint configuration to `Cargo.toml`:

```toml
[lints.rust]
unsafe_code = "warn"
dead_code = "warn"
unused_imports = "warn"
unused_variables = "warn"
deprecated = "warn"

[lints.clippy]
correctness = "deny"      # Deny bugs and correctness issues
suspicious = "deny"       # Deny suspicious patterns
perf = "warn"            # Warn on performance issues
complexity = "warn"      # Warn on unnecessary complexity
style = "warn"           # Warn on style issues
pedantic = "warn"        # Warn on pedantic issues
unwrap_used = "warn"     # Warn on unwrap usage
expect_used = "warn"     # Warn on expect usage
panic = "warn"           # Warn on panic usage
cargo = "warn"           # Warn on cargo issues
```

## Usage

### Formatting Code

```bash
# Format all code in the project
cargo fmt --all

# Check formatting without modifying files
cargo fmt --all -- --check

# Format a specific file
cargo fmt --all -- path/to/file.rs
```

### Running Clippy

```bash
# Run clippy on all features
cargo clippy --all-features

# Run clippy with auto-fix
cargo clippy --all-features --fix --allow-dirty --allow-staged

# Treat warnings as errors (CI mode)
cargo clippy --all-features -- -D warnings

# Only check workspace code (exclude dependencies)
cargo clippy --all-features --no-deps
```

### Common Workflows

#### Before Committing

```bash
# 1. Format code
cargo fmt --all

# 2. Run clippy and fix issues
cargo clippy --all-features --fix --allow-dirty --allow-staged

# 3. Run tests
cargo test --all-features
```

#### CI/CD Pipeline

```bash
# 1. Check formatting (fail if not formatted)
cargo fmt --all -- --check

# 2. Run clippy (treat warnings as errors)
cargo clippy --all-features -- -D warnings

# 3. Run tests
cargo test --all-features

# 4. Build release
cargo build --release --all-features
```

## Lint Categories

### Correctness (Deny)
- Catches definite bugs and correctness issues
- Examples: logic errors, type mismatches, incorrect API usage

### Suspicious (Deny)
- Catches suspicious patterns that are likely bugs
- Examples: unused results, incorrect comparisons, problematic patterns

### Performance (Warn)
- Identifies performance issues
- Examples: unnecessary clones, inefficient algorithms, allocation issues

### Complexity (Warn)
- Flags overly complex code
- Examples: deeply nested code, too many arguments, cognitive complexity

### Style (Warn)
- Enforces consistent code style
- Examples: naming conventions, formatting preferences, idiomatic patterns

### Pedantic (Warn)
- Enforces additional best practices
- Examples: documentation standards, explicit types, comprehensive patterns

## Allowed Lints

Some lints are explicitly allowed to reduce noise:

- `missing_errors_doc` - Not required to document every error
- `missing_panics_doc` - Not required to document every panic
- `module_name_repetitions` - Sometimes necessary for clarity

## Special Configurations

### Test Code

- `allow-expect-in-tests = true` - `.expect()` is allowed in tests
- `allow-unwrap-in-tests = true` - `.unwrap()` is allowed in tests

### Short Identifiers

Allowed short identifiers (below minimum length):
- `id` - Identifier
- `db` - Database
- `ui` - User Interface
- `io` - Input/Output
- `tx` - Transmitter/Transaction
- `rx` - Receiver

### Documentation Identifiers

Valid identifiers for documentation:
- DoDAF, BPMN, CMMN, DMN, BPM, UUID
- JSON, YAML, TOML, XML
- PostgreSQL, SQLx

## Current Status

### Formatting Status

✅ **Completed**: All code has been formatted with `cargo fmt --all`

Statistics:
- 171 files modified
- 4,100 insertions
- 5,062 deletions
- Net reduction: 962 lines (improved code density)

### Clippy Status

⚠️ **In Progress**: Clippy revealed compilation errors that need fixing

Current issues to resolve:
1. Missing field `attached_to_activity_id` in `IntermediateEventNode` (4 occurrences)
2. Incorrect field names in struct initialization (3 occurrences)
3. Borrow checker error in `property_editor.rs` (1 occurrence)
4. Unused imports (34 warnings)

### Next Steps

1. **Fix Compilation Errors** (Priority 1)
   - Fix `IntermediateEventNode` initialization
   - Fix `DataStore` field name
   - Fix `Subprocess` field names
   - Fix borrow checker issue in property editor

2. **Fix Warnings** (Priority 2)
   - Remove unused imports
   - Fix raw string literal hashes
   - Nest or-patterns
   - Add literal separators for large numbers

3. **Enable Auto-Fix** (Priority 3)
   - Run `cargo clippy --all-features --fix` after compilation errors are resolved
   - Review and commit auto-fixed changes

4. **CI/CD Integration** (Priority 4)
   - Add formatting check to CI pipeline
   - Add clippy check to CI pipeline
   - Configure to treat warnings as errors in CI

## Benefits

### Code Quality
- Consistent formatting across the entire codebase
- Early detection of bugs and anti-patterns
- Enforced best practices and idioms

### Developer Experience
- Reduced code review friction (formatting is automated)
- Clear warnings about potential issues
- Educational feedback on Rust best practices

### Maintenance
- Easier to read and understand code
- Reduced cognitive load when navigating codebase
- Better documentation through enforced standards

## Troubleshooting

### Unstable Feature Warnings

If you see warnings about unstable features, it means rustfmt is trying to use nightly-only features. The current `.rustfmt.toml` only uses stable features. To use nightly features:

```bash
# Install nightly
rustup install nightly

# Format with nightly
cargo +nightly fmt --all
```

### Conflicting Lints

If clippy lints conflict with each other or your needs, you can:

1. **Allow specific lints in code**:
   ```rust
   #[allow(clippy::lint_name)]
   fn my_function() { }
   ```

2. **Configure in clippy.toml**:
   Add the lint to the appropriate threshold

3. **Override in Cargo.toml**:
   ```toml
   [lints.clippy]
   specific_lint = "allow"
   ```

### Performance Issues

If linting is slow:

1. Use `--no-deps` to skip dependencies
2. Use `--all-features` selectively
3. Run clippy on changed files only in development

## References

- [rustfmt Documentation](https://rust-lang.github.io/rustfmt/)
- [Clippy Documentation](https://rust-lang.github.io/rust-clippy/)
- [Rust Lint List](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html)
- [Cargo Manifest Lints](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)

## Version History

- **2026-01-27**: Initial linting setup
  - Created `.rustfmt.toml` with stable features
  - Created `clippy.toml` with comprehensive configuration
  - Added lint configuration to `Cargo.toml`
  - Formatted entire codebase with `cargo fmt`
  - Identified compilation errors requiring fixes before clippy can complete
