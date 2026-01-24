//! Metrics Collectors
//!
//! This module contains specialized collectors for different categories of TrueNAS metrics.
//! Each collector is responsible for querying a specific TrueNAS API endpoint and updating
//! the corresponding Prometheus metrics.
//!
//! # Architecture
//!
//! Collectors follow a consistent pattern:
//! - Accept a `CollectionContext` containing shared state
//! - Query the TrueNAS API
//! - Update Prometheus metrics using helper methods
//! - Return `CollectionResult` (Ok(true) on success, Ok(false) on failure)
//!
//! # Error Handling
//!
//! Individual collector failures are non-fatal - they log warnings and return Ok(false).
//! This ensures partial metrics are still exposed even if some APIs are unavailable.

use crate::config::MetricsConfig;
use crate::metrics::MetricsCollector;
use crate::truenas::TrueNasClient;
use tracing::{info, warn};

/// Shared context passed to all collectors
///
/// This struct uses public fields for ergonomic access patterns.
/// All fields are immutable references, so no invariants can be violated.
#[derive(Clone, Copy)]
pub struct CollectionContext<'a> {
    /// TrueNAS API client for querying endpoints
    pub client: &'a TrueNasClient,
    /// Metrics collector for updating Prometheus metrics
    pub metrics: &'a MetricsCollector,
    /// Metrics configuration (feature flags, intervals, etc.)
    pub config: &'a MetricsConfig,
}

/// Status of a metrics collection operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionStatus {
    /// Metrics were successfully collected and updated
    Success,
    /// Collection failed but is non-fatal (already logged as warning)
    Failed,
}

/// Result type for collector functions
///
/// - `Ok(CollectionStatus::Success)` = Collection succeeded
/// - `Ok(CollectionStatus::Failed)` = Collection failed but non-fatal (logged as warning)
/// - `Err(_)` = Fatal error (should propagate)
pub type CollectionResult = Result<CollectionStatus, anyhow::Error>;

/// Helper to reduce boilerplate in collectors
///
/// Wraps API queries with consistent error handling:
/// - On success: processes data, logs success, returns `CollectionStatus::Success`
/// - On error: logs warning, returns `CollectionStatus::Failed` (non-fatal)
///
/// # Arguments
///
/// * `name` - Name of the metric type being collected (for logging)
/// * `query_future` - Async API call that returns data
/// * `process` - Function to process the data and update metrics
///
/// # Examples
///
/// ```no_run
/// # use truenas_exporter::collectors::*;
/// async fn example(ctx: &CollectionContext<'_>) -> CollectionResult {
///     collect_with_handler(
///         "pools",
///         ctx.client.query_pools(),
///         |pools| {
///             for pool in pools {
///                 // Update metrics...
///             }
///         },
///     ).await
/// }
/// ```
pub async fn collect_with_handler<T, F, P, E>(
    name: &str,
    query_future: F,
    process: P,
) -> CollectionResult
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    P: FnOnce(T),
{
    match query_future.await {
        Ok(data) => {
            process(data);
            info!("Updated {} metrics", name);
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query {}: {}", name, e);
            Ok(CollectionStatus::Failed)
        }
    }
}

// Collector modules
pub mod alert;
pub mod app;
pub mod cloud_sync;
pub mod dataset;
pub mod disk;
pub mod network_interface;
pub mod pool;
pub mod service;
pub mod share;
pub mod smart;
pub mod snapshot;
pub mod system_info;
pub mod system_reporting;

// Re-export collector functions for convenient access
pub use alert::collect_alert_metrics;
pub use app::collect_app_metrics;
pub use cloud_sync::collect_cloud_sync_metrics;
pub use dataset::collect_dataset_metrics;
pub use disk::collect_disk_metrics;
pub use network_interface::collect_network_interface_metrics;
pub use pool::collect_pool_metrics;
pub use service::collect_service_metrics;
pub use share::collect_share_metrics;
pub use smart::collect_smart_metrics;
pub use snapshot::collect_snapshot_metrics;
pub use system_info::collect_system_info_metrics;
pub use system_reporting::collect_system_reporting_metrics;
