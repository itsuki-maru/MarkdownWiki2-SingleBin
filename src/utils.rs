use std::path::Path;
use tokio::fs;
use tokio::io;

pub async fn ensure_dir(path: &Path) -> io::Result<()> {
    match fs::create_dir_all(path).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e),
    }
}

// コンソール表示処理（Windowsのみ有効）
#[cfg(windows)]
pub fn ensure_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole};

    // 開発時のみ
    if !cfg!(debug_assertions) {
        unsafe {
            // 親プロセスのコンソールへアタッチ
            if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
                AllocConsole().unwrap();
            }
        }
    }
}

// コンソール非表示処理（Windowsのみ有効）
#[cfg(windows)]
pub fn hide_console() {
    use windows::Win32::{
        System::Console::GetConsoleWindow,
        UI::WindowsAndMessaging::{SW_HIDE, ShowWindow},
    };

    unsafe {
        let hwnd = GetConsoleWindow();
        // console window がない場合
        if hwnd.is_invalid() {
            return;
        }
        // 非表示処理
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}
