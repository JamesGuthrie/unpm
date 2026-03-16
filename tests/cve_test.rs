use unpm::cve::CveChecker;

#[tokio::test]
async fn test_check_no_vulnerabilities() {
    let checker = CveChecker::new();
    let vulns = checker.check("htmx.org", "2.0.4").await.unwrap();
    assert!(vulns.is_empty());
}

// r[verify check.cve.query]
#[tokio::test]
async fn test_check_known_vulnerability() {
    let checker = CveChecker::new();
    let vulns = checker.check("lodash", "4.17.20").await.unwrap();
    assert!(!vulns.is_empty());
    assert!(!vulns[0].id.is_empty());
}

#[tokio::test]
async fn test_check_commit_no_vulnerabilities() {
    let checker = CveChecker::new();
    // htmx 2.0.4 release commit
    let vulns = checker
        .check_commit("8bab1cfbb1a75a0488560ecbab57e61a8ad60862")
        .await
        .unwrap();
    assert!(vulns.is_empty());
}

// r[verify check.cve.git-rev]
#[tokio::test]
async fn test_check_commit_known_vulnerability() {
    let checker = CveChecker::new();
    // radare2 commit with known OSV vulnerabilities (OSV-2021-1820 and others)
    let vulns = checker
        .check_commit("775f2b3d8d6d44f3312f9911dcf70b203268f387")
        .await
        .unwrap();
    assert!(!vulns.is_empty());
    assert!(!vulns[0].id.is_empty());
}
