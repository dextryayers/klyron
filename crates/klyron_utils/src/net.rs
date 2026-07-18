use url::Url;

pub struct UrlUtil;

impl UrlUtil {
    pub fn parse(url_str: &str) -> anyhow::Result<Url> {
        Url::parse(url_str).map_err(|e| anyhow::anyhow!("Invalid URL '{url_str}': {e}"))
    }

    pub fn join(base: &Url, path: &str) -> anyhow::Result<Url> {
        base.join(path)
            .map_err(|e| anyhow::anyhow!("Failed to join URL: {e}"))
    }

    pub fn is_http(url: &Url) -> bool {
        matches!(url.scheme(), "http" | "https")
    }

    pub fn is_localhost(url: &Url) -> bool {
        url.host_str()
            .map_or(false, |h| h == "localhost" || h == "127.0.0.1" || h == "::1")
    }

    pub fn query_param(url: &Url, key: &str) -> Option<String> {
        url.query_pairs()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.to_string())
    }

    pub fn query_params(url: &Url, key: &str) -> Vec<String> {
        url.query_pairs()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.to_string())
            .collect()
    }

    pub fn set_query_param(url: &mut Url, key: &str, value: &str) {
        let mut pairs: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        pairs.retain(|(k, _)| k != key);
        pairs.push((key.to_string(), value.to_string()));
        let query: String = pairs
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        url.set_query(Some(&query));
    }

    pub fn normalize(url_str: &str) -> anyhow::Result<String> {
        let mut url = Url::parse(url_str)?;
        let q = url.query().map(|s| s.to_string());
        url.set_query(q.as_deref());
        Ok(url.to_string())
    }
}

pub fn encode_uri_component(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

pub fn decode_uri_component(s: &str) -> String {
    url::form_urlencoded::parse(s.as_bytes())
        .map(|(k, v)| [k, v].concat())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_util_parse() {
        let url = UrlUtil::parse("https://example.com/path").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_url_util_is_http() {
        let url = Url::parse("https://example.com").unwrap();
        assert!(UrlUtil::is_http(&url));
        let url = Url::parse("ftp://example.com").unwrap();
        assert!(!UrlUtil::is_http(&url));
    }

    #[test]
    fn test_url_util_localhost() {
        let url = Url::parse("http://localhost:3000").unwrap();
        assert!(UrlUtil::is_localhost(&url));
        let url = Url::parse("http://example.com").unwrap();
        assert!(!UrlUtil::is_localhost(&url));
    }

    #[test]
    fn test_query_param() {
        let url = Url::parse("https://example.com?foo=bar&baz=qux").unwrap();
        assert_eq!(UrlUtil::query_param(&url, "foo"), Some("bar".into()));
        assert_eq!(UrlUtil::query_param(&url, "nonexistent"), None);
    }

    #[test]
    fn test_encode_decode() {
        let s = "hello world";
        let encoded = encode_uri_component(s);
        assert_eq!(encoded, "hello+world");
    }
}
