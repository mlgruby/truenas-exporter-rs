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

pub async fn collect_nfs_client_count(ctx: &CollectionContext<'_>) -> CollectionResult {
    // Fetch count + per-client detail (NFSv4 and NFSv3) concurrently
    let (count_res, v4_res, v3_res) = tokio::join!(
        ctx.client.query_nfs_client_count(),
        ctx.client.query_nfs4_clients(),
        ctx.client.query_nfs3_clients(),
    );

    match count_res {
        Ok(count) => ctx.metrics.nfs_client_count.set(count as f64),
        Err(e) => warn!("Failed to query NFS client count: {}", e),
    }

    ctx.metrics.nfs_client_info.reset();
    ctx.metrics.nfs_client_seconds_since_renew.reset();

    let mut total = 0usize;

    let emit = |clients: Vec<crate::truenas::types::NfsClient>,
                version: &str,
                metrics: &crate::metrics::MetricsCollector| {
        for c in clients {
            let addr = &c.info.address;
            let name = &c.info.name;
            let status = &c.info.status;
            metrics
                .nfs_client_info
                .with_label_values(&[addr, name, version, status])
                .set(1);
            metrics
                .nfs_client_seconds_since_renew
                .with_label_values(&[addr, name, version])
                .set(c.info.seconds_since_renew as f64);
        }
    };

    match v4_res {
        Ok(clients) => {
            total += clients.len();
            emit(clients, "4", ctx.metrics);
        }
        Err(e) => warn!("Failed to query NFSv4 clients: {}", e),
    }
    match v3_res {
        Ok(clients) => {
            total += clients.len();
            emit(clients, "3", ctx.metrics);
        }
        Err(e) => warn!("Failed to query NFSv3 clients: {}", e),
    }

    info!("Updated NFS client metrics: {} clients", total);
    Ok(CollectionStatus::Success)
}

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
