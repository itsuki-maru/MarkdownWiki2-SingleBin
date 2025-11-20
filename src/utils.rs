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

// Windowsのみ有効
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