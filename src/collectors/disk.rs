//! Disk Information Metrics Collector
//!
//! Collects disk information including serial numbers, models, and sizes.
//!
//! # Metrics Produced
//! - `truenas_disk_info` - Disk information (value is always 1)
//!   - Labels: disk, serial, model, size

use super::{collect_with_handler, CollectionContext, CollectionResult};

/// Collects disk information metrics from TrueNAS
///
/// Queries the TrueNAS disks API and updates Prometheus metrics with disk
/// information including serial numbers, models, and sizes.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected disk metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_disk_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    collect_with_handler("disks", ctx.client.query_disks(), |disks| {
        for disk in disks {
            // Set disk info metric
            let size_str = disk.size.to_string();
            ctx.metrics
                .disk_info
                .with_label_values(&[&disk.name, &disk.serial, &disk.model, &size_str])
                .set(1);
        }
    })
    .await
}
