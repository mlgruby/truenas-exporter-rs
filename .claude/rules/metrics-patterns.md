# Metrics Patterns — TrueNAS Exporter

## MetricsCollector Structure

`src/metrics.rs` is the single source of truth for all Prometheus metrics.

### Adding a new metric — 3 steps

**1. Declare field on struct:**
```rust
pub struct MetricsCollector {
    pub my_new_metric: Arc<Gauge>,
    pub my_labeled_metric: Arc<GaugeVec>,
}
```

**2. Register and store in `new()`:**
```rust
let my_new_metric = Arc::new(
    register_gauge_with_registry!(
        opts!("truenas_my_new_metric", "Human description"),
        registry
    )?
);
let my_labeled_metric = Arc::new(
    register_gauge_vec_with_registry!(
        opts!("truenas_my_labeled_metric", "Human description"),
        &["label_one", "label_two"],
        registry
    )?
);
```

**3. Return in struct literal:**
```rust
Ok(Self {
    // ...existing fields...
    my_new_metric,
    my_labeled_metric,
})
```

## Metric Types — When to Use What

| Type | Use case | Example |
|------|----------|---------|
| `Gauge` | Single numeric value | `boot_pool_health`, `nfs_client_count` |
| `GaugeVec` | Value per label combination (f64) | `nfs_client_seconds_since_renew` |
| `IntGaugeVec` | Info/presence metric (integer) | `nfs_client_info` (always 1) |

### Info pattern (presence metric)
When you want to expose label values but the numeric value is meaningless, use `IntGaugeVec` set to `1`:
```rust
ctx.metrics.nfs_client_info
    .with_label_values(&[addr, name, version, status])
    .set(1);
```
Prometheus query: `truenas_nfs_client_info{name="pve1"}` — the presence of the series is the signal.

## Label Design

- Labels are **permanent** — changing or reordering them after shipping breaks dashboards
- Cardinality: avoid labels with unbounded values (free-form strings from API)
- `address` label for IPs: include port if returned by API (e.g. `192.168.10.12:871`)
- `version` for NFS: string `"3"` or `"4"` (not integer)
- `status` labels: use the raw API string (e.g. `"confirmed"`, `"ONLINE"`) — don't map to 0/1

## Reset Pattern for Label-Vector Metrics

Any `GaugeVec` / `IntGaugeVec` that tracks a dynamic set (clients, pools, apps) MUST be reset before re-populating:
```rust
ctx.metrics.nfs_client_info.reset();
ctx.metrics.nfs_client_seconds_since_renew.reset();
// ... then emit current values
```

Without reset, a client that disconnects keeps its label series at stale value forever.

## Metric Naming Convention

```
truenas_<resource>_<measurement>[_<unit>]
```

| Resource | Examples |
|----------|---------|
| `pool` | `truenas_pool_health`, `truenas_pool_size_bytes` |
| `boot_pool` | `truenas_boot_pool_health`, `truenas_boot_pool_used_ratio` |
| `dataset` | `truenas_dataset_used_bytes` |
| `disk` | `truenas_disk_temperature_celsius` |
| `nfs` | `truenas_nfs_client_count`, `truenas_nfs_client_info` |
| `iscsi` | `truenas_iscsi_client_count` |
| `zfs` | `truenas_zfs_arc_size_bytes` |
| `app` | `truenas_app_info`, `truenas_app_running` |
| `alert` | `truenas_alert_count` |
| `system` | `truenas_system_uptime_seconds` |

### Unit suffixes
- `_bytes` — raw byte counts
- `_seconds` — unix timestamps OR durations in seconds
- `_ratio` — 0.0–1.0 fractional value
- `_celsius` — temperature
- `_count` — dimensionless count
- no suffix — boolean-ish (0/1) or dimensionless gauge

## CollectionStatus

Always return `Ok(CollectionStatus::Success)` or `Ok(CollectionStatus::Failed)` — never bubble up `Err` for expected API failures.

`Failed` is logged and the collection loop continues. `server.rs` tracks `any_success` — if ALL collectors fail, `truenas_up` is set to 0.

## ZFS ARC note

ARC size comes from `reporting.get_data` with graph `"arcsize"`, legend field `"size"`.
The value is current ARC resident bytes (~50–60 GB on a loaded system). This is NOT a memory leak — ZFS ARC fills available RAM by design.
