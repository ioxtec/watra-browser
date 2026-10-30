//! Stable, engine-agnostic contracts used by Watra's application core.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// A navigation request passed to a browser-engine adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationRequest {
    url: String,
}

impl NavigationRequest {
    /// Creates a request when the URL is non-empty.
    pub fn new(url: impl Into<String>) -> Result<Self, InvalidNavigationRequest> {
        let url = url.into();
        if url.trim().is_empty() {
            return Err(InvalidNavigationRequest);
        }

        Ok(Self { url })
    }

    /// Returns the URL exactly as supplied by the caller.
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// Returned when a navigation request contains no destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidNavigationRequest;

impl Display for InvalidNavigationRequest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("navigation URL cannot be empty")
    }
}

impl Error for InvalidNavigationRequest {}

/// Minimal boundary implemented by a future engine adapter.
pub trait BrowserEngine {
    /// Adapter-specific error type.
    type Error: Error + Send + Sync + 'static;

    /// Requests navigation without exposing engine-specific types to the core.
    fn navigate(&mut self, request: &NavigationRequest) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::NavigationRequest;

    #[test]
    fn accepts_a_non_empty_url() {
        let request = NavigationRequest::new("https://example.com").unwrap();

        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn rejects_a_blank_url() {
        assert!(NavigationRequest::new("  ").is_err());
    }
}
