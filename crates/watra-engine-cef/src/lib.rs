//! CEF adapter for Watra's engine boundary.
//!
//! The crate can be tested without downloading CEF. Enable the `cef-runtime`
//! feature only when building the real adapter against the pinned binary SDK.

use std::error::Error;

use watra_engine_api::{BrowserEngine, BrowserEngineError, BrowserSnapshot, NavigationRequest};

#[cfg(feature = "cef-runtime")]
mod client;

#[cfg(feature = "cef-runtime")]
pub use client::create_client;

/// Rust binding version pinned to the matching CEF binary distribution.
pub const CEF_RUST_VERSION: &str = "156.2.0+156.0.14";

/// Narrow browser-session capability required by Watra's CEF adapter.
///
/// Keeping this seam small lets the application core and unit tests run without
/// loading the native, multi-process CEF runtime.
pub trait CefBrowserSession {
    /// Navigation failure reported by the concrete session.
    type Error: Error + Send + Sync + 'static;

    /// Loads a URL in the session's main frame.
    fn load_url(&mut self, url: &str) -> Result<(), Self::Error>;

    /// Controls browser history and loading without exposing CEF types.
    fn go_back(&mut self) -> Result<(), Self::Error>;
    fn go_forward(&mut self) -> Result<(), Self::Error>;
    fn reload(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;

    /// Returns the engine-neutral state currently available from the session.
    fn snapshot(&self) -> BrowserSnapshot;
}

/// Adapts a live CEF browser session to Watra's engine contract.
pub struct CefEngine<S> {
    session: S,
}

impl<S> CefEngine<S> {
    /// Creates an adapter around an initialized browser session.
    pub fn new(session: S) -> Self {
        Self { session }
    }

    /// Returns the wrapped session, for orderly runtime shutdown.
    pub fn into_session(self) -> S {
        self.session
    }
}

impl<S: CefBrowserSession> BrowserEngine for CefEngine<S> {
    fn navigate(&mut self, request: &NavigationRequest) -> Result<(), BrowserEngineError> {
        self.session
            .load_url(request.url())
            .map_err(|error| BrowserEngineError::new("navigate", error))
    }

    fn go_back(&mut self) -> Result<(), BrowserEngineError> {
        self.session
            .go_back()
            .map_err(|error| BrowserEngineError::new("go back", error))
    }

    fn go_forward(&mut self) -> Result<(), BrowserEngineError> {
        self.session
            .go_forward()
            .map_err(|error| BrowserEngineError::new("go forward", error))
    }

    fn reload(&mut self) -> Result<(), BrowserEngineError> {
        self.session
            .reload()
            .map_err(|error| BrowserEngineError::new("reload", error))
    }

    fn stop(&mut self) -> Result<(), BrowserEngineError> {
        self.session
            .stop()
            .map_err(|error| BrowserEngineError::new("stop", error))
    }

    fn snapshot(&self) -> BrowserSnapshot {
        self.session.snapshot()
    }
}

#[cfg(feature = "cef-runtime")]
mod runtime {
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};

    use cef::{Browser, CefString, ImplBrowser, ImplFrame};

    use super::CefBrowserSession;

    /// A live browser object owned by the CEF lifecycle layer.
    pub struct LiveCefBrowser {
        browser: Browser,
    }

    impl LiveCefBrowser {
        /// Wraps a browser received after CEF reports successful creation.
        pub fn new(browser: Browser) -> Self {
            Self { browser }
        }

        /// Returns the underlying CEF browser handle.
        pub fn browser(&self) -> &Browser {
            &self.browser
        }
    }

    impl CefBrowserSession for LiveCefBrowser {
        type Error = CefNavigationError;

        fn load_url(&mut self, url: &str) -> Result<(), Self::Error> {
            let frame = self
                .browser
                .main_frame()
                .ok_or(CefNavigationError::MainFrameUnavailable)?;

            frame.load_url(Some(&CefString::from(url)));
            Ok(())
        }

        fn go_back(&mut self) -> Result<(), Self::Error> {
            self.browser.go_back();
            Ok(())
        }

        fn go_forward(&mut self) -> Result<(), Self::Error> {
            self.browser.go_forward();
            Ok(())
        }

        fn reload(&mut self) -> Result<(), Self::Error> {
            self.browser.reload();
            Ok(())
        }

        fn stop(&mut self) -> Result<(), Self::Error> {
            self.browser.stop_load();
            Ok(())
        }

        fn snapshot(&self) -> super::BrowserSnapshot {
            let frame = self.browser.main_frame();
            let url = frame
                .as_ref()
                .map(|frame| CefString::from(&frame.url()).to_string())
                .filter(|url| !url.is_empty());

            super::BrowserSnapshot {
                url,
                title: None,
                is_loading: self.browser.is_loading() != 0,
                can_go_back: self.browser.can_go_back() != 0,
                can_go_forward: self.browser.can_go_forward() != 0,
            }
        }
    }

    /// Failure to dispatch navigation to a live CEF browser.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CefNavigationError {
        /// CEF has not exposed a usable main frame yet, or is shutting down.
        MainFrameUnavailable,
    }

    impl Display for CefNavigationError {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Self::MainFrameUnavailable => formatter.write_str("CEF main frame is unavailable"),
            }
        }
    }

    impl Error for CefNavigationError {}
}

#[cfg(feature = "cef-runtime")]
pub use runtime::{CefNavigationError, LiveCefBrowser};

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::{CEF_RUST_VERSION, CefBrowserSession, CefEngine};
    use watra_engine_api::{BrowserEngine, BrowserSnapshot, NavigationRequest};

    #[derive(Default)]
    struct RecordingSession {
        loaded_urls: Vec<String>,
        commands: Vec<&'static str>,
    }

    impl CefBrowserSession for RecordingSession {
        type Error = Infallible;

        fn load_url(&mut self, url: &str) -> Result<(), Self::Error> {
            self.loaded_urls.push(url.to_owned());
            Ok(())
        }

        fn go_back(&mut self) -> Result<(), Self::Error> {
            self.commands.push("back");
            Ok(())
        }

        fn go_forward(&mut self) -> Result<(), Self::Error> {
            self.commands.push("forward");
            Ok(())
        }

        fn reload(&mut self) -> Result<(), Self::Error> {
            self.commands.push("reload");
            Ok(())
        }

        fn stop(&mut self) -> Result<(), Self::Error> {
            self.commands.push("stop");
            Ok(())
        }

        fn snapshot(&self) -> BrowserSnapshot {
            BrowserSnapshot::default()
        }
    }

    #[test]
    fn forwards_navigation_to_the_cef_session_boundary() {
        let mut engine = CefEngine::new(RecordingSession::default());
        let request = NavigationRequest::new("https://example.com").unwrap();

        engine.navigate(&request).unwrap();
        let session = engine.into_session();

        assert_eq!(session.loaded_urls, ["https://example.com"]);
    }

    #[test]
    fn exposes_the_pinned_binding_version() {
        assert_eq!(CEF_RUST_VERSION, "156.2.0+156.0.14");
    }

    #[test]
    fn forwards_controls_to_the_cef_session_boundary() {
        let mut engine = CefEngine::new(RecordingSession::default());

        engine.go_back().unwrap();
        engine.go_forward().unwrap();
        engine.reload().unwrap();
        engine.stop().unwrap();

        assert_eq!(
            engine.into_session().commands,
            ["back", "forward", "reload", "stop"]
        );
    }

    #[test]
    fn maps_adapter_errors_to_an_engine_neutral_error() {
        struct FailingSession;

        impl CefBrowserSession for FailingSession {
            type Error = std::io::Error;

            fn load_url(&mut self, _: &str) -> Result<(), Self::Error> {
                Err(std::io::Error::other("session unavailable"))
            }

            fn go_back(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
            fn go_forward(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
            fn reload(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
            fn stop(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
            fn snapshot(&self) -> BrowserSnapshot {
                BrowserSnapshot::default()
            }
        }

        let mut engine = CefEngine::new(FailingSession);
        let request = NavigationRequest::new("https://example.com").unwrap();
        let error = BrowserEngine::navigate(&mut engine, &request).unwrap_err();

        assert_eq!(error.operation(), "navigate");
        assert!(error.to_string().contains("session unavailable"));
    }
}
