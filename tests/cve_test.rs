use unpm::cve::CveChecker;

#[tokio::test]
async fn test_check_no_vulnerabilities() {
    let checker = CveChecker::new();
    let vulns = checker.check("htmx.org", "2.0.4").await.unwrap();
    assert!(vulns.is_empty());
}

#[tokio::test]
async fn test_check_known_vulnerability() {
    let checker = CveChecker::new();
    let vulns = checker.check("lodash", "4.17.20").await.unwrap();
    assert!(!vulns.is_empty());
    assert!(!vulns[0].id.is_empty());
}
