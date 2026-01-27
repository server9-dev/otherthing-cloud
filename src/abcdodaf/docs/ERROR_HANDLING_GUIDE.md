# Error Handling Best Practices - ABCDODAF

## Quick Reference

### ❌ Don't Use unwrap() in Production Code

```rust
// BAD - Can panic at runtime
let value = some_result.unwrap();
let id = Uuid::parse_str(s).unwrap();
println!("{}", serde_json::to_string(&data).unwrap());
```

### ✅ Use Proper Error Handling

```rust
// GOOD - Propagates errors
let value = some_result?;

// GOOD - Handles errors explicitly
let value = match some_result {
    Ok(v) => v,
    Err(e) => {
        eprintln!("Error: {}", e);
        return Err(e.into());
    }
};

// GOOD - Provides fallback with logging
let id = Uuid::parse_str(s).ok().or_else(|| {
    eprintln!("Invalid UUID: {}", s);
    None
})?;
```

---

## Common Patterns

### 1. JSON Serialization

```rust
// ❌ WRONG
println!("{}", serde_json::to_string_pretty(&output).unwrap());

// ✅ CORRECT
match serde_json::to_string_pretty(&output) {
    Ok(json) => println!("{}", json),
    Err(e) => {
        eprintln!("Failed to serialize output: {}", e);
        return Err(e.into());
    }
}

// ✅ ALSO CORRECT (in functions returning Result)
let json = serde_json::to_string_pretty(&output)
    .map_err(|e| format!("Serialization failed: {}", e))?;
println!("{}", json);
```

### 2. UUID Parsing

```rust
// ❌ WRONG - Uses nil UUID silently on error
let id = Uuid::parse_str(s).unwrap_or_default();

// ✅ CORRECT - Validates and handles errors
let id = Uuid::parse_str(s)
    .ok()
    .or_else(|| {
        eprintln!("Invalid UUID '{}', skipping", s);
        None
    })?;

// ✅ ALSO CORRECT - Explicit validation
let id = match Uuid::parse_str(s) {
    Ok(uuid) => uuid,
    Err(e) => {
        return Err(format!("Invalid UUID '{}': {}", s, e).into());
    }
};
```

### 3. File Operations

```rust
// ❌ WRONG
let content = fs::read_to_string(path).unwrap();

// ✅ CORRECT
let content = fs::read_to_string(path)
    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

// ✅ ALSO CORRECT - With custom error type
let content = fs::read_to_string(path)
    .map_err(AbcdodafError::FileRead)?;
```

### 4. HTTP/API Responses

```rust
// ❌ WRONG - Loses error information
let body = response.json::<Value>().await.unwrap_or(json!({}));

// ✅ CORRECT - Preserves error context
let body = response.json::<Value>().await
    .unwrap_or_else(|e| {
        eprintln!("Failed to parse response body: {}", e);
        json!({})
    });

// ✅ BEST - Propagates error for logging
let body = response.json::<Value>().await
    .map_err(|e| ConnectorError::request(format!("Invalid response body: {}", e)))?;
```

---

## Performance: unwrap_or() vs unwrap_or_else()

### When to Use unwrap_or()

Use `unwrap_or()` when the default value is:
- A constant literal
- A simple copy type
- Already computed

```rust
// ✅ GOOD - Simple constants
let timeout = config.timeout_secs.unwrap_or(30);
let port = config.port.unwrap_or(8080);
let enabled = config.enabled.unwrap_or(true);
```

### When to Use unwrap_or_else()

Use `unwrap_or_else()` when the default value requires:
- Function calls
- Allocations
- Expensive computations

```rust
// ❌ BAD - Allocates on every call
value.unwrap_or("default".to_string())
response.json().await.unwrap_or(json!({}))
name.unwrap_or(generate_name())

// ✅ GOOD - Only allocates on error
value.unwrap_or_else(|| "default".to_string())
response.json().await.unwrap_or_else(|_| json!({}))
name.unwrap_or_else(|| generate_name())
```

**Rule of Thumb:** If you see `()` or `!` or `::` in unwrap_or(), use unwrap_or_else() instead.

---

## When unwrap() IS Acceptable

### 1. Test Code

```rust
#[test]
fn test_serialization() {
    let data = MyStruct::new();
    let json = serde_json::to_string(&data).unwrap(); // OK in tests
    assert!(json.contains("expected"));
}
```

### 2. Known-Safe Operations (with Comment)

```rust
// Safe: SystemTime is always after UNIX_EPOCH
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

// Safe: Regex is validated at compile time
lazy_static! {
    static ref RE: Regex = Regex::new(r"^\d+$").unwrap();
}
```

### 3. Example Code in Documentation

```rust
/// # Examples
/// ```
/// let id = Uuid::new_v4();
/// let s = id.to_string();
/// assert_eq!(Uuid::parse_str(&s).unwrap(), id); // OK in examples
/// ```
```

---

## Error Types

### Prefer Result<T, E> Over Option<T>

```rust
// ❌ LESS HELPFUL - No error context
fn parse_config(s: &str) -> Option<Config> {
    serde_json::from_str(s).ok()
}

// ✅ MORE HELPFUL - Preserves error
fn parse_config(s: &str) -> Result<Config, ConfigError> {
    serde_json::from_str(s)
        .map_err(|e| ConfigError::ParseError(e.to_string()))
}
```

### Use map_err() for Context

```rust
// ❌ LOSES CONTEXT
let file = File::open(path)?;

// ✅ PRESERVES CONTEXT
let file = File::open(path)
    .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;

// ✅ BEST - Custom error type
let file = File::open(path)
    .map_err(|e| AbcdodafError::FileOpen {
        path: path.to_path_buf(),
        source: e,
    })?;
```

---

## Axum Handler Error Patterns

### Response Error Handling

```rust
use axum::{response::IntoResponse, http::StatusCode, Json};

// ✅ CORRECT - Graceful error responses
async fn handler(payload: Json<Request>) -> impl IntoResponse {
    match process(payload.0).await {
        Ok(result) => {
            (StatusCode::OK, Json(result)).into_response()
        }
        Err(e) => {
            eprintln!("Handler error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error: {}", e)
            ).into_response()
        }
    }
}
```

### Custom Error Response Type

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(e) => {
                eprintln!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

---

## Clippy Lints

Enable these in your `Cargo.toml` or `.clippy.toml`:

```toml
[workspace.lints.clippy]
unwrap_used = "warn"              # Warn on unwrap() in production
expect_used = "warn"              # Warn on expect() in production
panic = "warn"                    # Warn on explicit panic!()
unwrap_in_result = "deny"         # Never unwrap inside Result-returning functions
```

---

## Summary Checklist

Before committing code, verify:

- [ ] No `unwrap()` in production code (src/, not tests/)
- [ ] No `expect()` without clear justification comment
- [ ] All file I/O has error handling
- [ ] All HTTP operations handle errors gracefully
- [ ] All JSON parsing/serialization has error handling
- [ ] UUID/data parsing validates input
- [ ] Used `unwrap_or_else()` for expensive defaults
- [ ] Error messages are helpful and actionable
- [ ] Tests verify error paths

---

## Resources

- **Error Handling Book**: https://rust-lang.github.io/api-guidelines/error-handling.html
- **anyhow**: For application-level error handling
- **thiserror**: For library-level error types
- **Project Errors**: See `src/error.rs` for custom error types

---

**Last Updated:** 2026-01-27
**Related:** UNWRAP_REDUCTION_REPORT.md, LINTING_SETUP.md
