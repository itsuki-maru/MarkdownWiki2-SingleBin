use std::collections::HashMap;
use std::hash::Hash;
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

pub fn vec_to_hashmap<K, T, F>(vec: Vec<T>, key_fn: F) -> HashMap<K, T>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    vec.into_iter().map(|item| (key_fn(&item), item)).collect()
}

/// サーバーモード起動時にコンソールを確保する（Windows リリースビルド向け）。
/// リリースビルドでは windows_subsystem="windows" によりコンソールが非表示になるため、
/// 親プロセスのコンソールへのアタッチを試み、失敗した場合は新規割り当てを行う。
#[cfg(windows)]
pub fn ensure_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole};
    if !cfg!(debug_assertions) {
        unsafe {
            if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
                AllocConsole().unwrap();
            }
        }
    }
}
