//! Property-based tests using proptest
//!
//! Tests that verify properties hold for arbitrary inputs.

use proptest::prelude::*;
use truenas_exporter::metrics::MetricsCollector;

/// Helper to create a test metrics instance
fn create_test_metrics() -> MetricsCollector {
    MetricsCollector::new().expect("Failed to create metrics")
}

proptest! {
    #[test]
    fn test_any_pool_name_renders_without_panic(pool_name in "\\PC*") {
        // Given: A metrics collector and arbitrary pool name
        let metrics = create_test_metrics();
        let status = "ONLINE";

        // When: Setting pool health with any string
        metrics
            .pool_health
            .with_label_values(&[pool_name.as_str(), status])
            .set(1.0);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_any_pool_status_renders_without_panic(status in "\\PC*") {
        // Given: A metrics collector and arbitrary status string
        let metrics = create_test_metrics();

        // When: Setting pool health with any status
        metrics
            .pool_health
            .with_label_values(&["test-pool", &status])
            .set(1.0);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_any_positive_capacity_value(capacity in 0.0f64..1e18) {
        // Given: A metrics collector and arbitrary positive capacity
        let metrics = create_test_metrics();

        // When: Setting pool capacity with any positive value
        metrics
            .pool_capacity_bytes
            .with_label_values(&["test-pool"])
            .set(capacity);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_any_gauge_value(value in -1e18..1e18) {
        // Given: A metrics collector and arbitrary gauge value
        let metrics = create_test_metrics();

        // When: Setting any gauge metric with arbitrary value
        metrics
            .pool_capacity_bytes
            .with_label_values(&["test-pool"])
            .set(value);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_pool_names_no_collision(
        pool1 in "[a-zA-Z0-9_-]{1,20}",
        pool2 in "[a-zA-Z0-9_-]{1,20}"
    ) {
        // Given: A metrics collector and two different pool names
        let metrics = create_test_metrics();
        let status = "ONLINE";

        // When: Setting metrics for both pools
        metrics
            .pool_health
            .with_label_values(&[pool1.as_str(), status])
            .set(1.0);
        metrics
            .pool_health
            .with_label_values(&[pool2.as_str(), status])
            .set(1.0);

        // Then: Rendering should contain both pools
        let rendered = metrics.render().unwrap();
        prop_assert!(rendered.contains(&pool1));
        prop_assert!(rendered.contains(&pool2));
    }

    #[test]
    fn test_alert_level_variations(level in "[A-Z]{3,10}") {
        // Given: A metrics collector and arbitrary alert level
        let metrics = create_test_metrics();

        // When: Setting alert count with any level string
        let active = "true";
        metrics
            .alert_count
            .with_label_values(&[level.as_str(), active])
            .set(5.0);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_dataset_names_with_slashes(
        parts in prop::collection::vec("[a-z]{1,10}", 1..5)
    ) {
        // Given: A metrics collector and dataset name with multiple path components
        let metrics = create_test_metrics();
        let dataset_name = parts.join("/");
        let pool_name = "pool";

        // When: Setting dataset metrics
        metrics
            .dataset_used_bytes
            .with_label_values(&[dataset_name.as_str(), pool_name])
            .set(1000.0);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_disk_names_various_formats(disk_name in "[a-z]{2,4}[0-9]{0,2}") {
        // Given: A metrics collector and various disk name formats
        let metrics = create_test_metrics();

        // When: Setting disk metrics
        metrics
            .disk_power_on_hours
            .with_label_values(&[&disk_name])
            .set(12345.0);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_service_names(service in "[a-z-]{3,15}") {
        // Given: A metrics collector and arbitrary service name
        let metrics = create_test_metrics();

        // When: Setting service status
        metrics
            .service_status
            .with_label_values(&[&service])
            .set(1);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_compression_ratios(ratio in 0.1f64..20.0f64) {
        // Given: A metrics collector and compression ratio
        let metrics = create_test_metrics();

        // When: Setting compression ratio (realistic range 0.1x to 20x)
        metrics
            .dataset_compression_ratio
            .with_label_values(&["dataset", "pool"])
            .set(ratio);

        // Then: Rendering should contain the ratio
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_integer_metric_values(value in -1000i64..1000i64) {
        // Given: A metrics collector and arbitrary integer value
        let metrics = create_test_metrics();

        // When: Setting integer-based metric (SMART status)
        metrics
            .smart_test_status
            .with_label_values(&["sda", "Short"])
            .set(value);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_network_interface_names(iface in "[a-z]{2,6}[0-9]{0,2}") {
        // Given: A metrics collector and network interface name
        let metrics = create_test_metrics();

        // When: Setting network metrics
        metrics
            .network_receive_bytes_per_second
            .with_label_values(&[&iface])
            .set(1000000.0);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_temperature_values(temp in -50.0f64..150.0f64) {
        // Given: A metrics collector and temperature value
        let metrics = create_test_metrics();

        // When: Setting CPU temperature (realistic range)
        metrics
            .system_cpu_temperature_celsius
            .with_label_values(&["0"])
            .set(temp);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_percentage_values(percent in 0.0f64..100.0f64) {
        // Given: A metrics collector and percentage value
        let metrics = create_test_metrics();

        // When: Setting CPU usage percentage
        metrics
            .system_cpu_usage_percent
            .with_label_values(&["user"])
            .set(percent);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_vdev_error_counts(errors in 0u64..10000u64) {
        // Given: A metrics collector and error count
        let metrics = create_test_metrics();

        // When: Setting VDev error count
        metrics
            .pool_vdev_error_count
            .with_label_values(&["tank", "sda", "read"])
            .set(errors as f64);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_render_idempotency(value in 0.0f64..1e12) {
        // Given: A metrics collector with a value set
        let metrics = create_test_metrics();
        metrics
            .pool_capacity_bytes
            .with_label_values(&["test-pool"])
            .set(value);

        // When: Rendering multiple times
        let render1 = metrics.render().unwrap();
        let render2 = metrics.render().unwrap();

        // Then: Results should be identical (idempotent)
        prop_assert_eq!(render1, render2);
    }

    #[test]
    fn test_label_value_ordering_doesnt_matter(
        pool in "[a-z]{3,10}",
        status in "[A-Z]{3,10}"
    ) {
        // Given: Two metrics collectors with same data
        let metrics1 = create_test_metrics();
        let metrics2 = create_test_metrics();

        // When: Setting same pool with same status
        metrics1
            .pool_health
            .with_label_values(&[pool.as_str(), status.as_str()])
            .set(1.0);
        metrics2
            .pool_health
            .with_label_values(&[pool.as_str(), status.as_str()])
            .set(1.0);

        // Then: Renders should be identical
        let render1 = metrics1.render().unwrap();
        let render2 = metrics2.render().unwrap();
        prop_assert_eq!(render1, render2);
    }

    #[test]
    fn test_uptime_values(uptime in 0u64..31536000u64) {
        // Given: A metrics collector and uptime value (0 to 1 year in seconds)
        let metrics = create_test_metrics();

        // When: Setting system uptime
        metrics.system_uptime_seconds.set(uptime as f64);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_memory_sizes(memory in 0u64..1099511627776u64) {
        // Given: A metrics collector and memory size (0 to 1TB)
        let metrics = create_test_metrics();

        // When: Setting memory metrics
        metrics
            .system_memory_bytes
            .with_label_values(&["available"])
            .set(memory as f64);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_app_names_with_various_chars(
        name in "[a-zA-Z0-9][a-zA-Z0-9_-]{0,19}"
    ) {
        // Given: A metrics collector and app name
        let metrics = create_test_metrics();

        // When: Setting app status
        metrics
            .app_status
            .with_label_values(&[&name])
            .set(1);

        // Then: Rendering should not panic
        let result = metrics.render();
        prop_assert!(result.is_ok());
    }
}

// Additional property test: metrics always contain required metadata
proptest! {
    #[test]
    fn test_rendered_metrics_always_have_help_and_type(
        pool_name in "[a-z]{3,10}"
    ) {
        // Given: A metrics collector with any pool
        let metrics = create_test_metrics();
        let status = "ONLINE";
        metrics
            .pool_health
            .with_label_values(&[pool_name.as_str(), status])
            .set(1.0);

        // When: Rendering metrics
        let rendered = metrics.render().unwrap();

        // Then: Output should always contain Prometheus metadata
        prop_assert!(rendered.contains("# HELP"));
        prop_assert!(rendered.contains("# TYPE"));
    }
}
