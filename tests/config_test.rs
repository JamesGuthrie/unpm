use unpm::config::Config;

// r[verify config.output-dir.default]
#[test]
fn default_output_dir() {
    let config = Config::default();
    assert_eq!(config.output_dir, "static/vendor");
}

// r[verify config.format.empty]
// r[verify config.canonical.default]
#[test]
fn parse_empty_toml_uses_defaults() {
    let config: Config = toml::from_str("").unwrap();
    assert_eq!(config.output_dir, "static/vendor");
}

// r[verify config.output-dir.custom]
#[test]
fn parse_custom_output_dir() {
    let config: Config = toml::from_str(r#"output_dir = "assets/lib""#).unwrap();
    assert_eq!(config.output_dir, "assets/lib");
}
