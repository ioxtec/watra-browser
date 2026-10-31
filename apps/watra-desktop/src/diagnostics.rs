use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

pub(crate) fn initialize() -> io::Result<PathBuf> {
    let directory = log_directory();
    fs::create_dir_all(&directory)?;
    let path = directory.join("watra.log");
    let reset_log = fs::metadata(&path)
        .map(|metadata| metadata.len() > 2 * 1024 * 1024)
        .unwrap_or(false);
    let file = if reset_log {
        File::create(&path)?
    } else {
        OpenOptions::new().create(true).append(true).open(&path)?
    };
    let _ = LOG_FILE.set(Mutex::new(file));

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_else(|| "unknown".to_string());
        event(&format!("panic location={location}"));
        previous_hook(info);
    }));

    event("diagnostics.initialized");
    Ok(path)
}

pub(crate) fn event(message: &str) {
    let Some(file) = LOG_FILE.get() else {
        return;
    };
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    if let Ok(mut file) = file.lock() {
        let _ = writeln!(
            file,
            "{}.{} {}",
            elapsed.as_secs(),
            elapsed.subsec_millis(),
            message
        );
        let _ = file.flush();
    }
}

fn log_directory() -> PathBuf {
    #[cfg(target_os = "macos")]
    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join("Library/Logs/Watra");
    }

    #[cfg(target_os = "windows")]
    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("Watra/Logs");
    }

    #[cfg(target_os = "linux")]
    if let Some(state_home) = env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(state_home).join("watra");
    }

    if let Some(home) = env::var_os("HOME") {
        #[cfg(target_os = "linux")]
        return PathBuf::from(home).join(".local/state/watra");

        #[cfg(not(target_os = "linux"))]
        return PathBuf::from(home).join(".watra/logs");
    }

    env::temp_dir().join("watra-logs")
}
