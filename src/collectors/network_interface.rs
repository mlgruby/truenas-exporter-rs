//! Network Interface Metrics Collector
//!
//! Collects network interface information including link state.
//!
//! # Metrics Produced
//! - `truenas_network_interface_info` - Network interface information (value is always 1)
//!   - Labels: interface, link_state

use super::{collect_with_handler, CollectionContext, CollectionResult};

/// Collects network interface metrics from TrueNAS
///
/// Queries the TrueNAS network interfaces API and updates Prometheus metrics
/// with interface information including link state.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected network interface metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_network_interface_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    collect_with_handler(
        "network interfaces",
        ctx.client.query_network_interfaces(),
        |interfaces| {
            for iface in interfaces {
                let link_state = &iface.state.link_state;
                ctx.metrics
                    .network_interface_info
                    .with_label_values(&[&iface.name, link_state])
                    .set(1);
            }
        },
    )
    .await
}
