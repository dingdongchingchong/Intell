//! CaseFlow CMS desktop shell — embeds the web UI and manages the API process.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tauri::{AppHandle, Manager, RunEvent};

const BACKEND_PORT: u16 = 8080;

struct BackendProcess {
    child: Mutex<Option<Child>>,
}

impl BackendProcess {
    fn new() -> Self {
        Self {
            child: Mutex::new(None),
        }
    }

    fn stop(&self) {
        let mut guard = self.child.lock();
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
}

fn backend_dir(app: &AppHandle) -> PathBuf {
    let dev = project_root().join("backend");
    if dev.join(".env").exists() || dev.join("Cargo.toml").exists() {
        return dev;
    }

    if let Ok(resource) = app.path().resource_dir() {
        let bundled = resource.join("backend");
        if bundled.exists() {
            return bundled;
        }
        return resource;
    }

    dev
}

fn backend_binary(app: &AppHandle) -> Result<PathBuf, String> {
    let root = project_root();
    let triple = env!("TARGET");
    let candidates = [
        root.join("backend/target/debug/caseflow_cms"),
        root.join("backend/target/release/caseflow_cms"),
        root.join(format!("src-tauri/binaries/caseflow-backend-{triple}")),
    ];
    for path in &candidates {
        if path.is_file() {
            return Ok(path.clone());
        }
    }

    if let Ok(exe_dir) = app.path().executable_dir() {
        let packaged = [
            exe_dir.join("caseflow-backend"),
            exe_dir.join(format!("caseflow-backend-{triple}")),
            exe_dir.join("caseflow_cms"),
        ];
        for path in &packaged {
            if path.is_file() {
                return Ok(path.clone());
            }
        }
    }

    Err("Could not find caseflow backend binary. Run: npm run prepare:sidecar".into())
}

fn desktop_cors_origins(backend: &Path) -> String {
    let mut origins: Vec<String> = vec![
        "http://127.0.0.1:1420".into(),
        "http://localhost:1420".into(),
        "http://127.0.0.1:3000".into(),
        "http://localhost:3000".into(),
        "http://127.0.0.1:5500".into(),
        "http://localhost:5500".into(),
        "http://tauri.localhost".into(),
        "https://tauri.localhost".into(),
        "tauri://localhost".into(),
        "http://ipc.localhost".into(),
        "https://ipc.localhost".into(),
        "null".into(),
    ];

    let env_path = backend.join(".env");
    if let Ok(text) = std::fs::read_to_string(env_path) {
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("CORS_ORIGINS=") {
                let rest = rest.trim().trim_matches('"').trim_matches('\'');
                for part in rest.split(',') {
                    let o = part.trim();
                    if !o.is_empty() && !origins.iter().any(|x| x == o) {
                        origins.push(o.to_string());
                    }
                }
            }
        }
    }

    origins.join(",")
}

fn port_open(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().unwrap(),
        Duration::from_millis(200),
    )
    .is_ok()
}

fn wait_for_backend(timeout: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if port_open(BACKEND_PORT) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

fn start_backend(app: &AppHandle, state: &BackendProcess) -> Result<(), String> {
    if port_open(BACKEND_PORT) {
        eprintln!("[caseflow] backend already listening on :{BACKEND_PORT}");
        return Ok(());
    }

    let binary = backend_binary(app)?;
    let cwd = backend_dir(app);
    let cors = desktop_cors_origins(&cwd);

    eprintln!(
        "[caseflow] starting backend {} (cwd={})",
        binary.display(),
        cwd.display()
    );

    let child = Command::new(&binary)
        .current_dir(&cwd)
        .env("APP_HOST", "127.0.0.1")
        .env("APP_PORT", BACKEND_PORT.to_string())
        .env("CORS_ORIGINS", &cors)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Failed to spawn backend {}: {e}", binary.display()))?;

    *state.child.lock() = Some(child);

    if !wait_for_backend(Duration::from_secs(25)) {
        state.stop();
        return Err(format!(
            "Backend did not become ready on 127.0.0.1:{BACKEND_PORT} within 25s"
        ));
    }

    eprintln!("[caseflow] backend ready on http://127.0.0.1:{BACKEND_PORT}");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let backend_state = Arc::new(BackendProcess::new());
    let backend_for_setup = Arc::clone(&backend_state);
    let backend_for_exit = Arc::clone(&backend_state);

    let app = tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            if let Err(err) = start_backend(&handle, &backend_for_setup) {
                eprintln!("[caseflow] {err}");
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building CaseFlow desktop app");

    app.run(move |_app_handle, event| {
        if let RunEvent::Exit = event {
            backend_for_exit.stop();
        }
    });
}
