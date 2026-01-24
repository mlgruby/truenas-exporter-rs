//! Server integration tests
//!
//! Tests for HTTP endpoints and server functionality.

use truenas_exporter::metrics::MetricsCollector;

/// Helper to create a test metrics instance
fn create_test_metrics() -> MetricsCollector {
    MetricsCollector::new().expect("Failed to create metrics")
}

#[tokio::test]
async fn test_metrics_endpoint_returns_prometheus_format() {
    // Given: A metrics collector with some metrics set
    let metrics = create_test_metrics();
    metrics.up.set(1.0);
    metrics
        .pool_health
        .with_label_values(&["test-pool", "ONLINE"])
        .set(1.0);

    // When: Rendering metrics to Prometheus format
    let rendered = metrics.render().expect("Failed to render metrics");

    // Then: Output should be valid Prometheus format
    assert!(rendered.contains("# HELP"), "Missing HELP comment");
    assert!(rendered.contains("# TYPE"), "Missing TYPE comment");
    assert!(rendered.contains("truenas_up 1"), "Missing up metric");
    assert!(
        rendered.contains("truenas_pool_health"),
        "Missing pool_health metric"
    );
    assert!(
        rendered.contains("{pool=\"test-pool\",status=\"ONLINE\"}"),
        "Labels not in correct format"
    );
}

#[test]
fn test_metrics_rendering_is_stable() {
    // Given: A metrics collector with a metric set
    let metrics = create_test_metrics();
    metrics.up.set(1.0);

    // When: Rendering the same metrics twice
    let render1 = metrics.render().expect("First render failed");
    let render2 = metrics.render().expect("Second render failed");

    // Then: Both renderings should be identical
    assert_eq!(render1, render2, "Metrics rendering is not stable");
}

#[test]
fn test_metrics_up_gauge_default() {
    let metrics = create_test_metrics();

    // Check default value (should be 0)
    let rendered = metrics.render().expect("Failed to render");

    // The up metric should exist (it's a plain Gauge, always rendered)
    assert!(
        rendered.contains("truenas_up"),
        "up metric should always be present"
    );
}

#[test]
fn test_health_check_logic() {
    let metrics = create_test_metrics();

    // Simulate healthy state
    metrics.up.set(1.0);
    let up_value = metrics.up.get();
    assert!(up_value > 0.0, "Health check should pass when up > 0");

    // Simulate unhealthy state
    metrics.up.set(0.0);
    let up_value = metrics.up.get();
    assert!(up_value == 0.0, "Health check should fail when up == 0");
}

#[test]
fn test_multiple_pools_metrics() {
    let metrics = create_test_metrics();

    // Add multiple pools
    let pools = vec![
        ("tank", "ONLINE", true),
        ("backup", "DEGRADED", false),
        ("fast", "ONLINE", true),
    ];

    for (name, status, healthy) in pools {
        metrics
            .pool_health
            .with_label_values(&[name, status])
            .set(if healthy { 1.0 } else { 0.0 });

        metrics
            .pool_capacity_bytes
            .with_label_values(&[name])
            .set(1_000_000_000_000.0);
    }

    let rendered = metrics.render().expect("Failed to render");

    // Verify all pools are present
    assert!(rendered.contains("pool=\"tank\""));
    assert!(rendered.contains("pool=\"backup\""));
    assert!(rendered.contains("pool=\"fast\""));

    // Verify statuses
    assert!(rendered.contains("status=\"ONLINE\""));
    assert!(rendered.contains("status=\"DEGRADED\""));
}

#[test]
fn test_alert_count_all_levels() {
    // Given: A metrics collector with alert counts set for all severity levels
    let metrics = create_test_metrics();

    // Set counts for all alert levels
    let levels = ["CRITICAL", "ERROR", "WARNING", "INFO"];
    for level in levels {
        metrics
            .alert_count
            .with_label_values(&[level, "true"])
            .set(5.0);
        metrics
            .alert_count
            .with_label_values(&[level, "false"])
            .set(2.0);
    }

    // When: Rendering metrics to Prometheus format
    let rendered = metrics.render().expect("Failed to render");

    // Then: All alert levels and states should be present in output
    // Verify all levels are present
    for level in levels {
        assert!(
            rendered.contains(&format!("level=\"{}\"", level)),
            "Missing level: {}",
            level
        );
    }

    // Verify active/dismissed states
    assert!(rendered.contains("active=\"true\""));
    assert!(rendered.contains("active=\"false\""));
}

#[test]
fn test_system_reporting_metrics() {
    // Given: A metrics collector with system reporting data set (CPU, memory, disk, network)
    let metrics = create_test_metrics();

    // CPU usage
    metrics
        .system_cpu_usage_percent
        .with_label_values(&["user"])
        .set(45.5);
    metrics
        .system_cpu_usage_percent
        .with_label_values(&["system"])
        .set(12.3);

    // CPU temperature
    metrics
        .system_cpu_temperature_celsius
        .with_label_values(&["0"])
        .set(55.0);

    // Memory
    metrics
        .system_memory_bytes
        .with_label_values(&["available"])
        .set(8_000_000_000.0);
    metrics
        .system_memory_bytes
        .with_label_values(&["free"])
        .set(4_000_000_000.0);

    // Disk I/O
    metrics
        .disk_read_bytes_per_second
        .with_label_values(&["sda"])
        .set(1_000_000.0);
    metrics
        .disk_write_bytes_per_second
        .with_label_values(&["sda"])
        .set(500_000.0);

    // Network
    metrics
        .network_receive_bytes_per_second
        .with_label_values(&["eth0"])
        .set(10_000_000.0);
    metrics
        .network_transmit_bytes_per_second
        .with_label_values(&["eth0"])
        .set(5_000_000.0);

    // When: Rendering metrics to Prometheus format
    let rendered = metrics.render().expect("Failed to render");

    // Then: All system reporting metrics should be present in output
    // Verify all metrics present
    assert!(rendered.contains("truenas_system_cpu_usage_percent"));
    assert!(rendered.contains("truenas_system_cpu_temperature_celsius"));
    assert!(rendered.contains("truenas_system_memory_bytes"));
    assert!(rendered.contains("truenas_disk_read_bytes_per_second"));
    assert!(rendered.contains("truenas_disk_write_bytes_per_second"));
    assert!(rendered.contains("truenas_network_receive_bytes_per_second"));
    assert!(rendered.contains("truenas_network_transmit_bytes_per_second"));
}

#[test]
fn test_smart_test_metrics() {
    // Given: A metrics collector with SMART test data for multiple disks and test types
    let metrics = create_test_metrics();

    // Set SMART test metrics
    metrics
        .smart_test_status
        .with_label_values(&["sda", "Short"])
        .set(0); // 0 = success

    metrics
        .smart_test_status
        .with_label_values(&["sdb", "Extended"])
        .set(1); // 1 = failed

    metrics
        .smart_test_lifetime_hours
        .with_label_values(&["sda", "Short"])
        .set(12345.0);

    metrics
        .disk_power_on_hours
        .with_label_values(&["sda"])
        .set(12350.0);

    // When: Rendering metrics to Prometheus format
    let rendered = metrics.render().expect("Failed to render");

    // Then: All SMART test metrics should be present with correct labels
    assert!(rendered.contains("truenas_smart_test_status"));
    assert!(rendered.contains("truenas_smart_test_lifetime_hours"));
    assert!(rendered.contains("truenas_disk_power_on_hours"));
    assert!(rendered.contains("test_type=\"Short\""));
    assert!(rendered.contains("test_type=\"Extended\""));
}

#[test]
fn test_no_double_prefix() {
    // Given: A metrics collector with various metrics set
    let metrics = create_test_metrics();

    metrics.up.set(1.0);
    metrics.system_uptime_seconds.set(100.0);

    // When: Rendering metrics to Prometheus format
    let rendered = metrics.render().expect("Failed to render");

    // Then: No metric should have double prefix (truenas_truenas_)
    // Verify no double prefix (truenas_truenas_)
    assert!(
        !rendered.contains("truenas_truenas_"),
        "Found double prefix in metrics"
    );

    // All metrics should have single truenas_ prefix
    for line in rendered.lines() {
        if line.starts_with("truenas_") {
            assert!(
                !line.starts_with("truenas_truenas_"),
                "Double prefix found: {}",
                line
            );
        }
    }
}

#[test]
fn test_cloud_sync_and_snapshot_reset_behavior() {
    // Given: A metrics collector with initial cloud sync and snapshot states
    let metrics = create_test_metrics();

    // Set initial state
    metrics
        .cloud_sync_status
        .with_label_values(&["task1", "RUNNING"])
        .set(1.0);
    metrics
        .snapshot_task_status
        .with_label_values(&["tank/data", "RUNNING"])
        .set(1.0);

    let before = metrics.render().unwrap();
    assert!(before.contains("RUNNING"));

    // When: Resetting metrics and setting new states
    // Reset (simulating state change)
    metrics.cloud_sync_status.reset();
    metrics.snapshot_task_status.reset();

    // Set new state
    metrics
        .cloud_sync_status
        .with_label_values(&["task1", "SUCCESS"])
        .set(1.0);
    metrics
        .snapshot_task_status
        .with_label_values(&["tank/data", "FINISHED"])
        .set(1.0);

    let after = metrics.render().unwrap();

    // Then: Old state labels should be cleared, new states should appear
    // Old state should not appear
    assert!(!after.contains("cloud_sync_status{description=\"task1\",state=\"RUNNING\"}"));
    assert!(!after.contains("snapshot_task_status{dataset=\"tank/data\",state=\"RUNNING\"}"));

    // New state should appear
    assert!(after.contains("state=\"SUCCESS\""));
    assert!(after.contains("state=\"FINISHED\""));
}

#[test]
fn test_vdev_error_tracking() {
    // Given: A metrics collector with VDev error counts for multiple disks and error types
    let metrics = create_test_metrics();

    // Simulate VDev error tracking
    let pool = "tank";
    let vdevs = vec![
        ("sda", "read", 0.0),
        ("sda", "write", 0.0),
        ("sda", "checksum", 0.0),
        ("sdb", "read", 1.0), // Has read error
        ("sdb", "write", 0.0),
        ("sdb", "checksum", 2.0), // Has checksum errors
    ];

    for (vdev, error_type, count) in vdevs {
        metrics
            .pool_vdev_error_count
            .with_label_values(&[pool, vdev, error_type])
            .set(count);
    }

    // When: Rendering metrics to Prometheus format
    let rendered = metrics.render().unwrap();

    // Then: All VDev error metrics should be present with correct counts
    assert!(rendered.contains("truenas_pool_vdev_error_count"));
    assert!(rendered.contains("vdev=\"sda\""));
    assert!(rendered.contains("vdev=\"sdb\""));
    assert!(rendered.contains("type=\"read\""));
    assert!(rendered.contains("type=\"write\""));
    assert!(rendered.contains("type=\"checksum\""));

    // Verify error counts
    assert!(rendered.contains("pool_vdev_error_count{pool=\"tank\",type=\"read\",vdev=\"sdb\"} 1"));
    assert!(
        rendered.contains("pool_vdev_error_count{pool=\"tank\",type=\"checksum\",vdev=\"sdb\"} 2")
    );
}

#[test]
fn test_dataset_compression_ratio() {
    // Given: A metrics collector with compression ratios for multiple datasets
    let metrics = create_test_metrics();

    metrics
        .dataset_compression_ratio
        .with_label_values(&["tank/data", "tank"])
        .set(1.5);

    metrics
        .dataset_compression_ratio
        .with_label_values(&["tank/media", "tank"])
        .set(2.3);

    // When: Rendering metrics to Prometheus format
    let rendered = metrics.render().unwrap();

    // Then: Dataset compression ratio metrics should be present with correct labels
    assert!(rendered.contains("truenas_dataset_compression_ratio"));
    assert!(rendered.contains("dataset=\"tank/data\""));
    assert!(rendered.contains("dataset=\"tank/media\""));
}
