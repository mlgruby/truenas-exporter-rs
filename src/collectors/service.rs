//! Service Status Metrics Collector
//!
//! Collects status information for system services (SSH, NFS daemon, etc.).
//!
//! # Metrics Produced
//! - `truenas_service_status` - Service status (0=stopped, 1=running)
//!   - Labels: service

use super::{collect_with_handler, CollectionContext, CollectionResult};

/// Collects system service status metrics from TrueNAS
///
/// Queries the TrueNAS services API and updates Prometheus metrics with service
/// status (running/stopped) for system services like SSH, NFS daemon, etc.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected service metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_service_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    collect_with_handler("services", ctx.client.query_services(), |services| {
        for service in services {
            let status_value = if service.state.to_uppercase() == "RUNNING" {
                1
            } else {
                0
            };
            ctx.metrics
                .service_status
                .with_label_values(&[&service.service])
                .set(status_value);
        }
    })
    .await
}
