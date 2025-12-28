# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Comprehensive metric collection for:
  - Storage: Pools (status, capacity, scrub), Datasets (usage, compression), Disks (temperature, SMART).
  - Data Protection: Cloud Sync tasks, Snapshot tasks (status, success/failure).
  - Services: SMB, NFS, SSH status.
  - System: Alerts (critical, error, warning), System Info (CPU/Mem via loadavg/usage), Network stats.
- Unit tests for API type deserialization.
- Integration test for configuration loading.
- `lib.rs` refactoring to support library usage and better testing.
- TLS support (defaulting to port 443 w/ generic `YOUR_TRUENAS_IP` default).

### Changed

- Refactored `src/main.rs` to delegate logic to `src/lib.rs`.
- Updated `README.md` with detailed Feature List and Configuration guide.
- Removed sensitive internal IP addresses from codebase.
