# TrueNAS Exporter (Rust)

## Project Overview

Prometheus exporter for TrueNAS Scale, written in Rust. Connects to TrueNAS via WebSocket JSON-RPC 2.0 API, collects metrics, and exposes them at `/metrics` for Prometheus scraping.

**Data Flow**: TrueNAS Scale WebSocket API → Collectors → Prometheus Metrics → Grafana

## Tech Stack

- **Language**: Rust (stable)
- **Async**: Tokio
- **HTTP Server**: Axum
- **Metrics**: prometheus crate (Gauge, GaugeVec, IntGaugeVec)
- **WebSocket**: tokio-tungstenite
- **Config**: config crate + dotenvy (.env file)
- **Deployment**: Docker container on TrueNAS (or any host)

## Module Structure

```
src/
  main.rs                    ← entrypoint
  lib.rs                     ← public re-exports
  config.rs                  ← Config struct (loaded from .env / config/Default.toml)
  error.rs                   ← unified Result/Error types
  metrics.rs                 ← MetricsCollector — all Prometheus metric definitions
  server.rs                  ← Axum HTTP server + collection loop
  collectors/
    mod.rs                   ← CollectionContext, CollectionResult, re-exports
    pool.rs                  ← ZFS pool health, scan, VDev errors
    dataset.rs               ← Dataset used/available/compressratio
    disk.rs                  ← Disk info + temperature
    smart.rs                 ← SMART test results
    app.rs                   ← TrueNAS app state + update available
    share.rs                 ← SMB/NFS share counts
    cloud_sync.rs            ← Cloud sync task state + progress
    snapshot.rs              ← Snapshot task state
    alert.rs                 ← Alert counts by level/dismissed
    system_info.rs           ← Uptime, load avg, memory, version
    system_reporting.rs      ← CPU%, mem%, disk temps, ARC, net I/O
    network_interface.rs     ← Interface link state
    service.rs               ← Service running/enabled state
    boot_pool.rs             ← Boot pool health, scrub, used ratio
    nfs.rs                   ← NFS client count + per-client detail (v3/v4)
    iscsi.rs                 ← iSCSI session count
  truenas/
    mod.rs
    client.rs                ← TrueNasClient — one method per API endpoint
    connection.rs            ← WebSocket connection + auth (ConnectionManager)
    types.rs                 ← Serde structs for all API responses
tests/
  metrics_test.rs
  types_test.rs
  collectors_test_simple.rs
  integration_test.rs
  ...
```

## Development Guidelines

### Verification First

**After any code change**:
1. `cargo fmt --all` — format first
2. `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings
3. `cargo build` — must compile clean
4. `cargo test --all-features` — all tests green

Use `make dev` to run fmt + clippy + test in one shot.

### Following Rules

Read rules in `.claude/rules/` when you need them:
- **Writing any Rust code** → `code-style.md`
- **Adding/modifying metrics** → `metrics-patterns.md`
- **Adding/modifying tests** → `testing-requirements.md`

### Adding a New Metric — Checklist

1. Add Serde struct to `src/truenas/types.rs` (if new API type needed)
2. Add query method to `src/truenas/client.rs`
3. Add metric field to `MetricsCollector` in `src/metrics.rs` (register + store in `new()`)
4. Add collector function in `src/collectors/<area>.rs`
5. Wire collector into `collect_metrics()` in `src/server.rs`
6. Add to `collectors/mod.rs` re-exports if new file
7. Run `cargo fmt && cargo clippy && cargo test`

### Build Commands

```bash
make dev          # fmt + clippy + test (pre-commit check)
make build        # cargo build --release
make test         # cargo test --all-features
make fmt          # cargo fmt --all
make clippy       # cargo clippy --all-features --tests -- -D warnings
make audit        # cargo audit
make ci           # full CI simulation (check, fmt, clippy, test, audit, build, docker)
```

### Running Locally

```bash
cp .env.example .env   # fill in TRUENAS_HOST and TRUENAS_API_KEY
cargo run
curl http://localhost:9100/metrics
```

### Release Flow

Feature branch → PR to `develop` (CI: check, fmt, clippy, test, audit, build, docker)
`develop` → PR to `main` (CI: check, fmt, clippy)
Tag `vX.Y.Z` on `main` → GitHub Actions: Docker push to ghcr.io + GitHub Release

**Commit directly to `main`** for docs-only changes (README, CLAUDE.md, rules, skills) — no PR needed, no CI risk.
**Always use PR flow** for any `.rs`, `Cargo.toml`, `Dockerfile`, or `.github/workflows` changes.

## Key Invariants

- **Never `.unwrap()` on user/API data** — use `?` or handle with `warn!` + `CollectionStatus::Failed`
- **Collector failures are non-fatal** — one broken API must not stop other collectors
- **MetricsCollector is `Clone`** — all metric fields wrapped in `Arc<_>`
- **ConnectionManager holds single WebSocket** — do not open new connections per query
- **`#[serde(default)]` on all optional API fields** — TrueNAS may omit any field

## Related Resources

- TrueNAS Scale WebSocket API: `wss://<host>/websocket` (JSON-RPC 2.0 over DDP)
- Prometheus crate: `prometheus` on crates.io
- Grafana dashboards: `../Grafana_dashboards/`
- Alert rules: `../Grafana_dashboards/truenas_alerts.yaml`

## Skills

- **`/build`** — build release binary or run CI checks
- **`/test`** — run test suite
- **`/commit`** — generate conventional commit message
- **`/pr-ready`** — pre-PR checklist (fmt, clippy, tests, version)

## Agents

- **code-reviewer** — review code against project standards before PR
