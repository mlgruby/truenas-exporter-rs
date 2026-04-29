use super::{CollectionContext, CollectionResult, CollectionStatus};
use tracing::{info, warn};

pub async fn collect_nfs_client_count(ctx: &CollectionContext<'_>) -> CollectionResult {
    let (count_res, v4_res, v3_res) = tokio::join!(
        ctx.client.query_nfs_client_count(),
        ctx.client.query_nfs4_clients(),
        ctx.client.query_nfs3_clients(),
    );

    let count_ok = count_res.is_ok();
    let v4_ok = v4_res.is_ok();
    let v3_ok = v3_res.is_ok();

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

    if count_ok || v4_ok || v3_ok {
        info!("Updated NFS client metrics: {} clients", total);
        Ok(CollectionStatus::Success)
    } else {
        Ok(CollectionStatus::Failed)
    }
}
