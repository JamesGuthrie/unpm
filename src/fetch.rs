use anyhow::{bail, Result};
use sha2::{Digest, Sha256};

/// 50 MB — no vendored static asset should be anywhere near this.
const MAX_RESPONSE_SIZE: u64 = 50 * 1024 * 1024;

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
        log::debug!("fetching {url}");
        let response = self.client.get(url).send().await?.error_for_status()?;

        if let Some(len) = response.content_length() {
            if len > MAX_RESPONSE_SIZE {
                bail!(
                    "Response too large ({} bytes, max {}). Aborting download from {url}",
                    len,
                    MAX_RESPONSE_SIZE
                );
            }
        }

        let bytes = response.bytes().await?.to_vec();

        if bytes.len() as u64 > MAX_RESPONSE_SIZE {
            bail!(
                "Response too large ({} bytes, max {}). Aborting download from {url}",
                bytes.len(),
                MAX_RESPONSE_SIZE
            );
        }

        let sha256 = Self::hash(&bytes);
        let size = bytes.len() as u64;

        log::debug!("  -> {} bytes, sha256: {sha256}", size);

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
