use serde_json::json;
use truenas_exporter::truenas::types::*;

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
