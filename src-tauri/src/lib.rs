// 静待 QuietDo —— Tauri 后端
// 所有原生能力（本地 JSON 读写、窗口拖动/关闭、开机自启）通过自定义命令暴露给前端。

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;

/// 返回应用数据目录（如 %APPDATA%/com.quietdo.app/），不存在则创建。
fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法定位数据目录: {e}"))?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {e}"))?;
    }
    Ok(dir)
}

/// 读取任务列表 JSON（文件不存在时返回空数组）。
#[tauri::command]
fn load_todos(app: AppHandle) -> Result<String, String> {
    let path = data_dir(&app)?.join("todos.json");
    Ok(fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string()))
}

/// 写入任务列表 JSON。
#[tauri::command]
fn save_todos(app: AppHandle, data: String) -> Result<(), String> {
    let path = data_dir(&app)?.join("todos.json");
    fs::write(path, data).map_err(|e| format!("保存任务失败: {e}"))
}

/// 读取配置 JSON（文件不存在时返回空对象）。
#[tauri::command]
fn load_config(app: AppHandle) -> Result<String, String> {
    let path = data_dir(&app)?.join("config.json");
    Ok(fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string()))
}

/// 写入配置 JSON。
#[tauri::command]
fn save_config(app: AppHandle, data: String) -> Result<(), String> {
    let path = data_dir(&app)?.join("config.json");
    fs::write(path, data).map_err(|e| format!("保存配置失败: {e}"))
}

/// 无边框窗口拖动：由前端在标题栏 mousedown 时调用。
#[tauri::command]
fn start_drag(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

/// 无边框窗口缩放：由前端在边缘/角落手柄 mousedown 时调用。
#[tauri::command]
fn start_resize(window: tauri::Window, direction: String) -> Result<(), String> {
    use tauri_runtime::ResizeDirection;
    let dir = match direction.as_str() {
        "north" => ResizeDirection::North,
        "south" => ResizeDirection::South,
        "east" => ResizeDirection::East,
        "west" => ResizeDirection::West,
        "north-east" => ResizeDirection::NorthEast,
        "north-west" => ResizeDirection::NorthWest,
        "south-east" => ResizeDirection::SouthEast,
        "south-west" => ResizeDirection::SouthWest,
        _ => return Err(format!("未知的缩放方向: {direction}")),
    };
    window.start_resize_dragging(dir).map_err(|e| e.to_string())
}

/// 关闭并退出应用。
#[tauri::command]
fn close_app(window: WebviewWindow) {
    let _ = window.close();
}

/// 设置开机自启（注册/移除）。
#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

/// 查询当前是否已开机自启。
#[tauri::command]
fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // 自动记忆并恢复窗口位置/大小
        .plugin(tauri_plugin_window_state::Builder::default().build())
        // 开机自启（桌面平台）
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            load_todos,
            save_todos,
            load_config,
            save_config,
            start_drag,
            start_resize,
            close_app,
            set_autostart,
            get_autostart
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
