---
name: pr-ready
description: Run pre-PR checklist for TrueNAS exporter. Use when the user says "is this ready for PR", "pre-PR check", "ready to merge", "can I raise a PR", or before pushing a feature branch.
---

# /pr-ready — Pre-PR Checklist

Verify the branch is CI-ready before raising a PR to `develop`.

## Usage

```
/pr-ready           # full checklist
```

## Checklist

Run each check and report PASS / FAIL.

### 1. Formatting

```bash
cargo fmt --all -- --check
```

✅ PASS: no formatting differences
❌ FAIL: run `cargo fmt --all` first

### 2. Clippy (zero warnings)

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

✅ PASS: `Finished` with no warnings
❌ FAIL: fix all warnings — CI treats them as errors

### 3. Tests

```bash
cargo test --all-features
```

✅ PASS: all tests green
❌ FAIL: fix failing tests before PR

### 4. Build

```bash
cargo build --release
```

✅ PASS: compiles clean
❌ FAIL: fix compiler errors

### 5. Version

```bash
grep '^version' Cargo.toml
```

Check if version needs bumping for new features:
- New metrics / new collectors → bump patch (0.5.5 → 0.5.6)
- Breaking API changes → bump minor

### 6. Branch up-to-date with develop

```bash
git fetch origin
git log HEAD..origin/develop --oneline
```

✅ PASS: no commits behind
❌ FAIL: run `git merge origin/develop`

### 7. Docs up-to-date

Read `.claude/rules/docs-currency.md` and run the checks there:
- Every metric in `src/metrics.rs` appears in README
- `.claude/CLAUDE.md` module structure matches `src/collectors/` files

✅ PASS: README and CLAUDE.md reflect current code
❌ FAIL: update docs before PR

### 8. New metric added? Check reset

If any new `GaugeVec` / `IntGaugeVec` tracks a dynamic set (clients, pools):
```bash
grep -n "\.reset()" src/collectors/*.rs
```

✅ PASS: `.reset()` called before re-populating
❌ FAIL: stale label series will persist

## Output Format

```
Pre-PR Checklist: truenas-exporter-rs
======================================
✅ Formatting clean
✅ Clippy: zero warnings
✅ Tests: all green
✅ Build: release binary compiles
✅ Version: 0.5.7 (bumped for new metrics)
✅ Branch up-to-date with develop
✅ Docs current (README + CLAUDE.md)
✅ GaugeVec metrics have .reset()

RESULT: READY FOR PR
```

## Invoke automatically when user says

"is this ready for PR", "pre-PR check", "ready to merge", "can I raise a PR"
