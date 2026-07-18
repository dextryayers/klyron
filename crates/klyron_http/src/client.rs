use std::time::Duration;

pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(32)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { client })
    }

    pub fn with_config(config: HttpClientConfig) -> anyhow::Result<Self> {
        let mut builder = reqwest::Client::builder();
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(pool_size) = config.pool_max_idle {
            builder = builder.pool_max_idle_per_host(pool_size);
        }
        if let Some(headers) = config.default_headers {
            for (k, v) in headers {
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    reqwest::header::HeaderValue::from_str(&v),
                ) {
                    builder = builder.default_header(name, val);
                }
            }
        }
        if config.http2_only {
            builder = builder.http2_prior_knowledge();
        }
        Ok(Self {
            client: builder.build()?,
        })
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<String> {
        let resp = self.client.get(url).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn get_bytes(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let resp = self.client.get(url).send().await?;
        Ok(resp.bytes().await?.to_vec())
    }

    pub async fn post(&self, url: &str, body: &str) -> anyhow::Result<String> {
        let resp = self.client.post(url).body(body.to_string()).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn post_json<T: serde::Serialize>(&self, url: &str, data: &T) -> anyhow::Result<String> {
        let resp = self.client.post(url).json(data).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn put(&self, url: &str, body: &str) -> anyhow::Result<String> {
        let resp = self.client.put(url).body(body.to_string()).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn delete(&self, url: &str) -> anyhow::Result<String> {
        let resp = self.client.delete(url).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn head(&self, url: &str) -> anyhow::Result<reqwest::Response> {
        Ok(self.client.head(url).send().await?)
    }

    pub fn inner(&self) -> &reqwest::Client {
        &self.client
    }
}

pub struct HttpClientConfig {
    pub timeout: Option<Duration>,
    pub pool_max_idle: Option<usize>,
    pub default_headers: Option<Vec<(String, String)>>,
    pub http2_only: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            pool_max_idle: Some(32),
            default_headers: None,
            http2_only: false,
        }
    }
}

pub async fn fetch_url(url: &str) -> anyhow::Result<String> {
    let resp = reqwest::get(url).await?;
    Ok(resp.text().await?)
}

pub async fn fetch_json(url: &str) -> anyhow::Result<serde_json::Value> {
    let resp = reqwest::get(url).await?;
    Ok(resp.json().await?)
}
