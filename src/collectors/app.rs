//! Application Metrics Collector
//!
//! Collects status information for TrueNAS applications (apps).
//!
//! # Metrics Produced
//! - `truenas_app_status` - Application status (0=stopped, 1=running)
//!   - Labels: app
//! - `truenas_app_update_available` - Application update available (0=no, 1=yes)
//!   - Labels: app

use super::{collect_with_handler, CollectionContext, CollectionResult};

/// Collects application (app) metrics from TrueNAS
///
/// Queries the TrueNAS apps API and updates Prometheus metrics with application
/// status (running/stopped) and update availability information.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected app metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_app_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    collect_with_handler("applications", ctx.client.query_apps(), |apps| {
        for app in apps {
            // 0 = stopped, 1 = running
            let status_value = if app.state.to_uppercase() == "RUNNING" {
                1
            } else {
                0
            };
            ctx.metrics
                .app_status
                .with_label_values(&[&app.name])
                .set(status_value);

            // Update available
            let update_value = if app.update_available { 1 } else { 0 };
            ctx.metrics
                .app_update_available
                .with_label_values(&[&app.name])
                .set(update_value);
        }
    })
    .await
}
