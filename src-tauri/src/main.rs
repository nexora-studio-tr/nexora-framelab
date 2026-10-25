#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use framelab_core::{model::*, Engine};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

type Backend = Arc<Engine>;
#[tauri::command]
fn snapshot(engine: tauri::State<'_, Backend>) -> Result<Snapshot> {
    engine.snapshot()
}
#[tauri::command]
async fn refresh_system(engine: tauri::State<'_, Backend>) -> Result<Snapshot> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || engine.refresh())
        .await
        .map_err(|_| "SYSTEM_QUERY_FAILED")?
}
#[tauri::command]
async fn choose_replay(app: tauri::AppHandle, engine: tauri::State<'_, Backend>) -> Result<bool> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = app
            .dialog()
            .file()
            .add_filter("CS2 replay", &["dem"])
            .blocking_pick_file()
        else {
            return Ok(false);
        };
        let path = file.into_path().map_err(|_| "REPLAY_PATH_INVALID")?;
        engine.select_replay(&path)?;
        Ok(true)
    })
    .await
    .map_err(|_| "DIALOG_FAILED")?
}
#[tauri::command]
async fn import_capture(
    app: tauri::AppHandle,
    engine: tauri::State<'_, Backend>,
) -> Result<Option<Report>> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = app
            .dialog()
            .file()
            .add_filter("PresentMon CSV", &["csv"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = file.into_path().map_err(|_| "CSV_INVALID")?;
        engine.analyze(&path).map(Some)
    })
    .await
    .map_err(|_| "DIALOG_FAILED")?
}
#[tauri::command]
async fn start_experiment(
    engine: tauri::State<'_, Backend>,
    request: StartRequest,
) -> Result<String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || engine.start(request))
        .await
        .map_err(|_| "EXPERIMENT_START_FAILED")?
}
#[tauri::command]
fn cancel_experiment(engine: tauri::State<'_, Backend>) -> Result<()> {
    engine.cancel()
}
#[tauri::command]
async fn restore_profile(engine: tauri::State<'_, Backend>, report_id: String) -> Result<()> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || engine.restore(&report_id))
        .await
        .map_err(|_| "RESTORE_FAILED")?
}
#[tauri::command]
async fn export_report(
    app: tauri::AppHandle,
    engine: tauri::State<'_, Backend>,
    report_id: String,
) -> Result<bool> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = app
            .dialog()
            .file()
            .add_filter("FrameLab report", &["json"])
            .set_file_name("framelab-report.json")
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let path = file.into_path().map_err(|_| "EXPORT_PATH_INVALID")?;
        engine.export(&report_id, &path)?;
        Ok(true)
    })
    .await
    .map_err(|_| "DIALOG_FAILED")?
}
#[tauri::command]
fn load_example(engine: tauri::State<'_, Backend>) -> Result<Report> {
    engine.example()
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data = app.path().app_local_data_dir()?;
            let resources = app.path().resource_dir()?;
            let engine = Engine::open(data, resources.join("presentmon/PresentMon.exe"))
                .map_err(std::io::Error::other)?;
            app.manage(engine);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            refresh_system,
            choose_replay,
            import_capture,
            start_experiment,
            cancel_experiment,
            restore_profile,
            export_report,
            load_example
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let engine = window.state::<Backend>();
                if engine.busy() {
                    api.prevent_close();
                    let _ = engine.cancel();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("FrameLab initialization failed");
}
