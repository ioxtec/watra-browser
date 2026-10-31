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

/// Current browser state exposed without leaking engine-specific types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrowserSnapshot {
    pub url: Option<String>,
    pub title: Option<String>,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

/// Error returned by an engine adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserEngineError {
    operation: &'static str,
    details: String,
}

impl BrowserEngineError {
    /// Wraps an adapter-specific error with the operation that failed.
    pub fn new(operation: &'static str, error: impl Display) -> Self {
        Self {
            operation,
            details: error.to_string(),
        }
    }

    /// Returns the failed browser operation.
    pub fn operation(&self) -> &'static str {
        self.operation
    }
}

impl Display for BrowserEngineError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} failed: {}", self.operation, self.details)
    }
}

impl Error for BrowserEngineError {}

/// Browser-session operations shared by every rendering engine adapter.
pub trait BrowserEngine {
    /// Requests navigation without exposing engine-specific types to the core.
    fn navigate(&mut self, request: &NavigationRequest) -> Result<(), BrowserEngineError>;

    /// Navigates to the previous history entry.
    fn go_back(&mut self) -> Result<(), BrowserEngineError>;

    /// Navigates to the next history entry.
    fn go_forward(&mut self) -> Result<(), BrowserEngineError>;

    /// Reloads the current page.
    fn reload(&mut self) -> Result<(), BrowserEngineError>;

    /// Stops the current navigation.
    fn stop(&mut self) -> Result<(), BrowserEngineError>;

    /// Returns the latest engine-neutral view of this browser session.
    fn snapshot(&self) -> BrowserSnapshot;
}

impl<E: BrowserEngine + ?Sized> BrowserEngine for Box<E> {
    fn navigate(&mut self, request: &NavigationRequest) -> Result<(), BrowserEngineError> {
        (**self).navigate(request)
    }

    fn go_back(&mut self) -> Result<(), BrowserEngineError> {
        (**self).go_back()
    }

    fn go_forward(&mut self) -> Result<(), BrowserEngineError> {
        (**self).go_forward()
    }

    fn reload(&mut self) -> Result<(), BrowserEngineError> {
        (**self).reload()
    }

    fn stop(&mut self) -> Result<(), BrowserEngineError> {
        (**self).stop()
    }

    fn snapshot(&self) -> BrowserSnapshot {
        (**self).snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::{BrowserEngineError, BrowserSnapshot, NavigationRequest};

    #[test]
    fn accepts_a_non_empty_url() {
        let request = NavigationRequest::new("https://example.com").unwrap();

        assert_eq!(request.url(), "https://example.com");
    }

    #[test]
    fn rejects_a_blank_url() {
        assert!(NavigationRequest::new("  ").is_err());
    }

    #[test]
    fn engine_error_keeps_operation_context() {
        let error = BrowserEngineError::new("reload", "runtime unavailable");

        assert_eq!(error.operation(), "reload");
        assert_eq!(error.to_string(), "reload failed: runtime unavailable");
    }

    #[test]
    fn browser_snapshot_defaults_to_an_empty_session() {
        assert_eq!(
            BrowserSnapshot::default(),
            BrowserSnapshot {
                url: None,
                title: None,
                is_loading: false,
                can_go_back: false,
                can_go_forward: false,
            }
        );
    }
}
