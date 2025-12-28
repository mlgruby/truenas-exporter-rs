use truenas_exporter::metrics::MetricsCollector;

#[test]
fn test_metrics_registration() {
    // Verify that all metrics can be created and registered without panicking
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");

    // Test that we can render metrics (even if empty)
    let rendered = metrics.render();
    assert!(rendered.is_ok(), "Failed to render metrics");

    // Verify the rendered output contains expected metric names
    // Note: GaugeVec metrics only appear once they have values set
    // Scalar metrics like truenas_up always appear
    let output = rendered.unwrap();
    assert!(output.contains("truenas_up"), "Missing truenas_up metric");
    assert!(
        output.contains("truenas_system_uptime_seconds"),
        "Missing system uptime metric"
    );
    assert!(
        output.contains("truenas_system_memory_total_bytes"),
        "Missing total memory metric"
    );
}

#[test]
fn test_metrics_update() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");

    // Test updating a simple gauge
    metrics.up.set(1.0);
    metrics.system_uptime_seconds.set(12345.0);

    // Test updating a labeled metric
    metrics
        .pool_capacity_bytes
        .with_label_values(&["test_pool"])
        .set(1000000.0);

    let rendered = metrics.render().unwrap();
    assert!(
        rendered.contains("truenas_up 1"),
        "up metric not set correctly"
    );
    assert!(rendered.contains("test_pool"), "pool label not found");
}

#[test]
fn test_metrics_reset() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");

    // Set some values
    metrics
        .pool_capacity_bytes
        .with_label_values(&["pool1"])
        .set(1000.0);

    // Reset should clear dynamic metrics
    metrics.reset();

    // After reset, rendering should still work
    let rendered = metrics.render();
    assert!(rendered.is_ok(), "Failed to render after reset");
}
