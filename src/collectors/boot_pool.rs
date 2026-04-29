use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

pub async fn collect_boot_pool_metrics(ctx: &CollectionContext<'_>) -> CollectionResult {
    match ctx.client.query_boot_pool().await {
        Ok(pool) => {
            ctx.metrics
                .boot_pool_health
                .set(if pool.healthy { 1.0 } else { 0.0 });

            if pool.size > 0 {
                ctx.metrics
                    .boot_pool_used_ratio
                    .set(pool.allocated as f64 / pool.size as f64);
            } else {
                ctx.metrics.boot_pool_used_ratio.set(0.0);
            }

            if let Some(scan) = &pool.scan {
                ctx.metrics.boot_pool_scrub_errors.set(scan.errors as f64);

                if let Some(serde_json::Value::Object(map)) = &scan.end_time {
                    if let Some(serde_json::Value::Number(num)) = map.get("$date") {
                        if let Some(millis) = num.as_u64() {
                            ctx.metrics
                                .boot_pool_last_scrub_seconds
                                .set((millis / 1000) as f64);
                        }
                    }
                }
            } else {
                ctx.metrics.boot_pool_scrub_errors.set(0.0);
                ctx.metrics.boot_pool_last_scrub_seconds.set(0.0);
            }

            info!(
                "Updated boot pool metrics (status: {}, healthy: {})",
                pool.status, pool.healthy
            );
            Ok(CollectionStatus::Success)
        }
        Err(e) => {
            warn!("Failed to query boot pool: {}", e);
            Ok(CollectionStatus::Failed)
        }
    }
}
