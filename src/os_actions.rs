use std::path::Path;
use std::process::Command;

pub fn open_in_file_browser(path: &Path) -> Result<(), String> {
    let (cmd, args) = open_command(path);
    Command::new(cmd)
        .args(args)
        .spawn()
        .map_err(|e| format!("failed to open file browser: {e}"))?;
    Ok(())
}

pub fn delete_path(path: &Path, is_dir: bool) -> Result<(), String> {
    if is_dir {
        std::fs::remove_dir_all(path).map_err(|e| format!("failed to delete directory: {e}"))
    } else {
        std::fs::remove_file(path).map_err(|e| format!("failed to delete file: {e}"))
    }
}

pub fn open_command(path: &Path) -> (&'static str, Vec<String>) {
    #[cfg(target_os = "macos")]
    {
        ("open", vec![path.display().to_string()])
    }
    #[cfg(target_os = "windows")]
    {
        (
            "explorer",
            vec![path.display().to_string().replace('/', "\\")],
        )
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        ("xdg-open", vec![path.display().to_string()])
    }
}
