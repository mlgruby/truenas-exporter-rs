use serde_json::json;
use truenas_exporter::truenas::types::*;

#[test]
fn test_deserialize_boot_pool_state_full() {
    // Given: full boot pool response with scan and $date end_time
    let json = json!({
        "name": "boot-pool",
        "status": "ONLINE",
        "healthy": true,
        "size": 100_000_000u64,
        "allocated": 20_000_000u64,
        "scan": {
            "state": "FINISHED",
            "errors": 0,
            "end_time": {"$date": 1_700_000_000_000u64}
        }
    });

    // When: deserialized
    let pool: BootPoolState = serde_json::from_value(json).expect("Failed to parse BootPoolState");

    // Then: all fields populated correctly
    assert_eq!(pool.name, "boot-pool");
    assert!(pool.healthy);
    assert_eq!(pool.size, 100_000_000);
    assert_eq!(pool.allocated, 20_000_000);
    let scan = pool.scan.expect("scan should be present");
    assert_eq!(scan.errors, 0);
    assert!(scan.end_time.is_some());
}

#[test]
fn test_deserialize_boot_pool_state_missing_optional_fields() {
    // Given: minimal boot pool response (no scan, no size/allocated)
    let json = json!({
        "name": "boot-pool",
        "status": "ONLINE",
        "healthy": true
    });

    // When: deserialized
    let pool: BootPoolState = serde_json::from_value(json).expect("Failed to parse BootPoolState");

    // Then: optional fields default correctly
    assert!(pool.scan.is_none());
    assert_eq!(pool.size, 0);
    assert_eq!(pool.allocated, 0);
}

#[test]
fn test_boot_pool_used_ratio_zero_size_guard() {
    // Given: boot pool with size == 0 (division guard needed)
    let pool = BootPoolState {
        name: "boot-pool".to_string(),
        status: "ONLINE".to_string(),
        healthy: true,
        size: 0,
        allocated: 0,
        scan: None,
    };

    // When: ratio computed with guard
    let ratio = if pool.size > 0 {
        Some(pool.allocated as f64 / pool.size as f64)
    } else {
        None
    };

    // Then: no panic, ratio is None
    assert!(ratio.is_none());
}

#[test]
fn test_deserialize_nfs_client_info_rename() {
    // Given: NFS client info with space-containing field name
    let json = json!({
        "address": "192.168.10.12:871",
        "name": "Linux NFSv4.2 pve1",
        "status": "confirmed",
        "seconds from last renew": 5u64
    });

    // When: deserialized
    let info: NfsClientInfo = serde_json::from_value(json).expect("Failed to parse NfsClientInfo");

    // Then: renamed field maps correctly
    assert_eq!(info.address, "192.168.10.12:871");
    assert_eq!(info.seconds_since_renew, 5);
    assert_eq!(info.status, "confirmed");
}

#[test]
fn test_deserialize_nfs_client_info_missing_optional_fields() {
    // Given: NFS client info with only address present
    let json = json!({
        "address": "192.168.10.12:871"
    });

    // When: deserialized
    let info: NfsClientInfo = serde_json::from_value(json).expect("Failed to parse NfsClientInfo");

    // Then: optional fields default to empty string / 0
    assert_eq!(info.address, "192.168.10.12:871");
    assert_eq!(info.name, "");
    assert_eq!(info.status, "");
    assert_eq!(info.seconds_since_renew, 0);
}

#[test]
fn test_deserialize_nfs_client_info_missing_address() {
    // Given: NFS client info with address absent (should not panic — #[serde(default)])
    let json = json!({
        "name": "Linux NFSv4.2 pve1",
        "status": "confirmed",
        "seconds from last renew": 3u64
    });

    // When: deserialized
    let info: NfsClientInfo = serde_json::from_value(json).expect("Failed to parse NfsClientInfo");

    // Then: address defaults to empty string, rest populated
    assert_eq!(info.address, "");
    assert_eq!(info.seconds_since_renew, 3);
}

#[test]
fn test_deserialize_pool() {
    let json = json!({
        "id": 1,
        "name": "tank",
        "encrypt": 0,
        "encryptkey": "",
        "topology": {
            "data": [],
            "log": [],
            "cache": [],
            "spare": [],
            "special": [],
            "dedup": []
        },
        "status": "ONLINE",
        "healthy": true,
        "scan": {
            "function": null,
            "state": null,
            "start_time": null,
            "end_time": null,
            "bytes_to_process": null,
            "bytes_processed": null,
            "errors": null
        }
    });

    let pool: Pool = serde_json::from_value(json).expect("Failed to parse Pool");
    assert_eq!(pool.name, "tank");
    assert_eq!(pool.status, "ONLINE");
    // Test default values for scan fields (expect None for null)
    assert_eq!(pool.scan.as_ref().unwrap().bytes_to_process, None);
}

#[test]
fn test_deserialize_dataset() {
    let json = json!({
        "name": "tank/data",
        "used": {"parsed": 1024},
        "available": {"parsed": 2048},
        "compression_ratio": {"parsed": 1.5},
        "encrypted": false
    });

    let dataset: Dataset = serde_json::from_value(json).expect("Failed to parse Dataset");
    assert_eq!(dataset.name, "tank/data");
    assert_eq!(dataset.used.unwrap().parsed, 1024);
}

#[test]
fn test_deserialize_replication_task_full() {
    // Given: replication task with state.datetime populated and no active job
    let json = json!({
        "id": 1,
        "name": "backup-to-remote",
        "direction": "PUSH",
        "transport": "SSH",
        "enabled": true,
        "source_datasets": ["tank/data", "tank/media"],
        "target_dataset": "remote/backup",
        "state": {
            "state": "FINISHED",
            "datetime": {"$date": 1_715_000_000_000u64},
            "error": null
        }
    });

    // When: deserialized
    let task: ReplicationTask =
        serde_json::from_value(json).expect("Failed to parse ReplicationTask");

    // Then: all fields populated correctly
    assert_eq!(task.id, 1);
    assert_eq!(task.name, "backup-to-remote");
    assert_eq!(task.direction, "PUSH");
    assert_eq!(task.transport, "SSH");
    assert!(task.enabled);
    assert_eq!(task.source_datasets, vec!["tank/data", "tank/media"]);
    assert_eq!(task.target_dataset, "remote/backup");

    let state = task.state.expect("state should be present");
    assert_eq!(state.state, "FINISHED");
    assert!(state.error.is_none());

    // datetime must be present and shaped as {"$date": ms}
    let dt = state.datetime.expect("datetime should be present");
    let map = dt.as_object().expect("datetime should be an object");
    assert_eq!(
        map.get("$date").and_then(|v| v.as_u64()),
        Some(1_715_000_000_000)
    );

    assert!(task.job.is_none());
}

#[test]
fn test_deserialize_replication_task_pending_no_state_datetime() {
    // Given: replication task whose zettarepl state is the default `{"state": "PENDING"}`
    // (datetime/error absent — matches the empty zettarepl context case)
    let json = json!({
        "id": 2,
        "name": "fresh-task",
        "direction": "PULL",
        "transport": "LOCAL",
        "enabled": true,
        "state": {
            "state": "PENDING"
        }
    });

    let task: ReplicationTask =
        serde_json::from_value(json).expect("Failed to parse pending ReplicationTask");

    let state = task.state.expect("state should be present");
    assert_eq!(state.state, "PENDING");
    assert!(state.datetime.is_none());
    assert!(state.error.is_none());
    assert!(task.job.is_none());
}

#[test]
fn test_deserialize_replication_task_active_job() {
    // Given: replication mid-flight — state.state=RUNNING and a job with progress
    let json = json!({
        "id": 3,
        "name": "running-task",
        "direction": "PUSH",
        "transport": "SSH+NETCAT",
        "enabled": true,
        "state": {
            "state": "RUNNING"
        },
        "job": {
            "state": "RUNNING",
            "progress": {"percent": 42.5},
            "time_finished": null
        }
    });

    let task: ReplicationTask =
        serde_json::from_value(json).expect("Failed to parse running ReplicationTask");

    let state = task.state.expect("state should be present");
    assert_eq!(state.state, "RUNNING");

    let job = task.job.expect("job should be present mid-flight");
    assert_eq!(job.state, "RUNNING");
    let progress = job.progress.expect("progress should be present");
    assert_eq!(progress.percent, Some(42.5));
}

#[test]
fn test_deserialize_replication_task_minimal() {
    // Given: only the unconditionally-present fields
    let json = json!({
        "id": 4,
        "name": "bare",
        "direction": "PUSH",
        "transport": "SSH",
        "enabled": false
    });

    let task: ReplicationTask =
        serde_json::from_value(json).expect("Failed to parse minimal ReplicationTask");

    assert_eq!(task.name, "bare");
    assert!(!task.enabled);
    assert!(task.source_datasets.is_empty());
    assert_eq!(task.target_dataset, "");
    assert!(task.state.is_none());
    assert!(task.job.is_none());
}

#[test]
fn test_deserialize_alert() {
    let json = json!({
        "uuid": "1234",
        "level": "CRITICAL",
        "formatted": "Error!",
        "dismissed": false
    });
    let alert: TruenasAlert = serde_json::from_value(json).expect("Failed to parse Alert");
    assert_eq!(alert.level, "CRITICAL");
    assert!(!alert.dismissed);
}
