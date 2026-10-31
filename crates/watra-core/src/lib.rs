//! Engine- and UI-independent application logic for Watra Browser.

use watra_engine_api::{BrowserEngine, BrowserEngineError, BrowserSnapshot, NavigationRequest};

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
    pub fn navigate(&mut self, request: NavigationRequest) -> Result<(), BrowserEngineError> {
        self.engine.navigate(&request)?;
        self.current_url = Some(request.url().to_owned());
        Ok(())
    }

    /// Navigates to the previous history entry.
    pub fn go_back(&mut self) -> Result<(), BrowserEngineError> {
        self.engine.go_back()
    }

    /// Navigates to the next history entry.
    pub fn go_forward(&mut self) -> Result<(), BrowserEngineError> {
        self.engine.go_forward()
    }

    /// Reloads the current page.
    pub fn reload(&mut self) -> Result<(), BrowserEngineError> {
        self.engine.reload()
    }

    /// Stops the current navigation.
    pub fn stop(&mut self) -> Result<(), BrowserEngineError> {
        self.engine.stop()
    }

    /// Reads the state reported by the active rendering engine.
    pub fn snapshot(&self) -> BrowserSnapshot {
        self.engine.snapshot()
    }

    /// Returns the last successfully requested URL.
    pub fn current_url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::BrowserCore;
    use watra_engine_api::{BrowserEngine, BrowserEngineError, BrowserSnapshot, NavigationRequest};

    #[derive(Default)]
    struct RecordingEngine {
        visited: Vec<String>,
        commands: Vec<&'static str>,
    }

    impl BrowserEngine for RecordingEngine {
        fn navigate(&mut self, request: &NavigationRequest) -> Result<(), BrowserEngineError> {
            self.visited.push(request.url().to_owned());
            Ok(())
        }

        fn go_back(&mut self) -> Result<(), BrowserEngineError> {
            self.commands.push("back");
            Ok(())
        }

        fn go_forward(&mut self) -> Result<(), BrowserEngineError> {
            self.commands.push("forward");
            Ok(())
        }

        fn reload(&mut self) -> Result<(), BrowserEngineError> {
            self.commands.push("reload");
            Ok(())
        }

        fn stop(&mut self) -> Result<(), BrowserEngineError> {
            self.commands.push("stop");
            Ok(())
        }

        fn snapshot(&self) -> BrowserSnapshot {
            BrowserSnapshot::default()
        }
    }

    #[test]
    fn records_a_successful_navigation() {
        let mut core = BrowserCore::new(RecordingEngine::default());
        let request = NavigationRequest::new("https://example.com").unwrap();

        core.navigate(request).unwrap();

        assert_eq!(core.current_url(), Some("https://example.com"));
    }

    #[test]
    fn forwards_session_commands_through_the_engine_boundary() {
        let mut core = BrowserCore::new(RecordingEngine::default());

        core.go_back().unwrap();
        core.go_forward().unwrap();
        core.reload().unwrap();
        core.stop().unwrap();

        assert_eq!(core.engine.commands, ["back", "forward", "reload", "stop"]);
    }

    #[test]
    fn accepts_an_engine_selected_behind_a_runtime_interface() {
        let engine: Box<dyn BrowserEngine> = Box::new(RecordingEngine::default());
        let mut core = BrowserCore::new(engine);
        let request = NavigationRequest::new("https://example.com").unwrap();

        core.navigate(request).unwrap();
        core.go_back().unwrap();

        assert_eq!(core.current_url(), Some("https://example.com"));
        assert_eq!(core.snapshot(), BrowserSnapshot::default());
    }
}
