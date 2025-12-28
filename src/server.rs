//! HTTP Server and Metrics Collection
//!
//! This module implements the Prometheus exporter HTTP server and the metric collection loop.
//!
//! # Architecture
//!
//! - **HTTP Server**: Axum-based server exposing `/metrics`, `/health`, and `/` endpoints
//! - **Collection Loop**: Background task that periodically queries TrueNAS API and updates metrics
//! - **State Management**: Shared state (config, metrics, client) using Arc for thread-safety
//!
//! # Endpoints
//!
//! - `GET /` - HTML landing page with links to metrics and health
//! - `GET /metrics` - Prometheus metrics in text format
//! - `GET /health` - Health check (returns 200 if TrueNAS is reachable, 503 otherwise)
//!
//! # Metrics Collection
//!
//! The collection loop runs every N seconds (configured via `scrape_interval_seconds`) and:
//! 1. Queries all enabled TrueNAS API endpoints
//! 2. Updates Prometheus metrics with the latest values
//! 3. Sets `truenas_up` to 1 if any query succeeds, 0 if all fail
//!
//! # Error Handling
//!
//! Individual API failures are logged as warnings but don't stop the collection loop.
//! This ensures partial metrics are still exposed even if some APIs are unavailable.

use crate::config::Config;
use crate::metrics::MetricsCollector;
use crate::truenas::TrueNasClient;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn}; // Added for serde_json::Value

// Helper function to recursively collect VDev stats
fn collect_vdev_stats(
    pool_name: &str,
    vdev: &crate::truenas::types::VDev,
    metrics: &MetricsCollector,
) {
    let name = vdev
        .disk
        .as_deref()
        .or(vdev.device.as_deref())
        .unwrap_or(&vdev.name);

    if let Some(stats) = &vdev.stats {
        metrics
            .pool_vdev_error_count
            .with_label_values(&[pool_name, name, "read"])
            .set(stats.read_errors as f64);
        metrics
            .pool_vdev_error_count
            .with_label_values(&[pool_name, name, "write"])
            .set(stats.write_errors as f64);
        metrics
            .pool_vdev_error_count
            .with_label_values(&[pool_name, name, "checksum"])
            .set(stats.checksum_errors as f64);
    }
    for child in &vdev.children {
        collect_vdev_stats(pool_name, child, metrics);
    }
}

#[derive(Clone)]
struct AppState {
    config: Config,
    metrics: MetricsCollector,
    client: Arc<TrueNasClient>,
}

pub async fn start(config: Config) -> anyhow::Result<()> {
    let metrics = MetricsCollector::new()?;
    let client = Arc::new(TrueNasClient::new(config.truenas.clone()));

    let state = AppState {
        config: config.clone(),
        metrics: metrics.clone(),
        client: client.clone(),
    };

    // Start background metrics collection
    let collection_state = state.clone();
    tokio::spawn(async move {
        collect_metrics_loop(collection_state).await;
    });

    // Build the router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    // Start the server
    let addr = format!("{}:{}", config.server.addr, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Metrics server listening on {}", addr);
    info!("Metrics available at http://{}/metrics", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn collect_metrics_loop(state: AppState) {
    let mut ticker = interval(Duration::from_secs(
        state.config.metrics.scrape_interval_seconds,
    ));

    loop {
        ticker.tick().await;

        if let Err(e) = collect_metrics(&state).await {
            error!("Failed to collect metrics: {}", e);
            state.metrics.up.set(0.0);
        } else {
            state.metrics.up.set(1.0);
        }
    }
}

async fn collect_metrics(state: &AppState) -> anyhow::Result<()> {
    info!("Collecting metrics from TrueNAS");

    let mut any_success = false;

    // Collect pool metrics
    if state.config.metrics.collect_pool_metrics {
        match state.client.query_pools().await {
            Ok(pools) => {
                any_success = true;
                for pool in pools {
                    let health_value = if pool.healthy { 1.0 } else { 0.0 };

                    state
                        .metrics
                        .pool_health
                        .with_label_values(&[&pool.name, &pool.status])
                        .set(health_value);

                    state
                        .metrics
                        .pool_capacity_bytes
                        .with_label_values(&[&pool.name])
                        .set(pool.size as f64);

                    state
                        .metrics
                        .pool_allocated_bytes
                        .with_label_values(&[&pool.name])
                        .set(pool.allocated as f64);

                    state
                        .metrics
                        .pool_free_bytes
                        .with_label_values(&[&pool.name])
                        .set(pool.free as f64);

                    // Collect Scan Stats (Errors & Last Scrub)
                    if let Some(scan) = &pool.scan {
                        state
                            .metrics
                            .pool_scrub_errors
                            .with_label_values(&[&pool.name])
                            .set(scan.errors.unwrap_or(0) as f64);

                        if let Some(serde_json::Value::Object(map)) = &scan.end_time {
                            if let Some(serde_json::Value::Number(num)) = map.get("$date") {
                                if let Some(millis) = num.as_u64() {
                                    state
                                        .metrics
                                        .pool_last_scrub_seconds
                                        .with_label_values(&[&pool.name])
                                        .set((millis / 1000) as f64);
                                }
                            }
                        }
                    }

                    // Collect VDev Errors (Recursive)
                    if let Some(topology) = &pool.topology {
                        for vdev in &topology.data {
                            collect_vdev_stats(&pool.name, vdev, &state.metrics);
                        }
                    }

                    info!(
                        "Updated metrics for pool: {} (status: {}, healthy: {})",
                        pool.name, pool.status, pool.healthy
                    );
                }
            }
            Err(e) => {
                warn!("Failed to query pools: {}", e);
            }
        }
    }

    // Collect Dataset Metrics
    match state.client.query_datasets().await {
        Ok(datasets) => {
            for dataset in datasets {
                let pool_name = dataset.name.split('/').next().unwrap_or(&dataset.name);

                if let Some(used) = &dataset.used {
                    state
                        .metrics
                        .dataset_used_bytes
                        .with_label_values(&[dataset.name.as_str(), pool_name])
                        .set(used.parsed as f64);
                }
                if let Some(avail) = &dataset.available {
                    state
                        .metrics
                        .dataset_available_bytes
                        .with_label_values(&[dataset.name.as_str(), pool_name])
                        .set(avail.parsed as f64);
                }
                if let Some(ratio) = &dataset.compressratio {
                    if let Ok(val) = ratio.parsed.parse::<f64>() {
                        state
                            .metrics
                            .dataset_compression_ratio
                            .with_label_values(&[dataset.name.as_str(), pool_name])
                            .set(val);
                    }
                }
                state
                    .metrics
                    .dataset_encrypted
                    .with_label_values(&[dataset.name.as_str(), pool_name])
                    .set(if dataset.encrypted { 1.0 } else { 0.0 });
            }
            info!("Updated dataset metrics");
        }
        Err(e) => {
            warn!("Failed to query datasets: {}", e);
        }
    }

    // Collect Share Metrics
    match state.client.query_smb_shares().await {
        Ok(shares) => {
            for share in shares {
                state
                    .metrics
                    .share_smb_enabled
                    .with_label_values(&[&share.name, &share.path])
                    .set(if share.enabled { 1.0 } else { 0.0 });
            }
        }
        Err(e) => warn!("Failed to query SMB shares: {}", e),
    }

    match state.client.query_nfs_shares().await {
        Ok(shares) => {
            for share in shares {
                state
                    .metrics
                    .share_nfs_enabled
                    .with_label_values(&[&share.path])
                    .set(if share.enabled { 1.0 } else { 0.0 });
            }
        }
        Err(e) => warn!("Failed to query NFS shares: {}", e),
    }
    info!("Updated share metrics");

    // Collect Data Protection Metrics (Cloud Sync, Snapshots)
    if let Ok(tasks) = state.client.query_cloud_sync_tasks().await {
        for task in tasks {
            if let Some(job) = &task.job {
                state
                    .metrics
                    .cloud_sync_status
                    .with_label_values(&[&task.description, &job.state])
                    .set(1.0);

                if let Some(progress) = &job.progress {
                    if let Some(pct) = progress.percent {
                        state
                            .metrics
                            .cloud_sync_progress
                            .with_label_values(&[&task.description])
                            .set(pct);
                    }
                }
            }
        }
    }

    if let Ok(tasks) = state.client.query_snapshot_tasks().await {
        for task in tasks {
            if let Some(st) = &task.state {
                state
                    .metrics
                    .snapshot_task_status
                    .with_label_values(&[&task.dataset, &st.state])
                    .set(1.0);
            }
        }
    }
    info!("Updated data protection metrics");

    // Collect Alerts
    if let Ok(alerts) = state.client.query_alerts().await {
        // Group alerts by level and active status
        let mut alert_counts: std::collections::HashMap<(String, bool), f64> =
            std::collections::HashMap::new();

        for alert in alerts {
            let active = !alert.dismissed;
            let key = (alert.level.clone(), active);
            *alert_counts.entry(key).or_insert(0.0) += 1.0;
        }

        for ((level, active), count) in alert_counts {
            state
                .metrics
                .alert_count
                .with_label_values(&[level.as_str(), if active { "true" } else { "false" }])
                .set(count);
        }
    }
    info!("Updated alert metrics");

    // Collect system metrics
    if state.config.metrics.collect_system_metrics {
        match state.client.query_system_info().await {
            Ok(info) => {
                any_success = true;
                state.metrics.system_info.set(1);
                state.metrics.system_uptime_seconds.set(info.uptime_seconds);

                // Total memory
                if let Some(physmem) = info.physmem {
                    state.metrics.system_memory_total_bytes.set(physmem as f64);
                }

                // Load average
                if let Some(loadavg) = info.loadavg {
                    if loadavg.len() >= 3 {
                        state
                            .metrics
                            .system_load_average
                            .with_label_values(&["1m"])
                            .set(loadavg[0]);
                        state
                            .metrics
                            .system_load_average
                            .with_label_values(&["5m"])
                            .set(loadavg[1]);
                        state
                            .metrics
                            .system_load_average
                            .with_label_values(&["15m"])
                            .set(loadavg[2]);
                    }
                }

                info!(
                    "Updated system info: {} ({}) - uptime: {:.0}s",
                    info.hostname, info.version, info.uptime_seconds
                );
            }
            Err(e) => {
                warn!("Failed to query system info: {}", e);
            }
        }
    }

    // Collect reporting metrics (CPU, Memory, Disk Temp)
    match state.client.query_reporting_graphs().await {
        Ok(graphs) => {
            let mut queries = Vec::new();

            // Add CPU and Memory queries
            queries.push(crate::truenas::types::ReportingQuery {
                name: "cpu".to_string(),
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
                match state.client.query_reporting_data(queries, None).await {
                    Ok(results) => {
                        any_success = true;
                        for res in results {
                            if let Some(last_point) = res.data.last() {
                                match res.name.as_str() {
                                    "cpu" => {
                                        for (i, label) in res.legend.iter().enumerate() {
                                            if let Some(Some(val)) = last_point.get(i) {
                                                state
                                                    .metrics
                                                    .system_cpu_usage_percent
                                                    .with_label_values(&[label])
                                                    .set(*val);
                                            }
                                        }
                                    }
                                    "memory" => {
                                        for (i, label) in res.legend.iter().enumerate() {
                                            if let Some(Some(val)) = last_point.get(i) {
                                                state
                                                    .metrics
                                                    .system_memory_bytes
                                                    .with_label_values(&[label])
                                                    .set(*val);
                                            }
                                        }
                                    }
                                    "disktemp" => {
                                        // identifier contains the info.
                                        let device = res.identifier.as_deref().unwrap_or("unknown");

                                        // Legend: [time, temperature_value] or similar
                                        if let Some(idx) = res
                                            .legend
                                            .iter()
                                            .position(|l| l == "temperature_value" || l == "value")
                                        {
                                            if let Some(Some(val)) = last_point.get(idx) {
                                                state
                                                    .metrics
                                                    .disk_temperature_celsius
                                                    .with_label_values(&[device])
                                                    .set(*val);
                                            }
                                        } else if res.legend.len() > 1 {
                                            // Fallback: assume last column is value
                                            if let Some(Some(val)) = last_point.last() {
                                                state
                                                    .metrics
                                                    .disk_temperature_celsius
                                                    .with_label_values(&[device])
                                                    .set(*val);
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
                                                state
                                                    .metrics
                                                    .disk_read_bytes_per_second
                                                    .with_label_values(&[device])
                                                    .set(*val); // Assuming raw bytes/s or close
                                            }
                                        }
                                        if let Some(idx) =
                                            res.legend.iter().position(|l| l == "writes")
                                        {
                                            if let Some(Some(val)) = last_point.get(idx) {
                                                state
                                                    .metrics
                                                    .disk_write_bytes_per_second
                                                    .with_label_values(&[device])
                                                    .set(*val);
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
                                                state
                                                    .metrics
                                                    .network_receive_bytes_per_second
                                                    .with_label_values(&[interface])
                                                    .set(*val);
                                            }
                                        }
                                        if let Some(idx) =
                                            res.legend.iter().position(|l| l == "sent")
                                        {
                                            if let Some(Some(val)) = last_point.get(idx) {
                                                state
                                                    .metrics
                                                    .network_transmit_bytes_per_second
                                                    .with_label_values(&[interface])
                                                    .set(*val);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        info!("Updated reporting metrics (CPU, Mem, Disk Temp, Net, I/O)");
                    }
                    Err(e) => warn!("Failed to query reporting data: {}", e),
                }
            }
        }
        Err(e) => warn!("Failed to query reporting graphs: {}", e),
    }

    // Collect disk metrics
    match state.client.query_disks().await {
        Ok(disks) => {
            any_success = true;
            for disk in disks {
                // Set disk info metric
                let size_str = disk.size.to_string();
                state
                    .metrics
                    .disk_info
                    .with_label_values(&[&disk.name, &disk.serial, &disk.model, &size_str])
                    .set(1);
            }
            info!("Updated disk metrics");
        }
        Err(e) => {
            warn!("Failed to query disks: {}", e);
        }
    }

    // Collect SMART test results
    match state.client.query_smart_tests().await {
        Ok(tests) => {
            any_success = true;
            for test in tests {
                // 0 = success, 1 = failed
                let status_value = if test.status.to_uppercase() == "SUCCESS" {
                    0
                } else {
                    1
                };
                state
                    .metrics
                    .smart_test_status
                    .with_label_values(&[&test.disk, &test.test_type])
                    .set(status_value);
            }
            info!("Updated SMART test metrics");
        }
        Err(e) => {
            warn!("Failed to query SMART tests: {}", e);
        }
    }

    // Collect application info
    match state.client.query_apps().await {
        Ok(apps) => {
            any_success = true;
            for app in apps {
                // 0 = stopped, 1 = running
                let status_value = if app.state.to_uppercase() == "RUNNING" {
                    1
                } else {
                    0
                };
                state
                    .metrics
                    .app_status
                    .with_label_values(&[&app.name])
                    .set(status_value);

                // Update available
                let update_value = if app.update_available { 1 } else { 0 };
                state
                    .metrics
                    .app_update_available
                    .with_label_values(&[&app.name])
                    .set(update_value);
            }
            info!("Updated application status metrics");
        }
        Err(e) => {
            warn!("Failed to query apps: {}", e);
        }
    }

    // Collect network interface info
    match state.client.query_network_interfaces().await {
        Ok(interfaces) => {
            any_success = true;
            for iface in interfaces {
                let link_state = &iface.state.link_state;
                state
                    .metrics
                    .network_interface_info
                    .with_label_values(&[&iface.name, link_state])
                    .set(1);
            }
            info!("Updated network interface metrics");
        }
        Err(e) => {
            warn!("Failed to query network interfaces: {}", e);
        }
    }

    // Collect service status
    match state.client.query_services().await {
        Ok(services) => {
            any_success = true;
            for service in services {
                let status_value = if service.state.to_uppercase() == "RUNNING" {
                    1
                } else {
                    0
                };
                state
                    .metrics
                    .service_status
                    .with_label_values(&[&service.service])
                    .set(status_value);
            }
            info!("Updated service status metrics");
        }
        Err(e) => {
            warn!("Failed to query services: {}", e);
        }
    }

    // If all queries failed, return error so truenas_up is set to 0
    if !any_success {
        anyhow::bail!("Failed to collect any metrics from TrueNAS - check authentication");
    }

    Ok(())
}

async fn root_handler() -> impl IntoResponse {
    r#"<html>
<head><title>TrueNAS Exporter</title></head>
<body>
<h1>TrueNAS Prometheus Exporter</h1>
<p><a href="/metrics">Metrics</a></p>
<p><a href="/health">Health</a></p>
</body>
</html>"#
}

async fn metrics_handler(State(state): State<AppState>) -> Response {
    match state.metrics.render() {
        Ok(metrics) => metrics.into_response(),
        Err(e) => {
            error!("Failed to render metrics: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error rendering metrics: {}", e),
            )
                .into_response()
        }
    }
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let up_value = state.metrics.up.get();

    if up_value > 0.0 {
        (axum::http::StatusCode::OK, "OK")
    } else {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "TrueNAS API unreachable",
        )
    }
}
