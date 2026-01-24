//! Dataset Metrics Collector
//!
//! Collects ZFS dataset usage, compression, and encryption information.
//!
//! # Metrics Produced
//! - `truenas_dataset_used_bytes` - Used bytes of the dataset
//!   - Labels: dataset, pool
//! - `truenas_dataset_available_bytes` - Available bytes for the dataset
//!   - Labels: dataset, pool
//! - `truenas_dataset_compression_ratio` - Compression ratio of the dataset
//!   - Labels: dataset, pool
//! - `truenas_dataset_encrypted` - Encryption status (1=encrypted, 0=unencrypted)
//!   - Labels: dataset, pool

use super::{collect_with_handler, CollectionContext, CollectionResult};

/// Collects ZFS dataset metrics from TrueNAS
///
/// Queries the TrueNAS datasets API and updates Prometheus metrics with dataset
/// usage, availability, compression ratios, and encryption status.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected dataset metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_dataset_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    collect_with_handler("datasets", ctx.client.query_datasets(), |datasets| {
        for dataset in datasets {
            let pool_name = dataset.name.split('/').next().unwrap_or(&dataset.name);

            if let Some(used) = &dataset.used {
                ctx.metrics.set_gauge(
                    &ctx.metrics.dataset_used_bytes,
                    &[dataset.name.as_str(), pool_name],
                    used.parsed as f64,
                );
            }
            if let Some(avail) = &dataset.available {
                ctx.metrics.set_gauge(
                    &ctx.metrics.dataset_available_bytes,
                    &[dataset.name.as_str(), pool_name],
                    avail.parsed as f64,
                );
            }
            if let Some(ratio) = &dataset.compressratio {
                if let Ok(val) = ratio.parsed.parse::<f64>() {
                    ctx.metrics.set_gauge(
                        &ctx.metrics.dataset_compression_ratio,
                        &[dataset.name.as_str(), pool_name],
                        val,
                    );
                }
            }
            ctx.metrics.set_bool_metric(
                &ctx.metrics.dataset_encrypted,
                &[dataset.name.as_str(), pool_name],
                dataset.encrypted,
            );
        }
    })
    .await
}
