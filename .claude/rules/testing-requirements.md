# Testing Requirements — TrueNAS Exporter

## Test Structure

- **Unit tests**: `#[cfg(test)] mod tests` inside `src/` files (for pure logic)
- **Integration tests**: `tests/` directory (for collector/metrics/type logic against mocked data)
- **No live TrueNAS required**: All tests must pass without a real TrueNAS instance

## GIVEN / WHEN / THEN Pattern

All tests must follow this structure:
```rust
#[test]
fn test_boot_pool_healthy_sets_metric_to_one() {
    // Given: a healthy boot pool response
    let pool = BootPoolState {
        name: "boot-pool".to_string(),
        status: "ONLINE".to_string(),
        healthy: true,
        size: 100_000_000,
        allocated: 20_000_000,
        scan: None,
    };

    // When: health value is computed
    let health_value = if pool.healthy { 1.0 } else { 0.0 };

    // Then: it equals 1.0
    assert_eq!(health_value, 1.0);
}
```

Labels must describe the setup / trigger / expected outcome — not just mark the section.

## What to Test

### Types (`tests/types_test.rs`)
- JSON deserialization of API responses
- `#[serde(default)]` behavior for missing fields
- `#[serde(rename = "...")]` for fields with spaces/different names
- Optional field handling (`Option<T>` stays `None` when absent)

### Metrics (`tests/metrics_test.rs`)
- `MetricsCollector::new()` succeeds
- All metric fields are accessible
- `render()` produces valid Prometheus text format

### Collectors (`tests/collectors_test_simple.rs`)
- Pure logic that can be tested without network
- Metric computation (e.g. ratio = allocated / size)
- Edge cases (size = 0, empty Vec, None fields)

### What NOT to Test
- WebSocket connection behavior (needs live TrueNAS)
- Tokio runtime internals
- Serde or prometheus crate internals
- Third-party library behavior

## Serde Deserialization Tests

TrueNAS API responses often have missing or null fields. Test both full and partial JSON:
```rust
#[test]
fn test_boot_pool_state_missing_scan() {
    // Given: JSON without scan field
    let json = r#"{"name":"boot-pool","status":"ONLINE","healthy":true}"#;

    // When: deserialized
    let pool: BootPoolState = serde_json::from_str(json).unwrap();

    // Then: scan is None, numeric fields default to 0
    assert!(pool.scan.is_none());
    assert_eq!(pool.size, 0);
    assert_eq!(pool.allocated, 0);
}
```

## Test Naming

Use snake_case descriptive names:
```
test_pool_health_is_zero_when_degraded      ✅
test_nfs_client_info_empty_when_no_clients  ✅
test1                                        ❌
testPoolHealth                               ❌
```

## Running Tests

```bash
cargo test --all-features          # all tests
cargo test --test metrics_test     # specific file
cargo test boot_pool               # filter by name
cargo test -- --nocapture          # show println! output
```

Or via Makefile: `make test`
