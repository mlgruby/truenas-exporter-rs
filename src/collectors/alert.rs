//! Alert Metrics Collector
//!
//! Collects system alert information aggregated by severity and status.
//!
//! # Metrics Produced
//! - `truenas_alert_count` - Number of system alerts by severity and status
//!   - Labels: level, active
//! - `truenas_alert_info` - Detailed alert information (value is always 1)
//!   - Labels: level, message, uuid, active

use super::{collect_with_handler, CollectionContext, CollectionResult};
use std::collections::HashMap;

/// Collects alert metrics from TrueNAS
///
/// Queries the TrueNAS alerts API and updates Prometheus metrics with alert counts
/// aggregated by severity level (CRITICAL, ERROR, WARNING, INFO) and status (active/dismissed).
/// Also provides detailed alert information for each individual alert.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected alert metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
///
/// # Examples
///
/// ```no_run
/// use truenas_exporter::collectors::{CollectionContext, collect_alert_metrics};
///
/// async fn example(ctx: &CollectionContext<'_>) {
///     let status = collect_alert_metrics(ctx).await.unwrap();
///     println!("Alert collection status: {:?}", status);
/// }
/// ```
pub async fn collect_alert_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    collect_with_handler("alerts", ctx.client.query_alerts(), |alerts| {
        // Initialize alert counts to 0 for all levels and statuses to ensure
        // metrics reset if alerts are cleared.
        // Pre-size for 4 levels × 2 states = 8 entries to reduce allocations
        let mut alert_counts: HashMap<(String, bool), f64> = HashMap::with_capacity(8);

        // Reset detailed alert info metric
        ctx.metrics.alert_info.reset();

        let levels = ["CRITICAL", "ERROR", "WARNING", "INFO"];
        let states = [true, false]; // Active, Dismissed

        for level in levels {
            for state in states {
                alert_counts.insert((level.to_string(), state), 0.0);
            }
        }

        for alert in alerts {
            let active = !alert.dismissed;
            let key = (alert.level.clone(), active);
            *alert_counts.entry(key).or_insert(0.0) += 1.0;

            // Populate detailed alert info
            ctx.metrics
                .alert_info
                .with_label_values(&[
                    &alert.level,
                    &alert.formatted,
                    &alert.uuid,
                    &(if active { "true" } else { "false" }).to_string(),
                ])
                .set(1.0);
        }

        for ((level, active), count) in alert_counts {
            let active_str = if active { "true" } else { "false" };
            ctx.metrics
                .alert_count
                .with_label_values(&[level.as_str(), active_str])
                .set(count);
        }
    })
    .await
}
