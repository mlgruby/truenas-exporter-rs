//! Pool Metrics Collector
//!
//! Collects ZFS pool health, capacity, scrub information, and VDev error counts.
//!
//! # Metrics Produced
//! - `truenas_pool_health` - Pool health status (1=healthy, 0=unhealthy)
//!   - Labels: pool, status
//! - `truenas_pool_capacity_bytes` - Total storage capacity of the ZFS pool
//!   - Labels: pool
//! - `truenas_pool_allocated_bytes` - Used storage capacity of the ZFS pool
//!   - Labels: pool
//! - `truenas_pool_free_bytes` - Free storage capacity of the ZFS pool
//!   - Labels: pool
//! - `truenas_pool_scrub_errors` - Number of errors found during last ZFS scrub
//!   - Labels: pool
//! - `truenas_pool_last_scrub_seconds` - Timestamp of the last ZFS scrub
//!   - Labels: pool
//! - `truenas_pool_vdev_error_count` - ZFS vdev error counts (read/write/checksum)
//!   - Labels: pool, vdev, type

use super::{CollectionContext, CollectionResult, CollectionStatus};
use crate::metrics::MetricsCollector;
use crate::truenas::types::VDev;
use serde_json;
use tracing::{info, warn};

/// Recursively collects VDev error statistics
///
/// Traverses the VDev tree and updates Prometheus metrics for each VDev's
/// read, write, and checksum error counts.
///
/// # Arguments
///
/// * `pool_name` - Name of the pool containing this VDev
/// * `vdev` - VDev structure to process (will recursively process children)
/// * `metrics` - Metrics collector to update
fn collect_vdev_stats(pool_name: &str, vdev: &VDev, metrics: &MetricsCollector) {
    let name = vdev
        .disk
        .as_deref()
        .or(vdev.device.as_deref())
        .unwrap_or(&vdev.name);

    if let Some(stats) = &vdev.stats {
        metrics
            .pool_vdev_error_count
            .with_label_values(&[pool_name, name, "read"])
            .set(stats.read_errors as f64);
        metrics
            .pool_vdev_error_count
            .with_label_values(&[pool_name, name, "write"])
            .set(stats.write_errors as f64);
        metrics
            .pool_vdev_error_count
            .with_label_values(&[pool_name, name, "checksum"])
            .set(stats.checksum_errors as f64);
    }
    for child in &vdev.children {
        collect_vdev_stats(pool_name, child, metrics);
    }
}

/// Collects ZFS pool metrics from TrueNAS
///
/// Queries the TrueNAS pools API and updates Prometheus metrics with pool health,
/// capacity, scrub information, and VDev error counts. Recursively processes VDev
/// topology to collect error statistics for all devices.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected pool metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_pool_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_pools().await {
        Ok(pools) => {
            for pool in pools {
                let health_value = if pool.healthy { 1.0 } else { 0.0 };

                ctx.metrics
                    .pool_health
                    .with_label_values(&[&pool.name, &pool.status])
                    .set(health_value);

                ctx.metrics.set_gauge(
                    &ctx.metrics.pool_capacity_bytes,
                    &[&pool.name],
                    pool.size as f64,
                );

                ctx.metrics.set_gauge(
                    &ctx.metrics.pool_allocated_bytes,
                    &[&pool.name],
                    pool.allocated as f64,
                );

                ctx.metrics.set_gauge(
                    &ctx.metrics.pool_free_bytes,
                    &[&pool.name],
                    pool.free as f64,
                );

                // Collect Scan Stats (Errors & Last Scrub)
                if let Some(scan) = &pool.scan {
                    ctx.metrics.set_gauge(
                        &ctx.metrics.pool_scrub_errors,
                        &[&pool.name],
                        scan.errors.unwrap_or_default() as f64,
                    );

                    if let Some(serde_json::Value::Object(map)) = &scan.end_time {
                        if let Some(serde_json::Value::Number(num)) = map.get("$date") {
                            if let Some(millis) = num.as_u64() {
                                ctx.metrics.set_gauge(
                                    &ctx.metrics.pool_last_scrub_seconds,
                                    &[&pool.name],
                                    (millis / 1000) as f64,
                                );
                            }
                        }
                    }
                }

                // Collect VDev Errors (Recursive)
                if let Some(topology) = &pool.topology {
                    for vdev in &topology.data {
                        collect_vdev_stats(&pool.name, vdev, ctx.metrics);
                    }
                }

                info!(
                    "Updated metrics for pool: {} (status: {}, healthy: {})",
                    pool.name, pool.status, pool.healthy
                );
            }
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query pools: {}", e);
            Ok(CollectionStatus::Failed)
        }
    }
}
