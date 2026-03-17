use std::collections::BTreeMap;
use unpm::manifest::{Dependency, Manifest};

// r[verify manifest.dep.short]
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

    assert!(dep.ignore_cves().is_empty());
}

// r[verify manifest.dep.extended]
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
}

// r[verify manifest.field.ignore-cves]
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

// r[verify manifest.serial.short]
// r[verify manifest.serial.extended]
// r[verify manifest.serial.order]
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

// r[verify manifest.dep.short]
// r[verify manifest.dep.extended]
#[test]
fn mixed_short_and_extended() {
    let toml = r#"
[dependencies]
"htmx.org" = "2.0.4"
d3 = { version = "7.9.0", file = "dist/d3.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    assert_eq!(manifest.dependencies.len(), 2);

    assert!(matches!(
        manifest.dependencies["htmx.org"],
        Dependency::Short(_)
    ));
    assert!(matches!(
        manifest.dependencies["d3"],
        Dependency::Extended { .. }
    ));
}

// r[verify manifest.source.github-prefix]
#[test]
fn parse_github_source() {
    let toml = r#"
[dependencies]
"gh:alpinejs/alpine" = { version = "3.14.8", file = "packages/alpine/dist/cdn.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["gh:alpinejs/alpine"];
    assert_eq!(dep.version(), "3.14.8");
    assert_eq!(dep.file(), Some("packages/alpine/dist/cdn.min.js"));
}

// r[verify manifest.serial.key-quoting]
#[test]
fn inline_table_format_roundtrips() {
    // Verify that inline TOML table format parses correctly
    let contents = "[dependencies]\n\
         \"gh:user/repo\" = { version = \"1.0.0\", file = \"dist/lib.js\" }\n\
         \"htmx.org\" = \"2.0.4\"\n";
    let reparsed: Manifest = toml::from_str(contents).unwrap();
    assert_eq!(reparsed.dependencies.len(), 2);
    assert_eq!(reparsed.dependencies["htmx.org"].version(), "2.0.4");
    assert_eq!(reparsed.dependencies["gh:user/repo"].version(), "1.0.0");
    assert_eq!(
        reparsed.dependencies["gh:user/repo"].file(),
        Some("dist/lib.js")
    );
}

// Task 3 tests

// r[verify manifest.dep.extended]
#[test]
fn parse_files_form() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", files = ["dist/uPlot.min.js", "dist/uPlot.min.css"] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["uplot"];
    assert_eq!(dep.version(), "1.6.31");
    assert_eq!(
        dep.files(),
        Some(
            &[
                "dist/uPlot.min.js".to_string(),
                "dist/uPlot.min.css".to_string()
            ][..]
        )
    );
    assert_eq!(dep.file(), None);
}

// r[verify manifest.dep.extended]
#[test]
fn files_single_element_valid() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", files = ["dist/uPlot.min.js"] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["uplot"];
    assert_eq!(dep.files(), Some(&["dist/uPlot.min.js".to_string()][..]));
}

// r[verify manifest.serial.extended]
#[test]
fn save_roundtrip_with_files() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("unpm.toml");

    let manifest = Manifest {
        dependencies: BTreeMap::from([(
            "uplot".to_string(),
            Dependency::Extended {
                version: "1.6.31".to_string(),
                source: None,
                file: None,
                files: Some(vec![
                    "dist/uPlot.min.js".to_string(),
                    "dist/uPlot.min.css".to_string(),
                ]),

                ignore_cves: Vec::new(),
            },
        )]),
    };

    manifest.save_to(&manifest_path).unwrap();
    let reparsed = Manifest::load_from(&manifest_path).unwrap();
    let dep = &reparsed.dependencies["uplot"];
    assert_eq!(dep.version(), "1.6.31");
    assert_eq!(
        dep.files(),
        Some(
            &[
                "dist/uPlot.min.js".to_string(),
                "dist/uPlot.min.css".to_string()
            ][..]
        )
    );
    assert_eq!(dep.file(), None);
}

// r[verify manifest.validation.file-files]
#[test]
fn reject_file_and_files() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", file = "dist/uPlot.min.js", files = ["dist/uPlot.min.css"] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let err = manifest.validate().unwrap_err();
    assert!(
        err.to_string().contains("mutually exclusive"),
        "expected mutually exclusive error, got: {err}"
    );
}

// r[verify manifest.validation.files-empty]
#[test]
fn reject_empty_files() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", files = [] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let err = manifest.validate().unwrap_err();
    assert!(
        err.to_string().contains("must not be empty"),
        "expected empty error, got: {err}"
    );
}
