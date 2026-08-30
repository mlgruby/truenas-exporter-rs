//! Replication Task Metrics Collector
//!
//! Collects status, progress, and last-run timestamp for ZFS replication tasks.
//!
//! # Metrics Produced
//! - `truenas_replication_status` - Replication Task Status (1=current state)
//!   - Labels: name, direction, transport, state
//! - `truenas_replication_progress_percent` - Replication Progress Percentage
//!   - Labels: name
//!   - Only emitted while a middleware job is active.
//! - `truenas_replication_last_run_timestamp_seconds` - Unix timestamp of the last run
//!   - Labels: name

use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

/// Collects replication task metrics from TrueNAS.
///
/// Queries the TrueNAS `replication.query` endpoint and updates Prometheus metrics
/// with task status (sourced from the top-level zettarepl `state.state` field, which is
/// always populated — unlike the embedded `job` block which only exists mid-run),
/// in-flight progress (when a job is active), and the timestamp of the last run
/// (decoded from `state.datetime` which uses the `{"$date": ms_epoch}` wire format).
///
/// Resets all replication metrics before collection so stale state labels (e.g.
/// `RUNNING` → `FINISHED` transitions) and stale progress lines are cleared.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected replication metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (typically means no
///   tasks configured or the user lacks permission to query replication)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_replication_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_replication_tasks().await {
        Ok(tasks) => {
            ctx.metrics.replication_status.reset();
            ctx.metrics.replication_progress.reset();
            ctx.metrics.replication_last_run_seconds.reset();

            for task in tasks {
                if let Some(state) = &task.state {
                    ctx.metrics
                        .replication_status
                        .with_label_values(&[
                            &task.name,
                            &task.direction,
                            &task.transport,
                            &state.state,
                        ])
                        .set(1.0);

                    if let Some(serde_json::Value::Object(map)) = &state.datetime {
                        if let Some(serde_json::Value::Number(num)) = map.get("$date") {
                            if let Some(millis) = num.as_u64() {
                                ctx.metrics
                                    .replication_last_run_seconds
                                    .with_label_values(&[&task.name])
                                    .set((millis / 1000) as f64);
                            }
                        }
                    }
                }

                if let Some(job) = &task.job {
                    if let Some(progress) = &job.progress {
                        if let Some(pct) = progress.percent {
                            ctx.metrics
                                .replication_progress
                                .with_label_values(&[&task.name])
                                .set(pct);
                        }
                    }
                }
            }
            info!("Updated replication task metrics");
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query replication tasks: {}", e);
            Ok(CollectionStatus::Failed)
        }
    }
}
