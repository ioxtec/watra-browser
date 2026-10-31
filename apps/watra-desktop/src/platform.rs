#[cfg(target_os = "macos")]
pub(crate) mod macos;

#[cfg(target_os = "macos")]
pub(crate) use macos::{cef_application, cef_pump, window_chrome};

#[cfg(target_os = "macos")]
pub(crate) fn run() {
    macos::application::run();
}

#[cfg(not(target_os = "macos"))]
compile_error!("Watra's Windows and Linux desktop hosts are not implemented yet");
