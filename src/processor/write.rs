//! Output writing utilities.

use crate::ast::Document;
use crate::cli::{Args, OutputFormat};
use crate::formats::{to_json, to_json_pretty, write_dast};

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Write document output to file.
pub fn write_output(doc: &Document, file_path: &Path, args: &Args) -> Result<(), String> {
  let output_path = compute_output_path(file_path, args);
  ensure_parent_dir(&output_path)?;
  write_content(&output_path, doc, args)
}

fn compute_output_path(file_path: &Path, args: &Args) -> std::path::PathBuf {
  let file_name = file_path
    .file_name()
    .and_then(|s| s.to_str())
    .unwrap_or("output");
  let extension = match args.format {
    OutputFormat::Json => "json",
    OutputFormat::Dast => "dast",
  };
  args.output.join(format!("{}.{}", file_name, extension))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
  path
    .parent()
    .map(|p| fs::create_dir_all(p).map_err(|e| format!("Failed to create output directory: {}", e)))
    .transpose()
    .map(|_| ())
}

fn write_content(path: &Path, doc: &Document, args: &Args) -> Result<(), String> {
  match args.format {
    OutputFormat::Json => write_json(path, doc, args.pretty),
    OutputFormat::Dast => write_binary(path, doc),
  }
}

fn write_json(path: &Path, doc: &Document, pretty: bool) -> Result<(), String> {
  let content = if pretty {
    to_json_pretty(doc)
  } else {
    to_json(doc)
  };
  write_string_to_file(path, &content)
}

fn write_binary(path: &Path, doc: &Document) -> Result<(), String> {
  let data = write_dast(doc).map_err(|e| format!("Failed to serialize DAST: {}", e))?;
  let mut file = File::create(path).map_err(|e| format!("Failed to create output file: {}", e))?;
  file
    .write_all(&data)
    .map_err(|e| format!("Failed to write output: {}", e))
}

fn write_string_to_file(path: &Path, content: &str) -> Result<(), String> {
  let mut file = File::create(path).map_err(|e| format!("Failed to create output file: {}", e))?;
  file
    .write_all(content.as_bytes())
    .map_err(|e| format!("Failed to write output: {}", e))
}
