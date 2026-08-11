//! Tauri composition root for the M2 desktop shell.

use voice_application as _;

/// Starts the desktop shell without registering product commands or adapters.
///
/// # Panics
///
/// Panics when Tauri cannot initialize or run the desktop event loop.
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run the Voxora desktop shell");
}
