//! CEF adapter for Watra's engine boundary.
//!
//! The crate can be tested without downloading CEF. Enable the `cef-runtime`
//! feature only when building the real adapter against the pinned binary SDK.

use std::error::Error;

use watra_engine_api::{BrowserEngine, NavigationRequest};

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
    type Error = S::Error;

    fn navigate(&mut self, request: &NavigationRequest) -> Result<(), Self::Error> {
        self.session.load_url(request.url())
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
    use watra_engine_api::{BrowserEngine, NavigationRequest};

    #[derive(Default)]
    struct RecordingSession {
        loaded_urls: Vec<String>,
    }

    impl CefBrowserSession for RecordingSession {
        type Error = Infallible;

        fn load_url(&mut self, url: &str) -> Result<(), Self::Error> {
            self.loaded_urls.push(url.to_owned());
            Ok(())
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
}
