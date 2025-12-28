//! TrueNAS Scale Prometheus Exporter
//!
//! A Prometheus metrics exporter for TrueNAS Scale 25.x (Electric Eel) and newer.
//!
//! # Overview
//!
//! This exporter connects to the TrueNAS Scale WebSocket API to collect comprehensive
//! metrics about storage, services, data protection, and system health. Metrics are
//! exposed in Prometheus format for scraping by Prometheus/Grafana.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐      WebSocket       ┌──────────────┐
//! │  TrueNAS    │ ◄─────────────────►  │   Exporter   │
//! │   Scale     │   JSON-RPC 2.0       │              │
//! └─────────────┘                      │  ┌────────┐  │      HTTP      ┌────────────┐
//!                                      │  │ Client │  │ ◄────────────► │ Prometheus │
//!                                      │  └────────┘  │   /metrics     └────────────┘
//!                                      │  ┌────────┐  │
//!                                      │  │Metrics │  │
//!                                      │  └────────┘  │
//!                                      └──────────────┘
//! ```
//!
//! # Modules
//!
//! - [`truenas`] - WebSocket client and API type definitions
//! - [`metrics`] - Prometheus metric definitions
//! - [`server`] - HTTP server and collection loop
//! - [`config`] - Configuration management
//! - [`error`] - Error types
//!
//! # Quick Start
//!
//! ```no_run
//! use truenas_exporter::{config::Config, server};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::load("config/Default.toml")?;
//!     server::start(config).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - ✅ ZFS pool and dataset metrics
//! - ✅ Disk temperature and SMART status
//! - ✅ Cloud sync and snapshot task monitoring
//! - ✅ SMB/NFS share and application status
//! - ✅ System alerts and resource usage
//! - ✅ TLS support with optional certificate verification

pub mod config;
pub mod error;
pub mod metrics;
pub mod server;
pub mod truenas;
