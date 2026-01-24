//! SMART Test Metrics Collector
//!
//! Collects SMART test results and disk power-on hours.
//! Groups tests by description and keeps the one with the highest lifetime.
//!
//! # Metrics Produced
//! - `truenas_smart_test_status` - SMART test status (0=success, 1=failed)
//!   - Labels: disk, test_type
//! - `truenas_smart_test_lifetime_hours` - Disk lifetime hours when the last SMART test was run
//!   - Labels: disk, test_type
//! - `truenas_smart_test_timestamp_seconds` - Unix timestamp when the last SMART test was run
//!   - Labels: disk, test_type
//! - `truenas_disk_power_on_hours` - Total power-on hours for the disk
//!   - Labels: disk

use super::{CollectionContext, CollectionResult, CollectionStatus};
use crate::truenas::types::SmartTestEntry;
use std::collections::HashMap;
use tracing::{info, warn};

/// Collects SMART test results and disk power-on hours from TrueNAS
///
/// Queries the TrueNAS SMART tests API and updates Prometheus metrics with test
/// status, lifetime hours, timestamps, and disk power-on hours. Groups tests by
/// description (test type) and keeps only the most recent test (highest lifetime).
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected SMART metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
///
/// # Note
///
/// This function deduplicates tests by keeping only the test with the highest
/// lifetime hours for each test type (description) per disk.
pub async fn collect_smart_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_smart_tests().await {
        Ok(disks) => {
            for disk in disks {
                let disk_name = disk.name.clone();

                // Group tests by description (which acts as test type, e.g. "Extended offline")
                // and keep the one with the highest lifetime.
                // Pre-size for typical case: 2-4 test types per disk
                let mut latest_tests: HashMap<String, SmartTestEntry> = HashMap::with_capacity(4);

                for test in disk.tests {
                    // Use description as the test type key
                    let key = test.description.clone();
                    match latest_tests.entry(key) {
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert(test);
                        }
                        std::collections::hash_map::Entry::Occupied(mut e) => {
                            if test.lifetime > e.get().lifetime {
                                e.insert(test);
                            }
                        }
                    }
                }

                for (test_type, test) in latest_tests {
                    let status_str = test.status.to_uppercase();
                    let status_value = if status_str == "SUCCESS"
                        || status_str == "COMPLETED WITHOUT ERROR"
                        || status_str == "RUNNING"
                    {
                        0
                    } else {
                        warn!(
                            "SMART test failure/unknown status for disk {} ({}): {}",
                            disk_name, test_type, status_str
                        );
                        1
                    };

                    ctx.metrics
                        .smart_test_status
                        .with_label_values(&[&disk_name, &test_type])
                        .set(status_value);

                    ctx.metrics.set_gauge(
                        &ctx.metrics.smart_test_lifetime_hours,
                        &[&disk_name, &test_type],
                        test.lifetime as f64,
                    );

                    // Calculate and set test timestamp if power_on_hours_ago is available
                    if let Some(hours_ago) = test.power_on_hours_ago {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as f64;
                        let test_timestamp = now - (hours_ago as f64 * 3600.0);

                        ctx.metrics.set_gauge(
                            &ctx.metrics.smart_test_timestamp_seconds,
                            &[&disk_name, &test_type],
                            test_timestamp,
                        );

                        // Calculate current disk power-on hours
                        let current_disk_hours = test.lifetime + hours_ago;
                        ctx.metrics.set_gauge(
                            &ctx.metrics.disk_power_on_hours,
                            &[&disk_name],
                            current_disk_hours as f64,
                        );
                    }
                }
            }
            info!("Updated SMART test metrics");
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query SMART tests: {}", e);
            Ok(CollectionStatus::Failed)
        }
    }
}
