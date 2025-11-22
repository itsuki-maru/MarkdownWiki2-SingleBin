use tokio::fs;
use tokio::io;
use std::path::Path;

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
    use windows::Win32::System::Console::{
        AttachConsole, AllocConsole, ATTACH_PARENT_PROCESS,
    };

    unsafe {
        // 親プロセスのコンソールへアタッチ
        if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
            AllocConsole().unwrap();
        }
    }
}


// コンソール非表示処理（Windowsのみ有効）
#[cfg(windows)]
pub fn hide_console() {
    use windows::Win32::{
        System::Console::GetConsoleWindow,
        UI::WindowsAndMessaging::{ShowWindow, SW_HIDE},
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