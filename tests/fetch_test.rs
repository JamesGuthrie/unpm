use unpm::fetch::Fetcher;

#[tokio::test]
async fn test_fetch_and_hash() {
    let fetcher = Fetcher::new();
    let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js";
    let result = fetcher.fetch(url).await.unwrap();
    assert!(!result.bytes.is_empty());
    assert_eq!(result.sha256.len(), 64);
}

#[tokio::test]
async fn test_fetch_consistent_hash() {
    let fetcher = Fetcher::new();
    let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js";
    let r1 = fetcher.fetch(url).await.unwrap();
    let r2 = fetcher.fetch(url).await.unwrap();
    assert_eq!(r1.sha256, r2.sha256);
}

// r[verify install.integrity.sha256]
#[tokio::test]
async fn test_verify_mismatch() {
    let fetcher = Fetcher::new();
    let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js";
    let result = fetcher.fetch(url).await.unwrap();
    assert!(!Fetcher::verify(&result.bytes, "definitely_wrong_hash"));
    assert!(Fetcher::verify(&result.bytes, &result.sha256));
}
