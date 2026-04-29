---
name: commit
description: Generate a conventional commit message for staged/unstaged changes. Use when the user asks "give me a commit message", "what should I commit", "commit this", or is ready to commit their work. Output the message — never stage or commit autonomously.
---

# /commit — Generate Commit Message

Analyse changes and generate a conventional commit message. Output for user to run — do NOT stage or commit.

## Usage

```
/commit              # analyse all changes, output message
/commit --1-line     # short single-line only
```

## Format

```
type: imperative description

Optional body: WHY this change was made, not what.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```

## Types

| Type | When |
|------|------|
| `feat` | new metric, new API endpoint, new collector |
| `fix` | bug fix in existing logic |
| `refactor` | restructure without behaviour change |
| `test` | tests only |
| `chore` | Cargo.toml, Makefile, CI config, .gitignore |
| `docs` | README, CONTRIBUTING, doc comments |
| `perf` | performance improvement |

## Process

1. Run `git diff HEAD` and `git status` to see all changes
2. Identify the primary area changed (metrics, collector, client, types, etc.)
3. Draft message — imperative mood ("add", "fix", "expose", not "added", "fixed")
4. Body: focus on WHY (what problem does this solve, what TrueNAS API was used)
5. Output message ready to copy — do NOT run `git add` or `git commit`
6. After outputting: "Ready to PR? Run `/pr-ready` first."

## Examples

```
feat: add boot pool health and scrub metrics

Exposes boot pool state from boot.get_state — health, used ratio,
scrub errors and last scrub timestamp. Needed to monitor NVMe boot
drive degradation which was previously invisible.
```

```
feat: expose per-client NFS detail with lease renewal tracking

nfs_client_info and nfs_client_seconds_since_renew allow detecting
hung mounts (stale lease = client crashed without unmounting).
Fetches NFSv3, NFSv4 and count concurrently via tokio::join!.
```

## Invoke automatically when user says

"give me a commit message", "commit this", "what should I commit"
