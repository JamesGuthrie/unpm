use unpm::manifest::{Dependency, Manifest};

#[test]
fn parse_short_form() {
    let toml = r#"
[dependencies]
"htmx.org" = "2.0.4"
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["htmx.org"];
    assert_eq!(dep.version(), "2.0.4");
    assert_eq!(dep.file(), None);
    assert_eq!(dep.url(), None);
    assert!(dep.ignore_cves().is_empty());
}

#[test]
fn parse_extended_form_with_file() {
    let toml = r#"
[dependencies]
d3 = { version = "7.9.0", file = "dist/d3.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["d3"];
    assert_eq!(dep.version(), "7.9.0");
    assert_eq!(dep.file(), Some("dist/d3.min.js"));
    assert_eq!(dep.url(), None);
}

#[test]
fn parse_url_form() {
    let toml = r#"
[dependencies]
some-lib = { version = "1.0.0", url = "https://example.com/lib.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["some-lib"];
    assert_eq!(dep.version(), "1.0.0");
    assert_eq!(dep.url(), Some("https://example.com/lib.min.js"));
    assert_eq!(dep.file(), None);
}

#[test]
fn parse_ignore_cves() {
    let toml = r#"
[dependencies]
d3 = { version = "7.9.0", file = "dist/d3.min.js", ignore-cves = ["CVE-2024-1234"] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["d3"];
    assert_eq!(dep.ignore_cves(), &["CVE-2024-1234"]);
}

#[test]
fn roundtrip_serialization() {
    let toml_input = r#"
[dependencies]
"htmx.org" = "2.0.4"
d3 = { version = "7.9.0", file = "dist/d3.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml_input).unwrap();
    let serialized = toml::to_string_pretty(&manifest).unwrap();
    let reparsed: Manifest = toml::from_str(&serialized).unwrap();

    assert_eq!(
        manifest.dependencies["htmx.org"].version(),
        reparsed.dependencies["htmx.org"].version()
    );
    assert_eq!(
        manifest.dependencies["d3"].version(),
        reparsed.dependencies["d3"].version()
    );
    assert_eq!(
        manifest.dependencies["d3"].file(),
        reparsed.dependencies["d3"].file()
    );
}

#[test]
fn mixed_short_and_extended() {
    let toml = r#"
[dependencies]
"htmx.org" = "2.0.4"
d3 = { version = "7.9.0", file = "dist/d3.min.js" }
some-lib = { version = "1.0.0", url = "https://example.com/lib.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    assert_eq!(manifest.dependencies.len(), 3);

    assert!(matches!(manifest.dependencies["htmx.org"], Dependency::Short(_)));
    assert!(matches!(manifest.dependencies["d3"], Dependency::Extended { .. }));
}
