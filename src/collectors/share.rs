//! Share Metrics Collector
//!
//! Collects status information for SMB and NFS shares.
//!
//! # Metrics Produced
//! - `truenas_share_smb_enabled` - SMB Share Status (1=Enabled, 0=Disabled)
//!   - Labels: name, path
//! - `truenas_share_nfs_enabled` - NFS Share Status (1=Enabled, 0=Disabled)
//!   - Labels: path

use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

/// Collects SMB and NFS share metrics from TrueNAS
///
/// Queries both SMB and NFS share APIs and updates Prometheus metrics with
/// share status (enabled/disabled). Collects from both APIs independently.
///
/// # Arguments
///
/// * `ctx` - Collection context containing the TrueNAS client and metrics collector
///
/// # Returns
///
/// * `Ok(CollectionStatus::Success)` - Successfully collected at least one type of share metrics
/// * `Ok(CollectionStatus::Failed)` - Failed to collect any share metrics (non-fatal, logged as warning)
/// * `Err(_)` - Fatal error that should propagate
pub async fn collect_share_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    let mut any_success = false;

    // Collect SMB shares
    match ctx.client.query_smb_shares().await {
        Ok(shares) => {
            any_success = true;
            for share in shares {
                ctx.metrics.set_bool_metric(
                    &ctx.metrics.share_smb_enabled,
                    &[&share.name, &share.path],
                    share.enabled,
                );
            }
        }
        Err(e) => warn!("Failed to query SMB shares: {}", e),
    }

    // Collect NFS shares
    match ctx.client.query_nfs_shares().await {
        Ok(shares) => {
            any_success = true;
            for share in shares {
                ctx.metrics.set_bool_metric(
                    &ctx.metrics.share_nfs_enabled,
                    &[&share.path],
                    share.enabled,
                );
            }
        }
        Err(e) => warn!("Failed to query NFS shares: {}", e),
    }

    if any_success {
        info!("Updated share metrics");
        Ok(CollectionStatus::Success)
    } else {
        Ok(CollectionStatus::Failed)
    }
}
