use anyhow::Result;
use serde::{Deserialize, Serialize};

pub struct CveChecker {
    client: reqwest::Client,
}

#[derive(Debug)]
pub struct Vulnerability {
    pub id: String,
    pub summary: String,
    pub severity: Option<String>,
}

#[derive(Serialize)]
struct ApiRequest {
    package: ApiPackage,
    version: String,
}

#[derive(Serialize)]
struct ApiPackage {
    name: String,
    ecosystem: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    vulns: Option<Vec<ApiVuln>>,
}

#[derive(Deserialize)]
struct ApiVuln {
    id: String,
    summary: Option<String>,
    severity: Option<Vec<ApiSeverity>>,
}

#[derive(Deserialize)]
struct ApiSeverity {
    score: String,
}

impl CveChecker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn check(&self, package_name: &str, version: &str) -> Result<Vec<Vulnerability>> {
        let body = ApiRequest {
            package: ApiPackage {
                name: package_name.to_string(),
                ecosystem: "npm".to_string(),
            },
            version: version.to_string(),
        };

        let resp: ApiResponse = self
            .client
            .post("https://api.osv.dev/v1/query")
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let vulns = resp
            .vulns
            .unwrap_or_default()
            .into_iter()
            .map(|v| Vulnerability {
                id: v.id,
                summary: v.summary.unwrap_or_default(),
                severity: v.severity.and_then(|s| s.first().map(|s| s.score.clone())),
            })
            .collect();

        Ok(vulns)
    }
}
