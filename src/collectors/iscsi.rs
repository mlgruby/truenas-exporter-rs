use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

pub async fn collect_iscsi_client_count(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_iscsi_client_count().await {
        Ok(count) => {
            ctx.metrics.iscsi_client_count.set(count as f64);
            info!("Updated iSCSI client count: {}", count);
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query iSCSI client count: {}", e);
            Ok(CollectionStatus::Failed)
        }
    }
}
