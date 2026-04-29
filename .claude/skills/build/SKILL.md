---
name: build
description: Build the TrueNAS exporter binary or run CI checks. Use when the user says "build", "compile", "release binary", "run CI", "check if it compiles", or needs to verify before deploying.
---

# /build — Build and Verify

## Usage

```
/build              # release binary
/build --dev        # debug binary (faster)
/build --ci         # full CI simulation (fmt, clippy, test, audit, build, docker)
/build --check      # cargo check only (fastest compile check)
```

## Commands

| Option | Command |
|--------|---------|
| release | `cargo build --release` |
| dev | `cargo build` |
| check | `cargo check --all-features` |
| ci | `make ci` |
| docker | `make docker-build` |

## Process

1. Run the requested build command
2. If build fails — show the exact error, identify the file:line, fix it
3. If clippy warnings appear — treat as errors (CI uses `-D warnings`)
4. Report binary size for release builds: `du -h target/release/truenas-exporter`

## Invoke automatically when user says

"build this", "compile it", "release binary", "does it compile", "run CI"
