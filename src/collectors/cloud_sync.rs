//! Cloud Sync Task Metrics Collector
//!
//! Collects status and progress information for cloud sync tasks.
//!
//! # Metrics Produced
//! - `truenas_cloud_sync_status` - Cloud Sync Task Status (1=Active)
//!   - Labels: description, state
//! - `truenas_cloud_sync_progress_percent` - Cloud Sync Progress Percentage
//!   - Labels: description

use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

/// Collects cloud sync task metrics from TrueNAS
///
/// Queries the TrueNAS cloud sync API and updates Prometheus metrics with task status
/// and progress information. Resets metrics before collection to clear stale state labels.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected cloud sync metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (typically means no tasks configured)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_cloud_sync_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_cloud_sync_tasks().await {
        Ok(tasks) => {
            // Reset metrics to clear stale state labels
            ctx.metrics.cloud_sync_status.reset();
            ctx.metrics.cloud_sync_progress.reset();

            for task in tasks {
                if let Some(job) = &task.job {
                    ctx.metrics
                        .cloud_sync_status
                        .with_label_values(&[&task.description, &job.state])
                        .set(1.0);

                    if let Some(progress) = &job.progress {
                        if let Some(pct) = progress.percent {
                            ctx.metrics
                                .cloud_sync_progress
                                .with_label_values(&[&task.description])
                                .set(pct);
                        }
                    }
                }
            }
            info!("Updated cloud sync task metrics");
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query cloud sync tasks: {}", e);
            Ok(CollectionStatus::Failed)
        }
    }
}
