// MoltenCorrQA 专家盲评 — Tauri v2 薄壳客户端
// 主窗口直接加载远程 Arena（http://47.101.156.148:8200/），前端更新走服务端；
// 客户端壳更新走 GitHub Release：远程页面没有 IPC 权限，所以整个
// 检查更新 → 弹窗确认 → 下载安装 → 重启 流程全部在 Rust 侧完成。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let handle = app.handle().clone();
            // 启动后异步静默检查更新，不阻塞窗口加载
            tauri::async_runtime::spawn(async move {
                if let Err(e) = check_update(handle).await {
                    // 检查失败（离线、GitHub 不可达等）静默忽略，不打扰评审
                    eprintln!("[updater] check failed: {e}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn check_update(app: AppHandle) -> tauri_plugin_updater::Result<()> {
    let Some(update) = app.updater()?.check().await? else {
        return Ok(()); // 已是最新版
    };

    let msg = format!(
        "检测到新版本 v{}（当前 v{}）。\n\n是否现在下载并安装？安装完成后应用将自动重启。",
        update.version, update.current_version
    );

    let app_for_install = app.clone();
    app.dialog()
        .message(msg)
        .title("MoltenCorrQA 专家评审 — 发现新版本")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "立即更新".to_string(),
            "以后再说".to_string(),
        ))
        .show(move |confirmed| {
            if !confirmed {
                return;
            }
            tauri::async_runtime::spawn(async move {
                match update.download_and_install(|_chunk, _total| {}, || {}).await {
                    Ok(()) => {
                        // Windows/NSIS passive 安装时应用通常已被安装器结束；
                        // 若仍存活则主动重启到新版本。
                        app_for_install.restart();
                    }
                    Err(e) => {
                        eprintln!("[updater] install failed: {e}");
                        app_for_install
                            .dialog()
                            .message(format!(
                                "更新下载或安装失败：{e}\n\n可稍后重启应用重试，或到 GitHub Release 手动下载最新安装包。"
                            ))
                            .title("更新失败")
                            .kind(MessageDialogKind::Error)
                            .buttons(MessageDialogButtons::Ok)
                            .show(|_| {});
                    }
                }
            });
        });

    Ok(())
}
