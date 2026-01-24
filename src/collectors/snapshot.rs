//! Snapshot Task Metrics Collector
//!
//! Collects status information for snapshot tasks.
//!
//! # Metrics Produced
//! - `truenas_snapshot_task_status` - Snapshot Task Status (1=Active)
//!   - Labels: dataset, state

use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

/// Collects snapshot task metrics from TrueNAS
///
/// Queries the TrueNAS snapshot tasks API and updates Prometheus metrics with
/// task status. Resets metrics before collection to clear stale state labels
/// (e.g., RUNNING -> FINISHED transitions).
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected snapshot task metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (typically means no tasks configured)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_snapshot_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_snapshot_tasks().await {
        Ok(tasks) => {
            // Reset metric to clear stale state labels (e.g., RUNNING -> FINISHED transitions)
            ctx.metrics.snapshot_task_status.reset();

            for task in tasks {
                if let Some(st) = &task.state {
                    ctx.metrics
                        .snapshot_task_status
                        .with_label_values(&[&task.dataset, &st.state])
                        .set(1.0);
                }
            }
            info!("Updated snapshot task metrics");
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query snapshot tasks: {}", e);
            Ok(CollectionStatus::Failed)
        }
    }
}
