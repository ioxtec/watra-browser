use std::convert::Infallible;

use watra_core::BrowserCore;
use watra_engine_api::{BrowserEngine, NavigationRequest};

struct PlaceholderEngine;

impl BrowserEngine for PlaceholderEngine {
    type Error = Infallible;

    fn navigate(&mut self, _request: &NavigationRequest) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    let core = BrowserCore::new(PlaceholderEngine);
    println!(
        "Watra Browser workspace is ready (current URL: {}).",
        core.current_url().unwrap_or("none")
    );
}
