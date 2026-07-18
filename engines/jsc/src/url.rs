use url::Url;

pub struct JSCUrl;

impl JSCUrl {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<Url, String> {
        Url::parse(input).map_err(|e| format!("url.parse: {e}"))
    }

    pub fn resolve(&self, base: &str, relative: &str) -> Result<String, String> {
        let base_url = Url::parse(base).map_err(|e| format!("url.resolve base: {e}"))?;
        let resolved = base_url.join(relative).map_err(|e| format!("url.resolve: {e}"))?;
        Ok(resolved.to_string())
    }

    pub fn format(&self, url: &Url) -> String {
        url.to_string()
    }

    pub fn origin(&self, url_str: &str) -> Result<String, String> {
        let url = Url::parse(url_str).map_err(|e| format!("url.origin: {e}"))?;
        Ok(url.origin().ascii_serialization())
    }

    pub fn hostname(&self, url_str: &str) -> Result<String, String> {
        let url = Url::parse(url_str).map_err(|e| format!("url.hostname: {e}"))?;
        Ok(url.host_str().unwrap_or("").to_string())
    }

    pub fn pathname(&self, url_str: &str) -> Result<String, String> {
        let url = Url::parse(url_str).map_err(|e| format!("url.pathname: {e}"))?;
        Ok(url.path().to_string())
    }

    pub fn search(&self, url_str: &str) -> Result<String, String> {
        let url = Url::parse(url_str).map_err(|e| format!("url.search: {e}"))?;
        Ok(url.query().map(|q| format!("?{q}")).unwrap_or_default())
    }

    pub fn hash(&self, url_str: &str) -> Result<String, String> {
        let url = Url::parse(url_str).map_err(|e| format!("url.hash: {e}"))?;
        Ok(url.fragment().map(|f| format!("#{f}")).unwrap_or_default())
    }
}

impl Default for JSCUrl {
    fn default() -> Self {
        Self::new()
    }
}
