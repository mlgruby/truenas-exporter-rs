//! Simplified collector tests focusing on critical behavior

use truenas_exporter::collectors::{collect_with_handler, CollectionStatus};
use truenas_exporter::error::ExporterError;
use truenas_exporter::metrics::MetricsCollector;

fn create_test_metrics() -> MetricsCollector {
    MetricsCollector::new().expect("Failed to create test metrics")
}

#[tokio::test]
async fn test_collect_with_handler_success() {
    // Given: A successful API query returning data
    // When: The handler processes the query
    let result = collect_with_handler(
        "test",
        async { Ok::<Vec<String>, ExporterError>(vec!["data".to_string()]) },
        |data| {
            assert_eq!(data.len(), 1);
            assert_eq!(data[0], "data");
        },
    )
    .await;

    // Then: Collection should succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), CollectionStatus::Success);
}

#[tokio::test]
async fn test_collect_with_handler_error() {
    // Given: An API query that fails with an error
    // When: The handler processes the failed query
    let result = collect_with_handler(
        "test",
        async {
            Err::<Vec<String>, ExporterError>(ExporterError::Config("Test error".to_string()))
        },
        |_data| {
            panic!("Should not process data on error");
        },
    )
    .await;

    // Then: Collection should fail gracefully without panic
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), CollectionStatus::Failed);
}

#[tokio::test]
async fn test_collect_with_handler_anyhow_error() {
    // Given: An API query that fails with an anyhow error
    // When: The handler processes the error
    let result = collect_with_handler(
        "test",
        async { Err::<Vec<String>, anyhow::Error>(anyhow::anyhow!("Test error")) },
        |_data| {
            panic!("Should not process data on error");
        },
    )
    .await;

    // Then: Collection should fail gracefully
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), CollectionStatus::Failed);
}

#[test]
fn test_collection_status_enum() {
    // Given: CollectionStatus enum variants
    let success = CollectionStatus::Success;
    let failed = CollectionStatus::Failed;

    // When: Comparing enum values
    // Then: Values should be equal to themselves and different from each other
    assert_eq!(success, CollectionStatus::Success);
    assert_eq!(failed, CollectionStatus::Failed);
    assert_ne!(success, failed);
}

#[test]
fn test_metrics_helper_set_bool_metric() {
    // Given: A metrics collector
    let metrics = create_test_metrics();

    // When: Setting boolean metrics with helper method
    metrics.set_bool_metric(&metrics.dataset_encrypted, &["test", "pool"], true);
    metrics.set_bool_metric(&metrics.dataset_encrypted, &["test2", "pool"], false);

    // Then: Metrics should be rendered as 1 and 0
    let rendered = metrics.render().unwrap();
    assert!(rendered.contains("truenas_dataset_encrypted{dataset=\"test\",pool=\"pool\"} 1"));
    assert!(rendered.contains("truenas_dataset_encrypted{dataset=\"test2\",pool=\"pool\"} 0"));
}

#[test]
fn test_metrics_helper_set_gauge() {
    // Given: A metrics collector
    let metrics = create_test_metrics();

    // When: Setting a gauge metric with helper method
    metrics.set_gauge(&metrics.pool_capacity_bytes, &["test-pool"], 1000000.0);

    // Then: Metric should be rendered with the correct value
    let rendered = metrics.render().unwrap();
    assert!(rendered.contains("truenas_pool_capacity_bytes{pool=\"test-pool\"} 1000000"));
}

#[test]
fn test_metrics_helper_set_state_metric() {
    // Given: A metrics collector
    let metrics = create_test_metrics();

    // When: Setting state metrics comparing to a target state
    metrics.set_state_metric(
        &metrics.pool_health,
        &["pool1", "ONLINE"],
        "ONLINE",
        "ONLINE",
    );
    metrics.set_state_metric(
        &metrics.pool_health,
        &["pool2", "DEGRADED"],
        "DEGRADED",
        "ONLINE",
    );

    // Then: Matching state should be 1, non-matching should be 0
    let rendered = metrics.render().unwrap();
    assert!(rendered.contains("truenas_pool_health{pool=\"pool1\",status=\"ONLINE\"} 1"));
    assert!(rendered.contains("truenas_pool_health{pool=\"pool2\",status=\"DEGRADED\"} 0"));
}

#[test]
fn test_metric_reset_behavior() {
    // Given: A metrics collector with some metrics set
    let metrics = create_test_metrics();
    metrics
        .cloud_sync_status
        .with_label_values(&["task1", "RUNNING"])
        .set(1.0);
    metrics
        .cloud_sync_status
        .with_label_values(&["task2", "SUCCESS"])
        .set(1.0);

    let before = metrics.render().unwrap();
    assert!(before.contains("task1"));
    assert!(before.contains("task2"));

    // When: Resetting the metric and setting only one value
    metrics.cloud_sync_status.reset();
    metrics
        .cloud_sync_status
        .with_label_values(&["task1", "RUNNING"])
        .set(1.0);

    // Then: Only the newly set metric should appear
    let after = metrics.render().unwrap();
    assert!(after.contains("task1"));
    let task2_count = after.matches("task2").count();
    assert_eq!(task2_count, 0, "task2 should be cleared after reset");
}

#[tokio::test]
async fn test_empty_collection_succeeds() {
    // Given: An API query that returns empty data
    // When: The handler processes the empty collection
    let result = collect_with_handler(
        "test",
        async { Ok::<Vec<String>, ExporterError>(vec![]) },
        |data| {
            assert_eq!(data.len(), 0);
        },
    )
    .await;

    // Then: Collection should still succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), CollectionStatus::Success);
}

#[tokio::test]
async fn test_large_collection() {
    // Given: An API query that returns a large dataset (1000 items)
    let large_data: Vec<String> = (0..1000).map(|i| format!("item_{}", i)).collect();
    let expected_len = large_data.len();

    // When: The handler processes the large collection
    let result = collect_with_handler(
        "test",
        async move { Ok::<Vec<String>, ExporterError>(large_data) },
        move |data| {
            assert_eq!(data.len(), expected_len);
        },
    )
    .await;

    // Then: Collection should succeed without issues
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), CollectionStatus::Success);
}

#[test]
fn test_exporter_error_display() {
    // Given: Various ExporterError types
    let auth_err = ExporterError::Auth("test".to_string());
    let api_err = ExporterError::TrueNasApi("test".to_string());

    // When: Converting errors to strings
    // Then: Error messages should contain appropriate text
    assert!(format!("{}", auth_err).contains("Authentication failed"));
    assert!(format!("{}", api_err).contains("TrueNAS API error"));
}

#[test]
fn test_all_metric_types_can_be_set() {
    // Given: A metrics collector
    let metrics = create_test_metrics();

    // When: Setting various metric types
    metrics.up.set(1.0);
    metrics.system_uptime_seconds.set(12345.0);
    metrics
        .pool_health
        .with_label_values(&["tank", "ONLINE"])
        .set(1.0);
    metrics
        .alert_count
        .with_label_values(&["CRITICAL", "true"])
        .set(5.0);
    metrics
        .smart_test_status
        .with_label_values(&["sda", "Short"])
        .set(0);

    // Then: All metrics should be rendered correctly
    let rendered = metrics.render().unwrap();
    assert!(rendered.contains("truenas_up"));
    assert!(rendered.contains("truenas_system_uptime_seconds"));
    assert!(rendered.contains("truenas_pool_health"));
    assert!(rendered.contains("truenas_alert_count"));
    assert!(rendered.contains("truenas_smart_test_status"));
}
