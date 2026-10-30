//! Engine- and UI-independent application logic for Watra Browser.

use watra_engine_api::{BrowserEngine, NavigationRequest};

/// Coordinates application use cases through an engine boundary.
pub struct BrowserCore<E> {
    engine: E,
    current_url: Option<String>,
}

impl<E: BrowserEngine> BrowserCore<E> {
    /// Creates the core with an engine adapter supplied by the composition root.
    pub fn new(engine: E) -> Self {
        Self {
            engine,
            current_url: None,
        }
    }

    /// Navigates through the engine and records the URL only after success.
    pub fn navigate(&mut self, request: NavigationRequest) -> Result<(), E::Error> {
        self.engine.navigate(&request)?;
        self.current_url = Some(request.url().to_owned());
        Ok(())
    }

    /// Returns the last successfully requested URL.
    pub fn current_url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::BrowserCore;
    use watra_engine_api::{BrowserEngine, NavigationRequest};

    #[derive(Default)]
    struct RecordingEngine {
        visited: Vec<String>,
    }

    impl BrowserEngine for RecordingEngine {
        type Error = Infallible;

        fn navigate(&mut self, request: &NavigationRequest) -> Result<(), Self::Error> {
            self.visited.push(request.url().to_owned());
            Ok(())
        }
    }

    #[test]
    fn records_a_successful_navigation() {
        let mut core = BrowserCore::new(RecordingEngine::default());
        let request = NavigationRequest::new("https://example.com").unwrap();

        core.navigate(request).unwrap();

        assert_eq!(core.current_url(), Some("https://example.com"));
    }
}
