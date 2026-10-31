mod browser_session;
mod browser_shell;
mod diagnostics;
mod platform;
mod renderer_host;

#[cfg(target_os = "macos")]
pub(crate) use platform::{cef_application, cef_pump, window_chrome};

fn main() {
    match diagnostics::initialize() {
        Ok(path) => diagnostics::event(&format!("application.start log={}", path.display())),
        Err(error) => eprintln!("Failed to initialize Watra diagnostics: {error}"),
    }
    platform::run();
}
