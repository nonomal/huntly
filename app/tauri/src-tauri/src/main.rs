// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use reqwest::StatusCode;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{exit, Child, Command};
use std::sync::Mutex;
use tauri::{
    command, AppHandle, CustomMenuItem, Menu, MenuItem, Submenu, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem
};
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

#[macro_use]
extern crate lazy_static;

const CREATE_NO_WINDOW: u32 = 0x08000000;

lazy_static! {
    static ref SPRING_BOOT_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Settings {
    port: u16,
    auto_start_up: bool,
}

fn get_app_data_dir(app: &AppHandle) -> String {
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    if !app_dir.exists() {
        std::fs::create_dir(app_dir).unwrap();
    }
    return app
        .path_resolver()
        .app_data_dir()
        .unwrap()
        .as_os_str()
        .to_os_string()
        .into_string()
        .unwrap();
}

fn get_settings_path(app: &AppHandle) -> String {
    let app_dir = app.path_resolver().app_config_dir().unwrap();
    if !app_dir.exists() {
        std::fs::create_dir(app_dir).unwrap();
    }
    let path = app
        .path_resolver()
        .app_config_dir()
        .unwrap()
        .join("app.settings.json");
    return path.to_str().unwrap().to_owned();
}

#[command]
fn save_settings(app: AppHandle, settings: Settings) {
    let settings_str = serde_json::to_string(&settings).unwrap();
    std::fs::write(get_settings_path(&app), settings_str).unwrap();
}

#[command]
fn read_settings(app: AppHandle) -> String {
    return std_read_settings(&app);
}

fn std_read_settings(app: &AppHandle) -> String {
    return std::fs::read_to_string(get_settings_path(app)).unwrap();
}

fn get_default_settings() -> Settings {
    Settings {
        port: 8123,
        auto_start_up: false,
    }
}

fn get_settings(app: &AppHandle) -> Settings {
    let settings_str = std_read_settings(app);
    let settings: Settings = serde_json::from_str(&settings_str).unwrap();
    settings
}

#[command]
async fn is_server_running(app: AppHandle) -> bool {
    match reqwest::get(format!(
        "http://localhost:{}/api/health",
        get_settings(&app).port
    ))
    .await
    {
        Ok(response) => response.status() == StatusCode::OK,
        Err(_) => false,
    }
}

#[command]
fn is_server_started() -> bool {
    let spring_boot_process = SPRING_BOOT_PROCESS.lock().unwrap();
    spring_boot_process.is_some()
}

#[command]
fn start_server(app: AppHandle) {
    handle_start_server(&app);
}

fn handle_start_server(app: &AppHandle) {
    // 获取 Spring Boot Jar 文件路径
    let settings: Settings = get_settings(app);
    let port = settings.port;
    let mut java_resource_path = "server_bin/jre11/bin/java.exe";
    if cfg!(not(target_os = "windows")) {
        java_resource_path = "server_bin/jre11/bin/java";
    }
    let java_path = app
        .path_resolver()
        .resolve_resource(java_resource_path)
        .unwrap();
    let file_path = app
        .path_resolver()
        .resolve_resource("server_bin/huntly-server.jar")
        .unwrap();
    println!("java path:{}", canonicalize(java_path.as_os_str()));
    println!("jar file path:{}", canonicalize(file_path.as_os_str()));
    let java_path_str = canonicalize(java_path);
    let cmd_path = java_path_str.clone();
    // #[cfg(target_os = "windows")]
    // {
    //     cmd_path = "cmd".to_owned();
    // }
    // #[cfg(target_os = "macos")]
    // {
    //     cmd_path = java_path_str;
    // }
    let mut cmd = Command::new(cmd_path);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
        // cmd.raw_arg("/C").raw_arg(java_path_str);
    }
    println!("Starting Spring Boot application... ");
    let data_dir = get_app_data_dir(app);
    println!("data dir:{}", data_dir);
    #[cfg(target_os = "windows")]
    {
        let child = cmd
            .raw_arg("-jar")
            .raw_arg(format!("\"{}\"", canonicalize(file_path)))
            .raw_arg(format!("--server.port={}", port))
            .raw_arg(format!("--huntly.dataDir=\"{}/\"", data_dir))
            .raw_arg(format!("--huntly.luceneDir=\"{}/lucene\"", data_dir))
            // .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("failed to start spring boot application");
        let mut spring_boot_process = SPRING_BOOT_PROCESS.lock().unwrap();
        *spring_boot_process = Some(child);
    }
    #[cfg(target_os = "macos")]
    {
        let child = cmd
            .arg("-jar")
            .arg(format!("{}", canonicalize(file_path)))
            .arg(format!("--server.port={}", port))
            .arg(format!("--huntly.dataDir={}/", data_dir))
            .arg(format!("--huntly.luceneDir={}/lucene", data_dir))
            // .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("failed to start spring boot application");
        let mut spring_boot_process = SPRING_BOOT_PROCESS.lock().unwrap();
        *spring_boot_process = Some(child);
    }
    println!("Spring Boot application started. ");
}

fn canonicalize<P: AsRef<Path>>(path: P) -> String {
    let str = path
        .as_ref()
        .as_os_str()
        .to_os_string()
        .into_string()
        .unwrap();
    if str.starts_with("\\\\?\\") {
        str.replace("\\\\?\\", "")
    } else {
        str
    }
}

#[command]
fn stop_server() {
    // 获取 Spring Boot 进程对象
    let mut spring_boot_process = SPRING_BOOT_PROCESS.lock().unwrap();
    let child = spring_boot_process.take();

    // 如果存在进程对象，则终止它
    if let Some(mut process) = child {
        process.kill().expect("Failed to kill Spring Boot process");
    }

    println!("Backend gracefully shutdown.");
}

fn main() {
    let mut app = tauri::Builder::default()
        .setup(|app| {
            let settings_path = get_settings_path(&app.app_handle());
            let metadata_settings = std::fs::metadata(settings_path);
            match metadata_settings {
                Ok(_) => {}
                Err(_) => {
                    let default_settings = get_default_settings();
                    let settings_str = serde_json::to_string(&default_settings).unwrap();
                    let path = get_settings_path(&app.app_handle());
                    std::fs::write(path, settings_str).unwrap();
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--silently"]),
        ))
        .invoke_handler(tauri::generate_handler![
            save_settings,
            read_settings,
            start_server,
            stop_server,
            is_server_running,
            is_server_started
        ])
        .system_tray(tray())
        .on_system_tray_event(handler)
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(ActivationPolicy::Accessory);
    }
    app.run(|app, event| match event {
        RunEvent::Exit { .. } => {
            stop_server();
        }
        RunEvent::ExitRequested { api, .. } => {
            stop_server();
        }
        RunEvent::WindowEvent { label, event, .. } => {
            if label == "main" {
                match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        let window = app.get_window("main").unwrap();
                        window.hide().unwrap();
                        api.prevent_close();
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    })
}

fn tray() -> SystemTray {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("config", "Config"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("open", "Open huntly"))
        .add_item(CustomMenuItem::new("open_dir", "Open data directory"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("start", "Start server"))
        .add_item(CustomMenuItem::new("stop", "Stop server"))
        .add_item(CustomMenuItem::new("restart", "Restart server"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"));
    SystemTray::new().with_menu(tray_menu)
}

#[cfg(target_os = "windows")]
fn open_browser(url: &str) {
    let _ = Command::new("cmd")
        .creation_flags(CREATE_NO_WINDOW)
        .args(&["/C", "start"])
        .arg(url)
        .spawn();
}

#[cfg(target_os = "macos")]
fn open_browser(url: &str) {
    let _ = Command::new("open").arg(url).spawn();
}

fn handler(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick {
            tray_id,
            position,
            size,
            ..
        } => {
            #[cfg(target_os = "windows")]
            {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "config" => {
                let window = app.get_window("main").unwrap();
                window.set_focus().unwrap();
                window.unminimize().unwrap();
                window.show().unwrap();
            }
            "open" => {
                // open huntly server in web browser
                let settings: Settings = get_settings(app);
                let port = settings.port;
                let url = format!("http://localhost:{}", port);
                open_browser(url.as_str());
            }
            "open_dir" => {
                #[cfg(target_os = "windows")]
                {
                    let data_dir = get_app_data_dir(app);
                    println!("data_dir: {}", data_dir);
                    Command::new("explorer")
                        .creation_flags(CREATE_NO_WINDOW)
                        .raw_arg(&format!("/open,\"{}\"", data_dir))
                        .spawn()
                        .unwrap();
                }
                #[cfg(target_os = "macos")]
                {
                    let data_dir = get_app_data_dir(app);
                    Command::new("open")
                        .args(&["-R", &data_dir])
                        .spawn()
                        .unwrap();
                }
            }
            "restart" => {
                stop_server();
                handle_start_server(app);
            }
            "start" => {
                stop_server();
                handle_start_server(app);
            }
            "stop" => {
                stop_server();
            }
            "quit" => {
                stop_server();
                app.exit(0);
            }
            _ => {}
        },
        _ => {}
    }
}
