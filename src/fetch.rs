use anyhow::Result;
use sha2::{Digest, Sha256};

pub struct Fetcher {
    client: reqwest::Client,
}

pub struct FetchResult {
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub size: u64,
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn fetch(&self, url: &str) -> Result<FetchResult> {
        let bytes = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec();

        let sha256 = Self::hash(&bytes);
        let size = bytes.len() as u64;

        Ok(FetchResult {
            bytes,
            sha256,
            size,
        })
    }

    pub fn hash(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    pub fn verify(bytes: &[u8], expected_sha256: &str) -> bool {
        Self::hash(bytes) == expected_sha256
    }
}
