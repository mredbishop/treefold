use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct FsEntry {
    pub path: PathBuf,
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    pub children: Vec<FsEntry>,
    pub errors: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("path does not exist: {0}")]
    NotFound(PathBuf),
    #[error("failed to read metadata for {path}: {source}")]
    Metadata {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub fn scan_path(path: &Path) -> Result<FsEntry, ScanError> {
    if !path.exists() {
        return Err(ScanError::NotFound(path.to_path_buf()));
    }
    scan_node(path, &mut |_| {})
}

pub fn scan_path_with_progress<F>(path: &Path, progress: &mut F) -> Result<FsEntry, ScanError>
where
    F: FnMut(&Path),
{
    if !path.exists() {
        return Err(ScanError::NotFound(path.to_path_buf()));
    }
    scan_node(path, progress)
}

fn scan_node<F>(path: &Path, progress: &mut F) -> Result<FsEntry, ScanError>
where
    F: FnMut(&Path),
{
    let metadata = fs::symlink_metadata(path).map_err(|source| ScanError::Metadata {
        path: path.to_path_buf(),
        source,
    })?;

    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    if metadata.is_file() || metadata.file_type().is_symlink() {
        return Ok(FsEntry {
            path: path.to_path_buf(),
            name,
            kind: EntryKind::File,
            size: metadata.len(),
            children: Vec::new(),
            errors: Vec::new(),
        });
    }

    let mut errors = Vec::new();
    let mut children = Vec::new();
    progress(path);

    let read_dir = fs::read_dir(path);
    match read_dir {
        Ok(entries) => {
            for entry_result in entries {
                match entry_result {
                    Ok(entry) => match scan_node(&entry.path(), progress) {
                        Ok(node) => children.push(node),
                        Err(err) => errors.push(err.to_string()),
                    },
                    Err(err) => errors.push(format!("failed to read directory entry: {err}")),
                }
            }
        }
        Err(err) => errors.push(format!(
            "failed to read directory {}: {err}",
            path.display()
        )),
    }

    children.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
    let size = children.iter().map(|c| c.size).sum();

    Ok(FsEntry {
        path: path.to_path_buf(),
        name,
        kind: EntryKind::Directory,
        size,
        children,
        errors,
    })
}

pub fn count_errors(entry: &FsEntry) -> usize {
    entry.errors.len() + entry.children.iter().map(count_errors).sum::<usize>()
}
