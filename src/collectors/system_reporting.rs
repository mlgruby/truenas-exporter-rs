//! System Reporting Metrics Collector
//!
//! Collects reporting metrics including CPU usage, CPU temperature, memory usage,
//! disk temperature, disk I/O, and network traffic.
//!
//! This is the most complex collector as it requires:
//! 1. Querying reporting graphs to get available identifiers
//! 2. Building a batch query for all metrics
//! 3. Parsing legend-based results to extract metric values
//!
//! # Metrics Produced
//! - `truenas_system_cpu_usage_percent` - System CPU usage percentage by mode
//!   - Labels: mode
//! - `truenas_system_cpu_temperature_celsius` - System CPU temperature in Celsius
//!   - Labels: cpu
//! - `truenas_system_memory_bytes` - System memory usage in bytes by state
//!   - Labels: state
//! - `truenas_system_memory_used_bytes` - System memory used in bytes (Total - Available)
//! - `truenas_disk_temperature_celsius` - Current temperature of the disk in Celsius
//!   - Labels: device
//! - `truenas_disk_read_bytes_per_second` - Disk read rate in bytes per second
//!   - Labels: device
//! - `truenas_disk_write_bytes_per_second` - Disk write rate in bytes per second
//!   - Labels: device
//! - `truenas_network_receive_bytes_per_second` - Network receive rate in bytes per second
//!   - Labels: interface
//! - `truenas_network_transmit_bytes_per_second` - Network transmit rate in bytes per second
//!   - Labels: interface

use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

/// Collects system reporting metrics from TrueNAS
///
/// This is the most complex collector as it requires a two-step process:
/// 1. Query available reporting graphs to get identifiers
/// 2. Build and execute a batch query for all metrics
/// 3. Parse legend-based results to extract metric values
///
/// Collects CPU usage, CPU temperature, memory usage, disk temperature,
/// disk I/O rates, and network traffic rates.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected reporting metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
///
/// # Implementation Details
///
/// The function builds a batch query containing:
/// - CPU and CPU temperature metrics (no identifiers)
/// - Memory metrics (no identifiers)
/// - Disk temperature metrics (per disk identifier)
/// - Disk I/O metrics (per disk identifier)
/// - Network interface metrics (per interface identifier)
///
/// Results are parsed using legend arrays to map column positions to metric names.
pub async fn collect_system_reporting_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_reporting_graphs().await {
        Ok(graphs) => {
            // Pre-size for typical case: 3 base queries + ~10 disks + ~5 interfaces
            let mut queries = Vec::with_capacity(20);

            // Add CPU and Memory queries
            queries.push(crate::truenas::types::ReportingQuery {
                name: "cpu".to_string(),
                identifier: None,
            });
            queries.push(crate::truenas::types::ReportingQuery {
                name: "cputemp".to_string(),
                identifier: None,
            });
            queries.push(crate::truenas::types::ReportingQuery {
                name: "memory".to_string(),
                identifier: None,
            });

            // Find disk temp, disk I/O, and interface graphs
            for graph in graphs {
                if graph.name == "disktemp" {
                    if let Some(identifiers) = graph.identifiers.as_ref() {
                        for id in identifiers {
                            queries.push(crate::truenas::types::ReportingQuery {
                                name: "disktemp".to_string(),
                                identifier: Some(id.clone()),
                            });
                        }
                    }
                } else if graph.name == "disk" {
                    // Disk I/O
                    if let Some(identifiers) = graph.identifiers.as_ref() {
                        for id in identifiers {
                            queries.push(crate::truenas::types::ReportingQuery {
                                name: "disk".to_string(),
                                identifier: Some(id.clone()),
                            });
                        }
                    }
                } else if graph.name == "interface" {
                    // Network Traffic
                    if let Some(identifiers) = graph.identifiers.as_ref() {
                        for id in identifiers {
                            queries.push(crate::truenas::types::ReportingQuery {
                                name: "interface".to_string(),
                                identifier: Some(id.clone()),
                            });
                        }
                    }
                }
            }

            // Execute batch query if we have queries
            if !queries.is_empty() {
                match ctx.client.query_reporting_data(queries, None).await {
                    Ok(results) => {
                        for res in results {
                            if let Some(last_point) = res.data.last() {
                                match res.name.as_str() {
                                    "cpu" => {
                                        for (i, label) in res.legend.iter().enumerate() {
                                            if let Some(Some(val)) = last_point.get(i) {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.system_cpu_usage_percent,
                                                    &[label],
                                                    *val,
                                                );
                                            }
                                        }
                                    }
                                    "cputemp" => {
                                        for (i, label) in res.legend.iter().enumerate() {
                                            if let Some(Some(val)) = last_point.get(i) {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.system_cpu_temperature_celsius,
                                                    &[label],
                                                    *val,
                                                );
                                            }
                                        }
                                    }
                                    "memory" => {
                                        let mut available_bytes = 0.0;
                                        for (i, label) in res.legend.iter().enumerate() {
                                            if let Some(Some(val)) = last_point.get(i) {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.system_memory_bytes,
                                                    &[label],
                                                    *val,
                                                );

                                                // Capture available memory for calculating used memory
                                                if label == "available" {
                                                    available_bytes = *val;
                                                }
                                            }
                                        }

                                        // Calculate used = total - available
                                        let total = ctx.metrics.system_memory_total_bytes.get();
                                        if total > 0.0 && available_bytes > 0.0 {
                                            ctx.metrics
                                                .system_memory_used_bytes
                                                .set(total - available_bytes);
                                        }
                                    }
                                    "disktemp" => {
                                        // identifier contains the device info
                                        let device = res.identifier.as_deref().unwrap_or("unknown");

                                        // Legend: [time, temperature_value] or similar
                                        if let Some(idx) = res
                                            .legend
                                            .iter()
                                            .position(|l| l == "temperature_value" || l == "value")
                                        {
                                            if let Some(Some(val)) = last_point.get(idx) {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.disk_temperature_celsius,
                                                    &[device],
                                                    *val,
                                                );
                                            }
                                        } else if res.legend.len() > 1 {
                                            // Fallback: assume last column is value
                                            if let Some(Some(val)) = last_point.last() {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.disk_temperature_celsius,
                                                    &[device],
                                                    *val,
                                                );
                                            }
                                        }
                                    }
                                    "disk" => {
                                        // Disk I/O. Legend: ["time", "reads", "writes"]
                                        let device = res.identifier.as_deref().unwrap_or("unknown");

                                        if let Some(idx) =
                                            res.legend.iter().position(|l| l == "reads")
                                        {
                                            if let Some(Some(val)) = last_point.get(idx) {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.disk_read_bytes_per_second,
                                                    &[device],
                                                    *val,
                                                );
                                            }
                                        }
                                        if let Some(idx) =
                                            res.legend.iter().position(|l| l == "writes")
                                        {
                                            if let Some(Some(val)) = last_point.get(idx) {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.disk_write_bytes_per_second,
                                                    &[device],
                                                    *val,
                                                );
                                            }
                                        }
                                    }
                                    "interface" => {
                                        // Network Traffic. Legend: ["time", "received", "sent"]
                                        let interface =
                                            res.identifier.as_deref().unwrap_or("unknown");

                                        if let Some(idx) =
                                            res.legend.iter().position(|l| l == "received")
                                        {
                                            if let Some(Some(val)) = last_point.get(idx) {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.network_receive_bytes_per_second,
                                                    &[interface],
                                                    *val,
                                                );
                                            }
                                        }
                                        if let Some(idx) =
                                            res.legend.iter().position(|l| l == "sent")
                                        {
                                            if let Some(Some(val)) = last_point.get(idx) {
                                                ctx.metrics.set_gauge(
                                                    &ctx.metrics.network_transmit_bytes_per_second,
                                                    &[interface],
                                                    *val,
                                                );
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        info!("Updated reporting metrics (CPU, Mem, Disk Temp, Net, I/O)");
                        return Ok(CollectionStatus::Success);
                    }
                    Err(e) => warn!("Failed to query reporting data: {}", e),
                }
            }
        }
        Err(e) => warn!("Failed to query reporting graphs: {}", e),
    }
    Ok(CollectionStatus::Failed)
}
