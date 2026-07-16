use crate::PmError;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            name: "lodash".into(),
            description: Some("A utility library".into()),
            version: "4.17.21".into(),
            downloads: Some(1000000),
            updated_at: Some("2024-01-01".into()),
        };
        assert_eq!(result.name, "lodash");
        assert_eq!(result.version, "4.17.21");
        assert_eq!(result.description.as_deref(), Some("A utility library"));
    }

    #[test]
    fn test_search_result_no_description() {
        let result = SearchResult {
            name: "foo".into(),
            description: None,
            version: "1.0.0".into(),
            downloads: None,
            updated_at: None,
        };
        assert!(result.description.is_none());
        assert!(result.downloads.is_none());
    }

    #[test]
    fn test_search_results_pagination() {
        let results = SearchResults {
            results: vec![
                SearchResult {
                    name: "a".into(), description: None, version: "1.0.0".into(),
                    downloads: None, updated_at: None,
                },
            ],
            total: 1,
            page: 1,
            limit: 20,
        };
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.page, 1);
        assert_eq!(results.limit, 20);
    }

    #[test]
    fn test_urlencoding_simple() {
        assert_eq!(urlencoding("hello"), "hello");
        assert_eq!(urlencoding("hello world"), "hello+world");
    }

    #[test]
    fn test_urlencoding_special_chars() {
        assert_eq!(urlencoding("@scope/pkg"), "%40scope%2Fpkg");
    }

    #[test]
    fn test_urlencoding_alphanumeric() {
        let encoded = urlencoding("abc123-_.~");
        assert_eq!(encoded, "abc123-_.~");
    }

    #[test]
    fn test_search_query_with_qualifiers() {
        let query = "react";
        let by_author = Some("facebook");
        let mut qualifiers = Vec::new();
        qualifiers.push(format!("author:{}", by_author.unwrap()));
        let search_query = format!("{} {}", qualifiers.join(" "), query);
        assert_eq!(search_query, "author:facebook react");
    }

    #[test]
    fn test_search_with_exact_match() {
        let query = "react";
        let exact = true;
        let search_query = if exact { format!("\"{}\"", query) } else { query.to_string() };
        assert_eq!(search_query, "\"react\"");
    }

    #[test]
    fn test_limit_capping() {
        let limit = 300u32;
        let size = if limit > 250 { 250 } else { limit };
        assert_eq!(size, 250);
    }

    #[test]
    fn test_search_result_ordering() {
        let mut results = vec![
            SearchResult {
                name: "z-pkg".into(), description: None, version: "1.0.0".into(),
                downloads: Some(10), updated_at: None,
            },
            SearchResult {
                name: "a-pkg".into(), description: None, version: "1.0.0".into(),
                downloads: Some(1000), updated_at: None,
            },
        ];
        results.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(results[0].name, "a-pkg");
        assert_eq!(results[1].name, "z-pkg");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub downloads: Option<u64>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

pub fn search_packages(
    query: &str,
    exact: bool,
    by_author: Option<&str>,
    by_description: Option<&str>,
    page: u32,
    limit: u32,
) -> Result<SearchResults, PmError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| PmError::IoError(format!("HTTP client: {e}")))?;

    let size = if limit > 250 { 250 } else { limit };
    let from = ((page.saturating_sub(1)) * size).to_string();

    let mut search_query = query.to_string();

    if exact {
        search_query = format!("\"{}\"", query);
    }

    let mut qualifiers = Vec::new();
    if let Some(author) = by_author {
        qualifiers.push(format!("author:{author}"));
    }
    if let Some(desc) = by_description {
        qualifiers.push(format!("keywords:{desc}"));
    }
    if !qualifiers.is_empty() {
        search_query = format!("{} {}", qualifiers.join(" "), search_query);
    }

    let url = format!(
        "https://registry.npmjs.org/-/v1/search?text={}&size={}&from={}",
        urlencoding(&search_query),
        size,
        from
    );

    let resp = client.get(&url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| PmError::IoError(format!("Search request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(PmError::IoError(format!(
            "Search failed: HTTP {}",
            resp.status()
        )));
    }

    let body: serde_json::Value = resp.json()
        .map_err(|e| PmError::IoError(format!("Parse failed: {e}")))?;

    let total = body.get("total")
        .and_then(|t| t.as_u64())
        .unwrap_or(0);

    let mut results = Vec::new();

    if let Some(objects) = body.get("objects").and_then(|o| o.as_array()) {
        for obj in objects {
            if let Some(pkg) = obj.get("package") {
                let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
                let description = pkg.get("description").and_then(|d| d.as_str()).map(String::from);
                let downloads = obj.get("downloads")
                    .and_then(|d| d.as_object())
                    .and_then(|d| d.get("monthly"))
                    .and_then(|d| d.as_u64());
                let updated_at = pkg.get("date").and_then(|d| d.as_str()).map(String::from);

                if !name.is_empty() {
                    results.push(SearchResult {
                        name,
                        description,
                        version,
                        downloads,
                        updated_at,
                    });
                }
            }
        }
    }

    Ok(SearchResults {
        total,
        results,
        page,
        limit: size,
    })
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        c => format!("%{:02X}", c as u8),
    }).collect()
}
