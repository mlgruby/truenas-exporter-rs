---
name: docs-currency
description: Load when reviewing changes before a PR, or when any of these files are modified: collectors/, metrics.rs, types.rs, README.md, CLAUDE.md
---

# Docs Currency Rule

Before raising a PR, verify all docs reflect the current state of the code.

## What to check

### README.md — Metrics Exposed section

- Every metric in `src/metrics.rs` must appear in README under the correct section
- Labels listed must match the actual `&[...]` passed to `with_label_values`
- Limitations section must not claim something is unavailable if it was since added

### .claude/CLAUDE.md — Module Structure

- Every file in `src/collectors/` must have a line in the module structure listing
- The comment next to each file must describe what it actually collects (not what it used to)

### Collectors doc comment (// module-level)

- Each `src/collectors/<x>.rs` top-of-file comment must list all metrics it produces
- If a metric was renamed, the doc must reflect the new name

## Triggers

Run this check whenever:
- A new collector file is added
- A metric is added, renamed, or removed from `metrics.rs`
- A collector is split or merged
- A new API type is added to `types.rs`

## Fix pattern

1. Open `src/metrics.rs` — source of truth for all metric names and labels
2. Diff against README Metrics Exposed section
3. Update README for any missing/changed metrics
4. Open `.claude/CLAUDE.md` module structure — check each `collectors/` entry
5. Update CLAUDE.md for any new/moved/removed files
