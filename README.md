# TrueNAS Scale Prometheus Exporter

A robust Prometheus metrics exporter for TrueNAS Scale 25.x (Electric Eel) and newer, written in Rust. This exporter connects to the TrueNAS WebSocket API to collect comprehensive metrics for ZFS pools, datasets, sharing services, data protection tasks, and system health.

## Features

- ✅ **WebSocket API Support**: Uses TrueNAS Scale 25.x JSON-RPC 2.0 API (Secure WSS support).
- ✅ **Storage Monitoring**: ZFS Pools, Datasets, VDevs, and Physical Disks (Smart/Temp/IO).
- ✅ **Data Protection**: Cloud Sync (status/progress) and Snapshot Task monitoring.
- ✅ **Service Monitoring**: SMB, NFS, and Application (Apps) status.
- ✅ **System Health**: Alert monitoring, CPU/Mem/Network stats, and uptime.
- ✅ **Performance**: Async design using Tokio and Tungstenite.

## Quick Start

### Prerequisites

- TrueNAS Scale 25.x or later
- API key from TrueNAS (generate under "My API Keys" in the UI)
- Docker and Docker Compose (recommended)

### Using Docker Compose

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/truenas-exporter-rs.git
   cd truenas-exporter-rs
   ```

2. **Create `.env` file**:
   ```bash
   cp .env.example .env
   ```

3. **Edit `.env` with your TrueNAS details**:
   ```env
   TRUENAS_HOST=YOUR_TRUENAS_IP:443
   TRUENAS_API_KEY=your-api-key-here
   RUST_LOG=info
   # Set to true if using HTTPS/WSS (Default for port 443)
   # USE_TLS=true 
   ```

4. **Start the exporter**:
   ```bash
   docker-compose up -d
   ```

5. **Verify it's working**:
   ```bash
   curl http://localhost:9100/metrics
   ```

## Configuration

### Configuration File (config/Default.toml)

```toml
[truenas]
host = "YOUR_TRUENAS_IP:443"
api_key = "your-api-key-here"
use_tls = true
verify_ssl = false  # Set to false for self-signed certs

[server]
addr = "0.0.0.0"
port = 9100

[metrics]
scrape_interval_seconds = 60
collect_pool_metrics = true
collect_system_metrics = true
```

## Authentication & Connection Details

TrueNAS Scale 25.04+ (Electric Eel) has deprecated the REST API in favor of a WebSocket-only architecture. This exporter implements a robust, persistent connection model to handle this correctly.

### 1. Persistent Connection
Unlike traditional REST-based exporters, this exporter maintains a **single, long-lived WebSocket connection** to TrueNAS. It does **not** reconnect for every scrape.
- **Why?** TrueNAS aggressively rate-limits or rejects clients that attempt to authenticate too frequently (e.g., once per scrape).
- **Behavior:** The exporter connects and authenticates *once* at startup. If the connection drops, it automatically attempts to reconnect with exponential backoff.

### 2. Troubleshooting Authentication
If you see `truenas_up 0` and logs showing "Authentication failed", check the following:

#### `RuntimeError: AUTH: unexpected authenticator run state`
If you see this error in the logs (or server-side), it means the **server-side session state for your API key is corrupted**. This often happens if a client previously hammered the endpoint with rapid connection attempts.
**Fix:**
1.  Log into TrueNAS UI.
2.  Go to **My API Keys**.
3.  **Delete** the problematic API key.
4.  **Generate a NEW API key**.
5.  Update your `.env` or `config.toml` with the new key.
6.  Restart the exporter.

#### `[ENOTAUTHENTICATED]`
This is a standard failure. Check that:
- You pasted the API key correctly.
- The user associated with the key has permission to access the API.
- You are using `wss://` (TLS) if your server requires it (default for API keys).

## Metrics Exposed

### 1. Storage (ZFS & Disks)

- `truenas_pool_info` (Health, Capacity)
- `truenas_pool_last_scrub_seconds`, `truenas_pool_scrub_errors`
- `truenas_pool_vdev_error_count` (Read/Write/Checksum errors)
- `truenas_dataset_used_bytes`, `truenas_dataset_compression_ratio`
- `truenas_disk_temperature_celsius`, `truenas_disk_read/write_bytes_per_second`
- `truenas_smart_test_result` (Pass/Fail)

### 2. Data Protection (Backups)

- `truenas_cloud_sync_status` (Active tasks status)
- `truenas_cloud_sync_progress_percent` (Real-time progress)
- `truenas_snapshot_task_status` (Snapshot success)

### 3. Services & Sharing

- `truenas_share_smb_enabled` (Labels: `name`, `path`)
- `truenas_share_nfs_enabled` (Labels: `path`)
- `truenas_app_status` (Standard Apps like Immich, Jellyfin)
- `truenas_service_status` (SSH, NFS, CIFS daemon status)

### 4. System Health

- `truenas_alert_count` (Active/Dismissed alerts by level)
- `truenas_system_cpu_usage_percent`, `truenas_system_load_average`
- `truenas_network_rx/tx_bytes_per_second`

## Limitations / Future Work

- **Experimental Containers**: TrueNAS Scale "Containers" (systemd-nspawn/sandboxes) API is not currently exposed in a stable way. Use Standard Apps for monitoring.
- **Per-App Resources**: Detailed CPU/Memory usage per specific App is unavailable via the current API.
- **SMART Attributes**: Only Pass/Fail status is monitored. Detailed attribute normalized (e.g. Wear Level) is complex due to vendor differences.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
