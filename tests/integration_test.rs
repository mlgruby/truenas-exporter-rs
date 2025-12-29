use truenas_exporter::config::Config;

#[test]
fn test_config_load() {
    // This assumes config/Default.toml exists relative to where cargo test is run
    let config_res = Config::load("config/Default.toml");
    assert!(config_res.is_ok(), "Failed to load default config");
}
