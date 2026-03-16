use anyhow::{Result, bail};

/// Extract the file path portion from a jsdelivr CDN URL.
/// e.g. "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js" -> "dist/htmx.min.js"
pub fn extract_file_path(url: &str, version: &str) -> Result<String> {
    let marker = format!("@{version}/");
    let idx = url
        .find(&marker)
        .ok_or_else(|| anyhow::anyhow!("Cannot parse file path from URL: {url}"))?;
    let path = &url[idx + marker.len()..];
    if path.is_empty() {
        bail!("No file path found in URL: {url}");
    }
    Ok(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[verify update.files.path-extraction]
    #[test]
    fn extracts_npm_path() {
        let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js";
        assert_eq!(extract_file_path(url, "2.0.4").unwrap(), "dist/htmx.min.js");
    }

    // r[verify update.files.path-extraction]
    #[test]
    fn extracts_github_path() {
        let url = "https://cdn.jsdelivr.net/gh/user/repo@1.0.0/dist/lib.js";
        assert_eq!(extract_file_path(url, "1.0.0").unwrap(), "dist/lib.js");
    }

    #[test]
    fn errors_on_missing_version() {
        let url = "https://cdn.jsdelivr.net/npm/htmx.org/dist/htmx.min.js";
        assert!(extract_file_path(url, "2.0.4").is_err());
    }

    #[test]
    fn errors_on_empty_path() {
        let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/";
        assert!(extract_file_path(url, "2.0.4").is_err());
    }
}
