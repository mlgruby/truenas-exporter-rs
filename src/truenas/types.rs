//! TrueNAS API Type Definitions
//!
//! This module contains Rust struct definitions for all TrueNAS Scale API responses.
//! These types are used for deserializing JSON responses from the WebSocket API.
//!
//! # Design Notes
//!
//! - **Completeness**: Structs include all fields from the API, even if not currently used.
//!   This allows for future expansion without breaking changes.
//! - **Optional Fields**: Many fields are `Option<T>` because the API may omit them or return null.
//! - **Serde Defaults**: `#[serde(default)]` is used extensively to handle missing fields gracefully.
//! - **Dead Code**: The `#![allow(dead_code)]` attribute suppresses warnings for unused fields,
//!   as we intentionally keep complete type definitions for documentation and future use.
//!
//! # API Endpoints Covered
//!
//! - `pool.query` → [`Pool`], [`PoolScan`], [`Topology`], [`VDev`]
//! - `pool.dataset.query` → [`Dataset`]
//! - `disk.query` → [`DiskInfo`]
//! - `smart.test.results` → [`SmartTestResult`]
//! - `app.query` → [`AppInfo`]
//! - `sharing.smb.query` → [`SmbShare`]
//! - `sharing.nfs.query` → [`NfsShare`]
//! - `cloudsync.query` → [`CloudSyncTask`]
//! - `pool.snapshottask.query` → [`SnapshotTask`]
//! - `alert.list` → [`TruenasAlert`]
//! - `system.info` → [`SystemInfo`]
//!
//! # JSON-RPC Protocol
//!
//! - [`JsonRpcRequest`] - Outgoing method calls
//! - [`JsonRpcResponse`] - Incoming responses
//! - [`DdpConnect`] - Initial handshake message

#![allow(dead_code)] // Allow unused fields in API structs for completeness
use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub id: String,
    pub msg: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse {
    #[allow(dead_code)] // Part of JSON-RPC spec
    pub id: String,
    #[allow(dead_code)] // Part of JSON-RPC spec
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    #[serde(default)]
    pub error: Option<i32>,
    #[serde(default)]
    pub errname: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// DDP Connect message
#[derive(Debug, Serialize)]
pub struct DdpConnect {
    pub msg: String,
    pub version: String,
    pub support: Vec<String>,
}

impl Default for DdpConnect {
    fn default() -> Self {
        Self {
            msg: "connect".to_string(),
            version: "1".to_string(),
            support: vec!["1".to_string()],
        }
    }
}

/// Pool information from pool.query

#[derive(Debug, Deserialize, Clone)]
pub struct PoolScan {
    pub function: Option<String>,
    pub state: Option<String>,
    pub start_time: Option<serde_json::Value>,
    pub end_time: Option<serde_json::Value>,
    pub bytes_to_process: Option<u64>,
    pub bytes_processed: Option<u64>,
    pub errors: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Topology {
    #[serde(default)]
    pub data: Vec<VDev>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VDev {
    pub name: String,
    pub disk: Option<String>,
    pub device: Option<String>,
    #[serde(default)]
    pub stats: Option<VDevStats>,
    #[serde(default)]
    pub children: Vec<VDev>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VDevStats {
    #[serde(default)]
    pub read_errors: u64,
    #[serde(default)]
    pub write_errors: u64,
    #[serde(default)]
    pub checksum_errors: u64,
}
#[derive(Debug, Deserialize)]
pub struct Pool {
    pub name: String,
    pub status: String,
    pub healthy: bool,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub allocated: u64,
    #[serde(default)]
    pub free: u64,
    #[serde(default)]
    pub scan: Option<PoolScan>,
    #[serde(default)]
    pub topology: Option<Topology>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Dataset {
    pub name: String,
    pub encrypted: bool,
    #[serde(default)]
    pub used: Option<DatasetValue<u64>>,
    #[serde(default)]
    pub available: Option<DatasetValue<u64>>,
    #[serde(default)]
    pub compressratio: Option<DatasetValue<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatasetValue<T> {
    pub parsed: T,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmbShare {
    pub name: String,
    pub path: String,
    pub enabled: bool,
    #[serde(default)]
    pub comment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NfsShare {
    pub path: String,
    pub enabled: bool,
    #[serde(default)]
    pub comment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CloudSyncTask {
    pub id: i64,
    pub description: String,
    pub enabled: bool,
    #[serde(default)]
    pub job: Option<CloudSyncJob>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CloudSyncJob {
    pub state: String, // "RUNNING", "SUCCESS", etc.
    #[serde(default)]
    pub progress: Option<CloudSyncProgress>,
    #[serde(default)]
    pub time_finished: Option<serde_json::Value>, // handled like date
}

#[derive(Debug, Deserialize, Clone)]
pub struct CloudSyncProgress {
    #[serde(default)]
    pub percent: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SnapshotTask {
    pub dataset: String,
    pub enabled: bool,
    #[serde(default)]
    pub state: Option<SnapshotTaskState>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SnapshotTaskState {
    pub state: String, // "FINISHED", "ERROR"
    #[serde(default)]
    pub datetime: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TruenasAlert {
    pub uuid: String,
    pub level: String, // "CRITICAL", "ERROR", "WARNING", "INFO"
    #[serde(default)]
    pub dismissed: bool,
    #[serde(default)]
    pub formatted: String,
}

/// System information from system.info
#[derive(Debug, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub hostname: String,
    #[serde(default)]
    pub uptime_seconds: f64,
    #[serde(default)]
    pub loadavg: Option<Vec<f64>>,
    #[serde(default)]
    pub physmem: Option<u64>,
    #[serde(default)]
    pub usage: Option<SystemUsage>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SystemUsage {
    #[serde(default)]
    pub cpu_percent: f64,
    #[serde(default)]
    pub memory_percent: f64,
}

/// Disk information from disk.query
#[derive(Debug, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    #[serde(default)]
    pub serial: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub hddstandby: String,
    #[serde(default)]
    pub advpowermgmt: String,
}

/// Disk temperature aggregation from disk.temperature_agg
#[derive(Debug, Deserialize)]
pub struct DiskTemperature {
    #[serde(flatten)]
    pub temperatures: std::collections::HashMap<String, Option<f64>>,
}

/// SMART test result from smart.test.results
#[derive(Debug, Deserialize)]
pub struct SmartTestResult {
    pub disk: String,
    #[serde(rename = "type", default)]
    pub test_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub num: i32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub remaining: f64,
    #[serde(default)]
    pub lifetime: i64,
    #[serde(default)]
    pub lba_of_first_error: Option<String>,
}

/// Application information from app.query
#[derive(Debug, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub state: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub human_version: String,
    #[serde(default)]
    pub update_available: bool,
    #[serde(default)]
    pub portal: Option<String>,
}

/// Application statistics from app.stats
#[derive(Debug, Deserialize)]
pub struct AppStats {
    pub name: String,
    #[serde(default)]
    pub cpu_percent: f64,
    #[serde(default)]
    pub memory_bytes: u64,
    #[serde(default)]
    pub network_rx_bytes: u64,
    #[serde(default)]
    pub network_tx_bytes: u64,
}

/// Network interface information from interface.query
#[derive(Debug, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    #[serde(default)]
    pub state: NetworkInterfaceState,
}

#[derive(Debug, Deserialize, Default)]
pub struct NetworkInterfaceState {
    #[serde(default)]
    pub link_state: String,
    #[serde(default)]
    pub active_media_type: String,
    #[serde(default)]
    pub active_media_subtype: String,
}

/// Service information from service.query
#[derive(Debug, Deserialize)]
pub struct ServiceInfo {
    pub service: String,
    pub state: String,
    pub enable: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReportingGraph {
    pub name: String,
    pub title: String,
    pub vertical_label: String,
    #[serde(default)]
    pub identifiers: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ReportingQuery {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReportingData {
    pub name: String,
    #[serde(default)]
    pub identifier: Option<String>,
    pub data: Vec<Vec<Option<f64>>>, // Can be null
    pub legend: Vec<String>,
    #[serde(default)]
    pub start: u64,
    #[serde(default)]
    pub end: u64,
}
