use crate::ResultExt;
use async_fs as fs;
use futures_lite::StreamExt;
use std::{
    io,
    path::{Path, PathBuf},
};

/// Removes all files and directories matching the given predicate
pub async fn remove_matching<F>(dir: &Path, predicate: F)
where
    F: Fn(&Path) -> bool,
{
    if let Some(mut entries) = fs::read_dir(dir).await.log_err() {
        while let Some(entry) = entries.next().await {
            if let Some(entry) = entry.log_err() {
                let entry_path = entry.path();
                if predicate(entry_path.as_path())
                    && let Ok(metadata) = fs::metadata(&entry_path).await
                {
                    if metadata.is_file() {
                        fs::remove_file(&entry_path).await.log_err();
                    } else {
                        fs::remove_dir_all(&entry_path).await.log_err();
                    }
                }
            }
        }
    }
}

pub async fn find_file_name_in_dir<F>(dir: &Path, predicate: F) -> Option<PathBuf>
where
    F: Fn(&str) -> bool,
{
    if let Some(mut entries) = fs::read_dir(dir).await.log_err() {
        while let Some(entry) = entries.next().await {
            if let Some(entry) = entry.log_err() {
                let entry_path = entry.path();

                if let Some(file_name) = entry_path
                    .file_name()
                    .map(|file_name| file_name.to_string_lossy())
                    && predicate(&file_name)
                {
                    return Some(entry_path);
                }
            }
        }
    }

    None
}

#[cfg(unix)]
/// Set the permissions for the given path so that the file becomes executable.
/// This is a noop for non-unix platforms.
pub async fn make_file_executable(path: &Path) -> std::io::Result<()> {
    fs::set_permissions(
        path,
        <fs::Permissions as fs::unix::PermissionsExt>::from_mode(0o755),
    )
    .await
}

#[cfg(not(unix))]
#[allow(clippy::unused_async)]
/// Set the permissions for the given path so that the file becomes executable.
/// This is a noop for non-unix platforms.
pub async fn make_file_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

pub fn read_skipping_special_files(path: &Path) -> io::Result<Vec<u8>> {
    if is_special_file(path) {
        return Ok(Vec::new());
    }
    std::fs::read(path)
}

pub fn read_to_string_skipping_special_files(path: &Path) -> io::Result<String> {
    if is_special_file(path) {
        return Ok(String::new());
    }
    std::fs::read_to_string(path)
}

#[cfg(unix)]
pub fn is_special_file(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|metadata| !metadata.is_file() && !metadata.is_dir())
}

#[cfg(not(unix))]
pub fn is_special_file(_path: &Path) -> bool {
    false
}
