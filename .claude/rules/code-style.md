# Code Style — TrueNAS Exporter

## General Principles

- Follow standard Rust idioms and `rustfmt` formatting
- Meaningful names that reflect TrueNAS/Prometheus domain concepts
- Functions do one thing — if it collects metrics for a resource, it only does that
- No comments that explain WHAT; only WHY (non-obvious constraints, workarounds, TrueNAS API quirks)
- No unused code — if something is removed, delete it

## Error Handling

### At API boundaries (TrueNAS responses)
```rust
// Good — warn and return Failed, let other collectors continue
Err(e) => {
    warn!("Failed to query boot pool: {}", e);
    Ok(CollectionStatus::Failed)
}
```

### Never `.unwrap()` on external data
```rust
// Bad
let val = response.result.unwrap();

// Good
let val = response.result.ok_or_else(|| anyhow!("missing result"))?;
```

### Use `?` inside collectors that return `CollectionResult`
`CollectionResult` is `anyhow::Result<CollectionStatus>` — use `?` for unexpected errors, return `Failed` for expected API failures.

## Metrics Patterns

### All metric fields wrapped in Arc
```rust
pub struct MetricsCollector {
    pub pool_health: Arc<GaugeVec>,
    pub boot_pool_health: Arc<Gauge>,
}
```

### GaugeVec label order — stable and meaningful
Labels must be consistent with Prometheus naming conventions:
- `name` before `status`
- `address` before `name` before `version`
- Never change label order after a metric ships — it breaks existing dashboards

### Reset label-vector metrics before re-populating
```rust
ctx.metrics.nfs_client_info.reset();
// then re-emit all current values
```
Prevents stale label combinations persisting after clients disconnect.

### Metric names follow `truenas_<resource>_<measurement>_<unit>` pattern
- `truenas_pool_health` — no unit (boolean-ish gauge)
- `truenas_pool_used_bytes` — bytes suffix
- `truenas_boot_pool_used_ratio` — ratio (0–1), no unit suffix
- `truenas_boot_pool_last_scrub_seconds` — unix timestamp in seconds

## Struct Design

### API response types in `types.rs`
- `#[serde(default)]` on every field that TrueNAS may omit
- `Option<T>` for fields that can be null in the JSON
- `#[serde(rename = "api_field_name")]` when Rust name differs from API name
- All structs get `#[derive(Debug, Deserialize)]`; add `Clone` only if needed

```rust
#[derive(Debug, Deserialize)]
pub struct BootPoolState {
    pub name: String,
    pub status: String,
    pub healthy: bool,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub scan: Option<BootPoolScan>,
}
```

### Field names with spaces (TrueNAS API quirk)
```rust
#[serde(rename = "seconds from last renew", default)]
pub seconds_since_renew: u64,
```

## Async Patterns

### Concurrent API calls with `tokio::join!`
When multiple independent API calls are needed for a single collector:
```rust
let (count_res, v4_res, v3_res) = tokio::join!(
    ctx.client.query_nfs_client_count(),
    ctx.client.query_nfs4_clients(),
    ctx.client.query_nfs3_clients(),
);
```
Do NOT `.await` them sequentially when they're independent.

### Single WebSocket connection
All queries go through `ConnectionManager` — never create a new client/connection per collector call.

## Collector Function Signature

Every collector must match:
```rust
pub async fn collect_<name>(ctx: &CollectionContext<'_>) -> CollectionResult
```

Returns `Ok(CollectionStatus::Success)` or `Ok(CollectionStatus::Failed)` — never `Err(...)` for expected API failures.

## Naming Conventions

| Thing | Convention | Example |
|-------|-----------|---------|
| Collector function | `collect_<resource>_<aspect>` | `collect_boot_pool_metrics` |
| Query method | `query_<resource>` | `query_nfs4_clients` |
| Metric field | `<resource>_<measurement>` | `boot_pool_health` |
| Type struct | PascalCase matching API concept | `BootPoolState`, `NfsClientInfo` |

## Formatting

`cargo fmt --all` before every commit. CI enforces `cargo fmt --all -- --check`.

## Anti-Patterns

### ❌ Panicking on API data
```rust
pool.scan.unwrap().errors  // Bad — API may return null scan
pool.scan.as_ref().map(|s| s.errors).unwrap_or(0)  // Good
```

### ❌ Nested match hell for JSON extraction
```rust
// Acceptable for deeply nested optional JSON (TrueNAS date format)
if let Some(serde_json::Value::Object(map)) = &scan.end_time {
    if let Some(serde_json::Value::Number(num)) = map.get("$date") {
        if let Some(millis) = num.as_u64() { ... }
    }
}
```
This pattern is expected for TrueNAS `$date` objects — don't fight it.

### ❌ Hardcoded API method names outside client.rs
All TrueNAS API method strings (`"boot.get_state"`, `"nfs.client_count"`, etc.) belong in `client.rs` only.

### ❌ Registering metrics outside MetricsCollector::new()
All metric registration (`register_gauge!`, etc.) happens in `MetricsCollector::new()`.

### ❌ Docstrings on private implementation details
Only public API methods and structs warrant doc comments. Internal collector logic is self-documenting via naming.
