//! System Information Metrics Collector
//!
//! Collects system information including uptime, memory, and load average.
//!
//! # Metrics Produced
//! - `truenas_system_info` - TrueNAS system information (value is always 1)
//! - `truenas_system_uptime_seconds` - System uptime in seconds
//! - `truenas_system_memory_total_bytes` - Total system memory in bytes
//! - `truenas_system_load_average` - System load average
//!   - Labels: period (1m, 5m, 15m)

use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

/// Collects system information metrics from TrueNAS
///
/// Queries the TrueNAS system info API and updates Prometheus metrics with
/// system uptime, total memory, and load averages (1m, 5m, 15m).
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected system info metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_system_info_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_system_info().await {
        Ok(info) => {
            ctx.metrics.system_info.set(1);
            ctx.metrics.system_uptime_seconds.set(info.uptime_seconds);

            // Total memory
            if let Some(physmem) = info.physmem {
                ctx.metrics.system_memory_total_bytes.set(physmem as f64);
            }

            // Load average
            if let Some(loadavg) = info.loadavg {
                if loadavg.len() >= 3 {
                    ctx.metrics
                        .system_load_average
                        .with_label_values(&["1m"])
                        .set(loadavg[0]);
                    ctx.metrics
                        .system_load_average
                        .with_label_values(&["5m"])
                        .set(loadavg[1]);
                    ctx.metrics
                        .system_load_average
                        .with_label_values(&["15m"])
                        .set(loadavg[2]);
                }
            }

            info!(
                "Updated system info: {} ({}) - uptime: {:.0}s",
                info.hostname, info.version, info.uptime_seconds
            );
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query system info: {}", e);
            Ok(CollectionStatus::Failed)
        }
    }
}
