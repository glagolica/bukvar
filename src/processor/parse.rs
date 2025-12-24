//! File parsing utilities.

use crate::ast::{Document, DocumentType};
use crate::cli::Args;
use crate::markdown::MarkdownParser;
use crate::parsers::{JavaDocParser, JsDocParser, PyDocParser};
use crate::sourcemap::SourceMap;
use crate::streaming;
use crate::validate;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::write;

/// Parse a single file and write output.
pub fn process_single_file(file_path: &Path, args: &Args) -> Result<(DocumentType, usize), String> {
  let doc_type = detect_doc_type(file_path)?;
  let mut doc = parse_file(file_path, doc_type, args)?;

  doc.source_path = normalize_path(file_path);
  let node_count = doc.metadata.total_nodes;

  run_validation_if_enabled(&doc, file_path, args);
  write_sourcemap_if_enabled(&doc, file_path, args)?;
  write::write_output(&doc, file_path, args)?;

  Ok((doc_type, node_count))
}

/// Normalize path separators to forward slashes.
fn normalize_path(path: &Path) -> String {
  path.to_string_lossy().replace('\\', "/")
}

fn detect_doc_type(file_path: &Path) -> Result<DocumentType, String> {
  let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
  DocumentType::from_extension(extension).ok_or_else(|| {
    format!(
      "Unknown file extension: {} in {}",
      extension,
      file_path.display()
    )
  })
}

fn parse_file(file_path: &Path, doc_type: DocumentType, args: &Args) -> Result<Document, String> {
  match (args.streaming, doc_type) {
    (true, DocumentType::Markdown) => parse_streaming(file_path),
    _ => parse_normal(file_path, doc_type),
  }
}

fn parse_streaming(file_path: &Path) -> Result<Document, String> {
  let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
  Ok(streaming::parse_streaming(file))
}

fn parse_normal(file_path: &Path, doc_type: DocumentType) -> Result<Document, String> {
  let content = read_file_content(file_path)?;

  Ok(match doc_type {
    DocumentType::Markdown => MarkdownParser::new(&content).parse(),
    DocumentType::JavaScript | DocumentType::TypeScript => {
      let mut doc = JsDocParser::new(&content).parse();
      doc.doc_type = doc_type;
      doc
    }
    DocumentType::Java => JavaDocParser::new(&content).parse(),
    DocumentType::Python => PyDocParser::new(&content).parse(),
  })
}

fn read_file_content(file_path: &Path) -> Result<String, String> {
  let mut file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
  let mut content = String::new();
  file
    .read_to_string(&mut content)
    .map_err(|e| format!("Failed to read file: {}", e))?;
  Ok(content)
}

fn run_validation_if_enabled(doc: &Document, file_path: &Path, args: &Args) {
  if !args.validate {
    return;
  }

  let result = validate::validate(doc);

  if !result.is_ok() {
    eprintln!("Validation errors in {}:", file_path.display());
    result
      .errors
      .iter()
      .for_each(|e| eprintln!("  [ERROR] {} at line {}", e.message, e.line));
  }

  if result.has_warnings() {
    eprintln!("Validation warnings in {}:", file_path.display());
    result
      .warnings
      .iter()
      .for_each(|w| eprintln!("  [WARN] {} at line {}", w.message, w.line));
  }
}

fn write_sourcemap_if_enabled(doc: &Document, file_path: &Path, args: &Args) -> Result<(), String> {
  if !args.sourcemap {
    return Ok(());
  }

  let map = SourceMap::from_document(doc);
  let json = map.to_json();

  let file_name = file_path
    .file_name()
    .and_then(|s| s.to_str())
    .unwrap_or("output");
  let map_path = args.output.join(format!("{}.map.json", file_name));

  std::fs::write(&map_path, json).map_err(|e| format!("Failed to write sourcemap: {}", e))
}
