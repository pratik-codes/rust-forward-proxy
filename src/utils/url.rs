//! URL utility functions

use url::Url;

/// Parse URL and extract components
pub fn parse_url(url_str: &str) -> Result<Url, url::ParseError> {
    Url::parse(url_str)
}

/// Extract path from URL
pub fn extract_path(url: &Url) -> String {
    url.path().to_string()
}

/// Extract query string from URL
pub fn extract_query(url: &Url) -> Option<String> {
    url.query().map(|q| q.to_string())
}

/// Check if URL is HTTPS
pub fn is_https(url: &Url) -> bool {
    url.scheme() == "https"
}
