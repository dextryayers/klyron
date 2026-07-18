use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Http2Client {
    inner: Arc<Mutex<Http2ClientInner>>,
}

struct Http2ClientInner {
    base_url: String,
    headers: HashMap<String, String>,
    timeout_secs: u64,
}

impl Http2Client {
    pub fn new(base_url: &str) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Http2ClientInner {
                base_url: base_url.trim_end_matches('/').to_string(),
                headers: HashMap::new(),
                timeout_secs: 30,
            })),
        }
    }

    pub fn with_header(self, key: &str, value: &str) -> Self {
        let inner = self.inner.clone();
        let key = key.to_string();
        let value = value.to_string();
        tokio::spawn(async move {
            let mut inner = inner.lock().await;
            inner.headers.insert(key, value);
        });
        self
    }

    pub fn with_timeout(self, secs: u64) -> Self {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let mut inner = inner.lock().await;
            inner.timeout_secs = secs;
        });
        self
    }

    pub async fn get(&self, path: &str) -> anyhow::Result<String> {
        let url = self.build_url(path).await;
        let resp = reqwest::Client::builder()
            .http2_prior_knowledge()
            .build()?
            .get(&url)
            .send()
            .await?;
        Ok(resp.text().await?)
    }

    pub async fn post(&self, path: &str, body: &str) -> anyhow::Result<String> {
        let url = self.build_url(path).await;
        let resp = reqwest::Client::builder()
            .http2_prior_knowledge()
            .build()?
            .post(&url)
            .body(body.to_string())
            .send()
            .await?;
        Ok(resp.text().await?)
    }

    pub async fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        data: &T,
    ) -> anyhow::Result<String> {
        let url = self.build_url(path).await;
        let resp = reqwest::Client::builder()
            .http2_prior_knowledge()
            .build()?
            .post(&url)
            .json(data)
            .send()
            .await?;
        Ok(resp.text().await?)
    }

    async fn build_url(&self, path: &str) -> String {
        let inner = self.inner.lock().await;
        let base = inner.base_url.clone();
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }
}

pub fn is_h2_supported() -> bool {
    true
}

pub async fn h2_ping(url: &str) -> anyhow::Result<bool> {
    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()?;
    let resp = client.get(url).send().await?;
    Ok(resp.status().is_success())
}
