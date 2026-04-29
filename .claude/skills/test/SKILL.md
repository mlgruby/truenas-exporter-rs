---
name: test
description: Run TrueNAS exporter tests. Use when the user says "run tests", "are tests passing", "test this", "does it pass", "check tests", or after implementing a feature.
---

# /test — Run Tests

## Usage

```
/test                          # all tests
/test --file metrics_test      # specific test file
/test --filter boot_pool       # filter by test name pattern
/test --verbose                # with output (--nocapture)
```

## Commands

| Option | Command |
|--------|---------|
| all | `cargo test --all-features` |
| specific file | `cargo test --test <filename>` |
| filter | `cargo test <pattern>` |
| verbose | `cargo test -- --nocapture` |
| doc tests | `cargo test --doc` |

## Test Files

| File | What it tests |
|------|--------------|
| `tests/metrics_test.rs` | MetricsCollector creation, render output |
| `tests/types_test.rs` | Serde deserialization of API types |
| `tests/collectors_test_simple.rs` | Collector logic without network |
| `tests/integration_test.rs` | End-to-end server startup |
| `tests/edge_cases_test.rs` | Boundary conditions |
| `tests/error_quality_test.rs` | Error message quality |
| `tests/property_based_test.rs` | Proptest-based property tests |

## Process

1. Run tests
2. If failures — show exact test name + failure message
3. Read the failing test to understand expected behavior
4. Fix the code (or test if the test is wrong)
5. Re-run to confirm green

## Invoke automatically when user says

"run tests", "are tests green", "test this", "does it pass"
