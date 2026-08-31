// 静待 QuietDo —— Tauri 后端
// 所有原生能力（本地 JSON 读写、窗口拖动/关闭、开机自启）通过自定义命令暴露给前端。

use std::fs;
use std::path::PathBuf;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindow,
};
use tauri_plugin_autostart::ManagerExt;

/// 显示并聚焦主窗口。
fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

/// 切换主窗口显示/隐藏。
fn toggle_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.unminimize();
            let _ = win.set_focus();
        }
    }
}

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

/// 隐藏窗口到系统托盘（不退出）。
#[tauri::command]
fn hide_window(window: WebviewWindow) {
    let _ = window.hide();
}

/// 设置窗口是否始终置顶。
#[tauri::command]
fn set_always_on_top(window: WebviewWindow, enabled: bool) -> Result<(), String> {
    window.set_always_on_top(enabled).map_err(|e| e.to_string())
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
            hide_window,
            set_always_on_top,
            set_autostart,
            get_autostart
        ])
        .setup(|app| {
            // ===== 系统托盘 =====
            let show_hide = MenuItemBuilder::with_id("show_hide", "显示 / 隐藏静待").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_hide)
                .separator()
                .item(&quit)
                .build()?;

            let mut builder = TrayIconBuilder::with_id("main-tray")
                .tooltip("静待 QuietDo")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show_hide" => toggle_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键单击（抬起时）显示窗口
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                });

            // 使用应用默认图标作为托盘图标
            if let Some(icon) = app.default_window_icon() {
                builder = builder.icon(icon.clone());
            }
            builder.build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
