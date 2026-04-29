#!/bin/bash
# Claude Code session-start hook — TrueNAS Exporter

cat << 'EOF'

TrueNAS Exporter (Rust) - Quick Reminders

MANDATORY RULES (read before coding):
   Code style:     .claude/rules/code-style.md
     - #[serde(default)] on all optional API fields
     - Never .unwrap() on API data — use ? or warn!+Failed
     - GaugeVec metrics must .reset() before re-populate
     - All API method strings live in client.rs only
   Metrics:        .claude/rules/metrics-patterns.md
     - Names: truenas_<resource>_<measurement>[_<unit>]
     - Register AND return in MetricsCollector::new()
     - Label order is permanent — changing breaks dashboards
   Testing:        .claude/rules/testing-requirements.md
     - GIVEN/WHEN/THEN pattern required
     - No live TrueNAS needed — mock JSON in tests

KEY INVARIANTS:
   Collector failures are non-fatal — return Ok(Failed), not Err
   Single WebSocket connection — never create new client per query
   CollectionContext is the only way collectors access metrics/client
   MetricsCollector is Clone — all fields are Arc<_>

ADDING A NEW METRIC:
   1. types.rs  — add Serde struct for API response
   2. client.rs — add query_<name>() method
   3. metrics.rs — declare field, register, return in new()
   4. collectors/<area>.rs — implement collect_<name>()
   5. server.rs  — wire collect_<name>() into collect_metrics()
   6. cargo fmt && cargo clippy && cargo test

COMMON COMMANDS:
   make dev          # fmt + clippy + test
   make build        # release binary
   make ci           # full CI simulation
   cargo run         # run locally (needs .env)
   curl http://localhost:9100/metrics

RELEASE FLOW:
   feature branch -> PR to develop (CI runs)
   develop -> PR to main
   tag v0.X.Y on main -> Docker push + GitHub Release

EOF
