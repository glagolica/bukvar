//! File collection utilities.

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

/// Collect files matching extensions from directory.
pub fn collect_files(
  dir: &Path,
  extensions: &[String],
  recursive: bool,
) -> Result<Vec<PathBuf>, String> {
  let mut files = Vec::new();
  let mut queue = VecDeque::new();
  queue.push_back(dir.to_path_buf());

  while let Some(current_dir) = queue.pop_front() {
    let entries = fs::read_dir(&current_dir)
      .map_err(|e| format!("Failed to read directory {}: {}", current_dir.display(), e))?;

    for entry in entries.flatten() {
      let path = entry.path();

      if path.is_dir() {
        if recursive && !should_skip_dir(&path) {
          queue.push_back(path);
        }
      } else if path.is_file() && matches_extension(&path, extensions) {
        files.push(path);
      }
    }
  }

  Ok(files)
}

fn matches_extension(path: &Path, extensions: &[String]) -> bool {
  path
    .extension()
    .and_then(|e| e.to_str())
    .map(|ext| extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)))
    .unwrap_or(false)
}

fn should_skip_dir(path: &Path) -> bool {
  path
    .file_name()
    .and_then(|n| n.to_str())
    .map(is_ignored_dir)
    .unwrap_or(false)
}

fn is_ignored_dir(name: &str) -> bool {
  const IGNORED: &[&str] = &[
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    "target",
    "build",
    "dist",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    "venv",
    ".venv",
    ".idea",
    ".vscode",
    "vendor",
    "packages",
    ".next",
    ".nuxt",
  ];
  IGNORED.contains(&name)
}
