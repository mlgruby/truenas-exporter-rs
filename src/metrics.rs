//! Prometheus Metrics Definitions
//!
//! This module defines all Prometheus metrics exposed by the TrueNAS exporter.
//!
//! # Metric Categories
//!
//! ## Storage Metrics
//! - Pool health, capacity, and scrub status
//! - Dataset usage and compression
//! - Disk information and temperature
//! - VDev error counts
//!
//! ## Data Protection
//! - Cloud sync task status and progress
//! - Snapshot task status
//!
//! ## Services
//! - SMB/NFS share status
//! - Application (Apps) status and updates
//! - System service status (SSH, NFS daemon, etc.)
//!
//! ## System Health
//! - Alert counts by severity
//! - CPU, memory, and network usage
//! - System uptime and load average
//!
//! # Metric Types
//!
//! - **Gauge**: Current value (e.g., pool size, temperature)
//! - **IntGauge**: Integer gauge (e.g., service status 0/1)
//! - **GaugeVec**: Gauge with labels (e.g., pool metrics labeled by pool name)
//!
//! All metrics use the `truenas_` namespace prefix.

use prometheus::{Encoder, Gauge, GaugeVec, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};
use std::sync::Arc;

/// Metrics collector for TrueNAS
#[derive(Clone)]
pub struct MetricsCollector {
    registry: Arc<Registry>,

    // Pool metrics
    pub pool_health: Arc<GaugeVec>,
    pub pool_capacity_bytes: Arc<GaugeVec>,
    pub pool_allocated_bytes: Arc<GaugeVec>,
    pub pool_free_bytes: Arc<GaugeVec>,
    pub pool_last_scrub_seconds: Arc<GaugeVec>,
    pub pool_scrub_errors: Arc<GaugeVec>,
    pub pool_vdev_error_count: Arc<GaugeVec>,

    // Dataset metrics
    pub dataset_used_bytes: Arc<GaugeVec>,
    pub dataset_available_bytes: Arc<GaugeVec>,
    pub dataset_compression_ratio: Arc<GaugeVec>,
    pub dataset_encrypted: Arc<GaugeVec>,

    // Share metrics
    pub share_smb_enabled: Arc<GaugeVec>,
    pub share_nfs_enabled: Arc<GaugeVec>,

    // Data Protection metrics
    pub cloud_sync_status: Arc<GaugeVec>,
    pub cloud_sync_progress: Arc<GaugeVec>,
    pub snapshot_task_status: Arc<GaugeVec>,
    pub alert_count: Arc<GaugeVec>,
    pub alert_info: Arc<GaugeVec>,

    // Disk metrics
    pub disk_temperature_celsius: Arc<GaugeVec>,
    pub disk_read_bytes_per_second: Arc<GaugeVec>,
    pub disk_write_bytes_per_second: Arc<GaugeVec>,
    pub disk_info: Arc<IntGaugeVec>,

    // SMART metrics
    pub smart_test_status: Arc<IntGaugeVec>,
    pub smart_test_lifetime_hours: Arc<GaugeVec>,
    pub smart_test_timestamp_seconds: Arc<GaugeVec>,
    pub disk_power_on_hours: Arc<GaugeVec>,

    // Application metrics
    pub app_status: Arc<IntGaugeVec>,
    pub app_cpu_percent: Arc<GaugeVec>,
    pub app_memory_bytes: Arc<GaugeVec>,
    pub app_update_available: Arc<IntGaugeVec>,

    // System metrics
    pub system_info: Arc<IntGauge>,
    pub system_uptime_seconds: Arc<Gauge>,
    pub system_cpu_usage_percent: Arc<GaugeVec>,
    pub system_cpu_temperature_celsius: Arc<GaugeVec>,
    pub system_memory_bytes: Arc<GaugeVec>,
    pub system_memory_used_bytes: Arc<Gauge>,
    pub system_memory_total_bytes: Arc<Gauge>,
    pub system_load_average: Arc<GaugeVec>,
    pub up: Arc<Gauge>,

    // Network
    pub network_interface_info: Arc<IntGaugeVec>,
    pub network_receive_bytes_per_second: Arc<GaugeVec>,
    pub network_transmit_bytes_per_second: Arc<GaugeVec>,

    // Service status
    pub service_status: Arc<IntGaugeVec>,
}

impl MetricsCollector {
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();

        // Pool metrics
        let pool_health = GaugeVec::new(
            Opts::new("pool_health", "Pool health status (1=healthy, 0=unhealthy)")
                .namespace("truenas"),
            &["pool", "status"],
        )?;

        let pool_capacity_bytes = GaugeVec::new(
            Opts::new(
                "pool_capacity_bytes",
                "Total storage capacity of the ZFS pool",
            )
            .namespace("truenas"),
            &["pool"],
        )?;

        let pool_allocated_bytes = GaugeVec::new(
            Opts::new(
                "pool_allocated_bytes",
                "Used storage capacity of the ZFS pool",
            )
            .namespace("truenas"),
            &["pool"],
        )?;

        let pool_free_bytes = GaugeVec::new(
            Opts::new("pool_free_bytes", "Free storage capacity of the ZFS pool")
                .namespace("truenas"),
            &["pool"],
        )?;

        let pool_last_scrub_seconds = GaugeVec::new(
            Opts::new("pool_last_scrub_seconds", "Timestamp of the last ZFS scrub")
                .namespace("truenas"),
            &["pool"],
        )?;

        let pool_scrub_errors = GaugeVec::new(
            Opts::new(
                "pool_scrub_errors",
                "Number of errors found during last ZFS scrub",
            )
            .namespace("truenas"),
            &["pool"],
        )?;

        let pool_vdev_error_count = GaugeVec::new(
            Opts::new(
                "pool_vdev_error_count",
                "ZFS vdev error counts (read/write/checksum)",
            )
            .namespace("truenas"),
            &["pool", "vdev", "type"],
        )?;

        // Dataset metrics
        let dataset_used_bytes = GaugeVec::new(
            Opts::new("dataset_used_bytes", "Used bytes of the dataset").namespace("truenas"),
            &["dataset", "pool"],
        )?;

        let dataset_available_bytes = GaugeVec::new(
            Opts::new("dataset_available_bytes", "Available bytes for the dataset")
                .namespace("truenas"),
            &["dataset", "pool"],
        )?;

        let dataset_compression_ratio = GaugeVec::new(
            Opts::new(
                "dataset_compression_ratio",
                "Compression ratio of the dataset",
            )
            .namespace("truenas"),
            &["dataset", "pool"],
        )?;

        let dataset_encrypted = GaugeVec::new(
            Opts::new(
                "dataset_encrypted",
                "Encryption status of the dataset (1=encrypted, 0=unencrypted)",
            )
            .namespace("truenas"),
            &["dataset", "pool"],
        )?;

        // Share metrics
        let share_smb_enabled = GaugeVec::new(
            Opts::new(
                "share_smb_enabled",
                "SMB Share Status (1=Enabled, 0=Disabled)",
            )
            .namespace("truenas"),
            &["name", "path"],
        )?;
        let share_nfs_enabled = GaugeVec::new(
            Opts::new(
                "share_nfs_enabled",
                "NFS Share Status (1=Enabled, 0=Disabled)",
            )
            .namespace("truenas"),
            &["path"],
        )?;

        // Data Protection metrics
        let cloud_sync_status = GaugeVec::new(
            Opts::new("cloud_sync_status", "Cloud Sync Task Status (1=Active)")
                .namespace("truenas"),
            &["description", "state"],
        )?;
        let cloud_sync_progress = GaugeVec::new(
            Opts::new(
                "cloud_sync_progress_percent",
                "Cloud Sync Progress Percentage",
            )
            .namespace("truenas"),
            &["description"],
        )?;
        let snapshot_task_status = GaugeVec::new(
            Opts::new("snapshot_task_status", "Snapshot Task Status (1=Active)")
                .namespace("truenas"),
            &["dataset", "state"],
        )?;
        let alert_count = GaugeVec::new(
            Opts::new(
                "alert_count",
                "Number of system alerts by severity and status",
            )
            .namespace("truenas"),
            &["level", "active"],
        )?;

        let alert_info = GaugeVec::new(
            Opts::new(
                "alert_info",
                "Detailed alert information (value is always 1)",
            )
            .namespace("truenas"),
            &["level", "message", "uuid", "active"],
        )?;

        // Disk metrics
        let disk_temperature_celsius = GaugeVec::new(
            Opts::new(
                "disk_temperature_celsius",
                "Current temperature of the disk in Celsius",
            )
            .namespace("truenas"),
            &["device"],
        )?;

        let disk_read_bytes_per_second = GaugeVec::new(
            Opts::new(
                "disk_read_bytes_per_second",
                "Disk read rate in bytes per second",
            )
            .namespace("truenas"),
            &["device"],
        )?;

        let disk_write_bytes_per_second = GaugeVec::new(
            Opts::new(
                "disk_write_bytes_per_second",
                "Disk write rate in bytes per second",
            )
            .namespace("truenas"),
            &["device"],
        )?;

        let disk_info = IntGaugeVec::new(
            Opts::new("disk_info", "Disk information (value is always 1)").namespace("truenas"),
            &["disk", "serial", "model", "size"],
        )?;

        // SMART metrics
        let smart_test_status = IntGaugeVec::new(
            Opts::new(
                "smart_test_status",
                "SMART test status (0=success, 1=failed)",
            )
            .namespace("truenas"),
            &["disk", "test_type"],
        )?;

        let smart_test_lifetime_hours = GaugeVec::new(
            Opts::new(
                "smart_test_lifetime_hours",
                "Disk lifetime hours when the last SMART test was run",
            )
            .namespace("truenas"),
            &["disk", "test_type"],
        )?;

        let smart_test_timestamp_seconds = GaugeVec::new(
            Opts::new(
                "smart_test_timestamp_seconds",
                "Unix timestamp when the last SMART test was run",
            )
            .namespace("truenas"),
            &["disk", "test_type"],
        )?;

        let disk_power_on_hours = GaugeVec::new(
            Opts::new("disk_power_on_hours", "Total power-on hours for the disk")
                .namespace("truenas"),
            &["disk"],
        )?;

        // Application metrics
        let app_status = IntGaugeVec::new(
            Opts::new("app_status", "Application status (0=stopped, 1=running)")
                .namespace("truenas"),
            &["app"],
        )?;

        let app_cpu_percent = GaugeVec::new(
            Opts::new("app_cpu_percent", "Application CPU usage percentage").namespace("truenas"),
            &["app"],
        )?;

        let app_memory_bytes = GaugeVec::new(
            Opts::new("app_memory_bytes", "Application memory usage in bytes").namespace("truenas"),
            &["app"],
        )?;

        let app_update_available = IntGaugeVec::new(
            Opts::new(
                "app_update_available",
                "Application update available (0=no, 1=yes)",
            )
            .namespace("truenas"),
            &["app"],
        )?;

        // System metrics
        let system_info = IntGauge::new(
            "truenas_system_info",
            "TrueNAS system information (value is always 1)",
        )?;

        let system_uptime_seconds =
            Gauge::new("truenas_system_uptime_seconds", "System uptime in seconds")?;

        let system_cpu_usage_percent = GaugeVec::new(
            Opts::new(
                "system_cpu_usage_percent",
                "System CPU usage percentage by mode",
            )
            .namespace("truenas"),
            &["mode"],
        )?;

        let system_cpu_temperature_celsius = GaugeVec::new(
            Opts::new(
                "system_cpu_temperature_celsius",
                "System CPU temperature in Celsius",
            )
            .namespace("truenas"),
            &["cpu"],
        )?;

        let system_memory_bytes = GaugeVec::new(
            Opts::new(
                "system_memory_bytes",
                "System memory usage in bytes by state",
            )
            .namespace("truenas"),
            &["state"],
        )?;

        let system_memory_used_bytes = Gauge::new(
            "truenas_system_memory_used_bytes",
            "System memory used in bytes (Total - Available)",
        )?;

        let system_memory_total_bytes = Gauge::new(
            "truenas_system_memory_total_bytes",
            "Total system memory in bytes",
        )?;

        let system_load_average = GaugeVec::new(
            Opts::new("system_load_average", "System load average").namespace("truenas"),
            &["period"],
        )?;

        let network_interface_info = IntGaugeVec::new(
            Opts::new(
                "network_interface_info",
                "Network interface information (value is always 1)",
            )
            .namespace("truenas"),
            &["interface", "link_state"],
        )?;

        let network_receive_bytes_per_second = GaugeVec::new(
            Opts::new(
                "network_receive_bytes_per_second",
                "Network receive rate in bytes per second",
            )
            .namespace("truenas"),
            &["interface"],
        )?;

        let network_transmit_bytes_per_second = GaugeVec::new(
            Opts::new(
                "network_transmit_bytes_per_second",
                "Network transmit rate in bytes per second",
            )
            .namespace("truenas"),
            &["interface"],
        )?;

        let service_status = IntGaugeVec::new(
            Opts::new("service_status", "Service status (0=stopped, 1=running)")
                .namespace("truenas"),
            &["service"],
        )?;

        let up = Gauge::new(
            "truenas_up",
            "Whether the TrueNAS API is reachable (1=up, 0=down)",
        )?;

        // Register all metrics
        registry.register(Box::new(pool_health.clone()))?;
        registry.register(Box::new(pool_capacity_bytes.clone()))?;
        registry.register(Box::new(pool_allocated_bytes.clone()))?;
        registry.register(Box::new(pool_free_bytes.clone()))?;
        registry.register(Box::new(pool_last_scrub_seconds.clone()))?;
        registry.register(Box::new(pool_scrub_errors.clone()))?;
        registry.register(Box::new(pool_vdev_error_count.clone()))?;
        registry.register(Box::new(dataset_used_bytes.clone()))?;
        registry.register(Box::new(dataset_available_bytes.clone()))?;
        registry.register(Box::new(dataset_compression_ratio.clone()))?;
        registry.register(Box::new(dataset_encrypted.clone()))?;
        registry.register(Box::new(share_smb_enabled.clone()))?;
        registry.register(Box::new(share_nfs_enabled.clone()))?;
        registry.register(Box::new(cloud_sync_status.clone()))?;
        registry.register(Box::new(cloud_sync_progress.clone()))?;
        registry.register(Box::new(snapshot_task_status.clone()))?;
        registry.register(Box::new(alert_count.clone()))?;
        registry.register(Box::new(alert_info.clone()))?;
        registry.register(Box::new(disk_temperature_celsius.clone()))?;
        registry.register(Box::new(disk_read_bytes_per_second.clone()))?;
        registry.register(Box::new(disk_write_bytes_per_second.clone()))?;
        registry.register(Box::new(disk_info.clone()))?;
        registry.register(Box::new(smart_test_status.clone()))?;
        registry.register(Box::new(smart_test_lifetime_hours.clone()))?;
        registry.register(Box::new(smart_test_timestamp_seconds.clone()))?;
        registry.register(Box::new(disk_power_on_hours.clone()))?;
        registry.register(Box::new(app_status.clone()))?;
        registry.register(Box::new(app_cpu_percent.clone()))?;
        registry.register(Box::new(app_memory_bytes.clone()))?;
        registry.register(Box::new(app_update_available.clone()))?;
        registry.register(Box::new(system_info.clone()))?;
        registry.register(Box::new(system_uptime_seconds.clone()))?;
        registry.register(Box::new(system_cpu_usage_percent.clone()))?;
        registry.register(Box::new(system_cpu_temperature_celsius.clone()))?;
        registry.register(Box::new(system_memory_bytes.clone()))?;
        registry.register(Box::new(system_memory_used_bytes.clone()))?;
        registry.register(Box::new(system_memory_total_bytes.clone()))?;
        registry.register(Box::new(system_load_average.clone()))?;
        registry.register(Box::new(network_interface_info.clone()))?;
        registry.register(Box::new(network_receive_bytes_per_second.clone()))?;
        registry.register(Box::new(network_transmit_bytes_per_second.clone()))?;
        registry.register(Box::new(service_status.clone()))?;
        registry.register(Box::new(up.clone()))?;

        Ok(Self {
            registry: Arc::new(registry),
            pool_health: Arc::new(pool_health),
            pool_capacity_bytes: Arc::new(pool_capacity_bytes),
            pool_allocated_bytes: Arc::new(pool_allocated_bytes),
            pool_free_bytes: Arc::new(pool_free_bytes),
            pool_last_scrub_seconds: Arc::new(pool_last_scrub_seconds),
            pool_scrub_errors: Arc::new(pool_scrub_errors),
            pool_vdev_error_count: Arc::new(pool_vdev_error_count),
            dataset_used_bytes: Arc::new(dataset_used_bytes),
            dataset_available_bytes: Arc::new(dataset_available_bytes),
            dataset_compression_ratio: Arc::new(dataset_compression_ratio),
            dataset_encrypted: Arc::new(dataset_encrypted),
            share_smb_enabled: Arc::new(share_smb_enabled),
            share_nfs_enabled: Arc::new(share_nfs_enabled),
            cloud_sync_status: Arc::new(cloud_sync_status),
            cloud_sync_progress: Arc::new(cloud_sync_progress),
            snapshot_task_status: Arc::new(snapshot_task_status),
            alert_count: Arc::new(alert_count),
            alert_info: Arc::new(alert_info),
            disk_temperature_celsius: Arc::new(disk_temperature_celsius),
            disk_read_bytes_per_second: Arc::new(disk_read_bytes_per_second),
            disk_write_bytes_per_second: Arc::new(disk_write_bytes_per_second),
            disk_info: Arc::new(disk_info),
            smart_test_status: Arc::new(smart_test_status),
            smart_test_lifetime_hours: Arc::new(smart_test_lifetime_hours),
            smart_test_timestamp_seconds: Arc::new(smart_test_timestamp_seconds),
            disk_power_on_hours: Arc::new(disk_power_on_hours),
            app_status: Arc::new(app_status),
            app_cpu_percent: Arc::new(app_cpu_percent),
            app_memory_bytes: Arc::new(app_memory_bytes),
            app_update_available: Arc::new(app_update_available),
            system_info: Arc::new(system_info),
            system_uptime_seconds: Arc::new(system_uptime_seconds),
            system_cpu_usage_percent: Arc::new(system_cpu_usage_percent),
            system_cpu_temperature_celsius: Arc::new(system_cpu_temperature_celsius),
            system_memory_bytes: Arc::new(system_memory_bytes),
            system_memory_used_bytes: Arc::new(system_memory_used_bytes),
            system_memory_total_bytes: Arc::new(system_memory_total_bytes),
            system_load_average: Arc::new(system_load_average),
            network_interface_info: Arc::new(network_interface_info),
            network_receive_bytes_per_second: Arc::new(network_receive_bytes_per_second),
            network_transmit_bytes_per_second: Arc::new(network_transmit_bytes_per_second),
            service_status: Arc::new(service_status),
            up: Arc::new(up),
        })
    }

    /// Render metrics in Prometheus text format
    pub fn render(&self) -> anyhow::Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Reset all metrics (useful before a fresh scrape)
    #[allow(dead_code)] // MVP: Will be used in future iterations
    pub fn reset(&self) {
        self.pool_health.reset();
        self.pool_capacity_bytes.reset();
        self.pool_allocated_bytes.reset();
        self.pool_free_bytes.reset();
        self.pool_last_scrub_seconds.reset();
        self.pool_scrub_errors.reset();
        self.pool_vdev_error_count.reset();
        self.dataset_used_bytes.reset();
        self.dataset_available_bytes.reset();
        self.dataset_compression_ratio.reset();
        self.dataset_encrypted.reset();
        self.share_smb_enabled.reset();
        self.share_nfs_enabled.reset();
        self.cloud_sync_status.reset();
        self.cloud_sync_progress.reset();
        self.snapshot_task_status.reset();
        self.alert_count.reset();
        self.alert_info.reset();
        self.disk_temperature_celsius.reset();
        self.disk_read_bytes_per_second.reset();
        self.disk_write_bytes_per_second.reset();
        self.disk_info.reset();
        self.smart_test_status.reset();
        self.app_status.reset();
        self.app_cpu_percent.reset();
        self.app_memory_bytes.reset();
        self.app_update_available.reset();
        // IntGauge and Gauge do not have a reset method.
        // For IntGauge, we can't reset it to a default value like 0 or 1 without knowing its purpose.
        // For Gauge, we can set it to 0 if that's the desired "reset" state.
        // self.system_info.reset(); // IntGauge doesn't have reset()
        self.system_uptime_seconds.set(0.0); // Gauge can't reset, but we can set to 0? Or just leave it? Gauge doesn't have reset?
                                             // Actually Gauge doesn't have reset() method in rust-prometheus?
                                             // Wait, IntGauge/Gauge don't have reset(). The GaugeVec does.
                                             // We should probably just not reset scalar gauges or set them to 0.
        self.system_cpu_usage_percent.reset();
        self.system_cpu_usage_percent.reset();
        self.system_cpu_temperature_celsius.reset();
        self.system_memory_bytes.reset();
        self.system_memory_used_bytes.set(0.0);
        self.system_memory_total_bytes.set(0.0);
        self.system_load_average.reset();
        self.network_interface_info.reset();
        self.network_receive_bytes_per_second.reset();
        self.network_transmit_bytes_per_second.reset();
        self.service_status.reset();
        // self.up.reset(); // Gauge doesn't have reset()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics collector")
    }
}
