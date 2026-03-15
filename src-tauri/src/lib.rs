pub mod commands;
pub mod models;
pub mod server;
pub mod services;

use models::file::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::set_shared_dir,
            commands::get_shared_dir,
            commands::clear_shared_dir,
            commands::start_server,
            commands::stop_server,
            commands::get_server_status,
            commands::generate_qr_code,
            commands::get_server_addresses,
            commands::select_directory,
            commands::get_connected_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}