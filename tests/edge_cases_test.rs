//! Edge case tests
//!
//! Tests for unusual but valid data scenarios.

use truenas_exporter::metrics::MetricsCollector;

/// Helper to create a test metrics instance
fn create_test_metrics() -> MetricsCollector {
    MetricsCollector::new().expect("Failed to create metrics")
}

#[test]
fn test_empty_collections_render_without_error() {
    // Given: A metrics collector with no data set
    let metrics = create_test_metrics();

    // When: Rendering metrics
    let result = metrics.render();

    // Then: Should render successfully with only base metrics
    assert!(result.is_ok());
    let rendered = result.unwrap();
    assert!(rendered.contains("# HELP"));
    assert!(rendered.contains("# TYPE"));
}

#[test]
fn test_very_large_pool_size() {
    // Given: A metrics collector with petabyte-scale pool
    let metrics = create_test_metrics();

    let petabytes = 5_000_000_000_000_000.0; // 5 PB
    metrics
        .pool_capacity_bytes
        .with_label_values(&["huge-pool"])
        .set(petabytes);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle very large numbers
    assert!(rendered.contains("truenas_pool_capacity_bytes"));
    assert!(rendered.contains("huge-pool"));
}

#[test]
fn test_pool_name_with_spaces() {
    // Given: A metrics collector with pool name containing spaces
    let metrics = create_test_metrics();

    metrics
        .pool_health
        .with_label_values(&["my pool", "ONLINE"])
        .set(1.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should escape or handle spaces correctly
    assert!(rendered.contains("truenas_pool_health"));
    assert!(rendered.contains("my pool"));
}

#[test]
fn test_pool_name_with_special_characters() {
    // Given: A metrics collector with pool name containing special chars
    let metrics = create_test_metrics();

    // Prometheus labels should handle quotes and backslashes
    metrics
        .pool_health
        .with_label_values(&["pool-with-dashes_and_underscores", "ONLINE"])
        .set(1.0);

    metrics
        .pool_health
        .with_label_values(&["pool.with.dots", "ONLINE"])
        .set(1.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle special characters
    assert!(rendered.contains("pool-with-dashes_and_underscores"));
    assert!(rendered.contains("pool.with.dots"));
}

#[test]
fn test_unicode_in_pool_names() {
    // Given: A metrics collector with Unicode characters in labels
    let metrics = create_test_metrics();

    metrics
        .pool_health
        .with_label_values(&["piscine-été", "ONLINE"]) // French
        .set(1.0);

    metrics
        .pool_health
        .with_label_values(&["プール", "ONLINE"]) // Japanese
        .set(1.0);

    metrics
        .pool_health
        .with_label_values(&["бассейн", "ONLINE"]) // Russian
        .set(1.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle Unicode characters
    assert!(rendered.contains("piscine-été"));
    assert!(rendered.contains("プール"));
    assert!(rendered.contains("бассейн"));
}

#[test]
fn test_very_long_label_values() {
    // Given: A metrics collector with very long label value
    let metrics = create_test_metrics();

    let long_name = "a".repeat(1000);
    let status = "ONLINE";
    metrics
        .pool_health
        .with_label_values(&[long_name.as_str(), status])
        .set(1.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle very long labels
    assert!(rendered.contains("truenas_pool_health"));
    // The long name should be present (Prometheus handles this)
}

#[test]
fn test_zero_values_are_exported() {
    // Given: A metrics collector with explicit zero values
    let metrics = create_test_metrics();

    metrics
        .pool_capacity_bytes
        .with_label_values(&["empty-pool"])
        .set(0.0);

    metrics
        .alert_count
        .with_label_values(&["CRITICAL", "true"])
        .set(0.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Zero values should be exported
    assert!(rendered.contains("empty-pool"));
    assert!(rendered.contains("0"));
}

#[test]
fn test_negative_values_for_gauges() {
    // Given: A metrics collector with negative values (shouldn't happen but test it)
    let metrics = create_test_metrics();

    // Some metrics theoretically could be negative
    metrics
        .pool_capacity_bytes
        .with_label_values(&["test-pool"])
        .set(-1000.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle negative values (even if nonsensical)
    assert!(rendered.contains("test-pool"));
    assert!(rendered.contains("-1000"));
}

#[test]
fn test_floating_point_precision() {
    // Given: A metrics collector with high-precision floating point values
    let metrics = create_test_metrics();

    metrics
        .dataset_compression_ratio
        .with_label_values(&["test-dataset", "test-pool"])
        .set(1.23456789012345);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should preserve reasonable precision
    assert!(rendered.contains("test-dataset"));
    assert!(rendered.contains("1.234")); // Should have some decimal places
}

#[test]
fn test_maximum_label_cardinality() {
    // Given: A metrics collector with many unique label combinations
    let metrics = create_test_metrics();

    // Create 1000 unique pools
    for i in 0..1000 {
        let pool_name = format!("pool-{}", i);
        let status = "ONLINE";
        metrics
            .pool_health
            .with_label_values(&[pool_name.as_str(), status])
            .set(1.0);
    }

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle high cardinality
    assert!(rendered.contains("pool-0"));
    assert!(rendered.contains("pool-999"));
}

#[test]
fn test_empty_string_labels() {
    // Given: A metrics collector with empty string labels
    let metrics = create_test_metrics();

    metrics
        .pool_health
        .with_label_values(&["", "ONLINE"])
        .set(1.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle empty string labels
    assert!(rendered.contains("truenas_pool_health"));
}

#[test]
fn test_metric_reset_with_no_data() {
    // Given: A metrics collector with data, then reset
    let metrics = create_test_metrics();

    metrics
        .cloud_sync_status
        .with_label_values(&["task1", "RUNNING"])
        .set(1.0);

    // When: Resetting without setting new data
    metrics.cloud_sync_status.reset();
    let rendered = metrics.render().expect("Failed to render");

    // Then: Metric should not appear in output
    let count = rendered.matches("cloud_sync_status").count();
    // Only in HELP and TYPE comments, not in actual values
    assert!(count <= 2);
}

#[test]
fn test_concurrent_metric_updates() {
    // Given: A metrics collector
    let metrics = create_test_metrics();

    // When: Updating metrics from multiple "threads" (simulated sequentially)
    for i in 0..100 {
        metrics
            .pool_capacity_bytes
            .with_label_values(&["test-pool"])
            .set(i as f64);
    }

    // Then: Should have latest value
    let rendered = metrics.render().expect("Failed to render");
    assert!(rendered.contains("99"));
}

#[test]
fn test_alert_level_case_sensitivity() {
    // Given: A metrics collector with different case alert levels
    let metrics = create_test_metrics();

    metrics
        .alert_count
        .with_label_values(&["CRITICAL", "true"])
        .set(5.0);

    metrics
        .alert_count
        .with_label_values(&["critical", "true"])
        .set(3.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Both should be treated as different labels
    assert!(rendered.contains("CRITICAL"));
    assert!(rendered.contains("critical"));
}

#[test]
fn test_null_equivalent_handling() {
    // Given: A metrics collector where we don't set optional metrics
    let metrics = create_test_metrics();

    // Only set some metrics, leave others unset
    metrics.up.set(1.0);
    // Don't set any pool metrics

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should only render set metrics plus always-present ones
    assert!(rendered.contains("truenas_up"));
}

#[test]
fn test_dataset_with_slash_in_name() {
    // Given: A metrics collector with ZFS-style dataset names (pool/dataset/child)
    let metrics = create_test_metrics();

    metrics
        .dataset_used_bytes
        .with_label_values(&["tank/data/photos/2024", "tank"])
        .set(1_000_000_000.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle slashes in dataset names
    assert!(rendered.contains("tank/data/photos/2024"));
}

#[test]
fn test_smart_test_with_special_status() {
    // Given: A metrics collector with unusual SMART test statuses
    let metrics = create_test_metrics();

    // Test with status codes beyond 0 and 1
    metrics
        .smart_test_status
        .with_label_values(&["sda", "Short"])
        .set(255); // Unusual status code

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle any integer value
    assert!(rendered.contains("255"));
}

#[test]
fn test_boundary_value_port_numbers() {
    // Given: Boundary port values
    let min_port = 1u16;
    let max_port = 65535u16;

    // When: Using these values
    // Then: They should be valid u16 values
    assert_eq!(min_port, 1);
    assert_eq!(max_port, 65535);
    // This is more of a documentation test - Rust type system ensures validity
}

#[test]
fn test_metrics_stability_after_multiple_renders() {
    // Given: A metrics collector with data
    let metrics = create_test_metrics();

    metrics
        .pool_health
        .with_label_values(&["test-pool", "ONLINE"])
        .set(1.0);

    // When: Rendering multiple times
    let render1 = metrics.render().expect("First render failed");
    let render2 = metrics.render().expect("Second render failed");
    let render3 = metrics.render().expect("Third render failed");

    // Then: All renders should be identical
    assert_eq!(render1, render2);
    assert_eq!(render2, render3);
}

#[test]
fn test_whitespace_in_labels() {
    // Given: A metrics collector with labels containing various whitespace
    let metrics = create_test_metrics();

    metrics
        .pool_health
        .with_label_values(&["pool\twith\ttabs", "ONLINE"])
        .set(1.0);

    metrics
        .pool_health
        .with_label_values(&["pool\nwith\nnewlines", "ONLINE"])
        .set(1.0);

    // When: Rendering metrics
    let rendered = metrics.render().expect("Failed to render");

    // Then: Should handle whitespace (Prometheus client lib handles escaping)
    assert!(rendered.contains("truenas_pool_health"));
}
