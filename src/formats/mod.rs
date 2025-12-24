//! Output formats: DAST (binary) and JSON

mod json;
mod reader;
mod writer;

pub use json::{to_json, to_json_pretty};
pub use reader::DastReader;
pub use writer::DastWriter;

use crate::ast::Document;
use std::io;

/// Magic bytes for DAST format identification.
pub const MAGIC: &[u8; 4] = b"DAST";
/// Current format version.
pub const VERSION: u8 = 1;

/// Write document to DAST binary format.
pub fn write_dast(doc: &Document) -> io::Result<Vec<u8>> {
  let mut writer = DastWriter::new();
  let mut buf = Vec::new();
  writer.write(doc, &mut buf)?;
  Ok(buf)
}

/// Read document from DAST binary format.
#[allow(dead_code)]
pub fn read_dast(data: &[u8]) -> io::Result<Document> {
  let mut reader = DastReader::new();
  let mut cursor = std::io::Cursor::new(data);
  reader.read(&mut cursor)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::*;

  fn test_doc() -> Document {
    Document {
      source_path: "test.md".to_string(),
      doc_type: DocumentType::Markdown,
      nodes: vec![
        Node::new(NodeKind::Paragraph, Span::new(0, 10, 1, 1)),
        Node::with_children(
          NodeKind::Heading {
            level: 1,
            id: Some("title".to_string()),
          },
          Span::new(11, 20, 2, 1),
          vec![Node::new(
            NodeKind::Text {
              content: "Hello".to_string(),
            },
            Span::new(13, 18, 2, 3),
          )],
        ),
      ],
      metadata: DocumentMetadata {
        title: Some("Test Doc".to_string()),
        description: Some("A test document".to_string()),
        total_lines: 5,
        total_nodes: 3,
      },
    }
  }

  #[test]
  fn test_magic_bytes() {
    assert_eq!(MAGIC, b"DAST");
    assert_eq!(VERSION, 1);
  }

  #[test]
  fn test_write_dast_basic() {
    let doc = test_doc();
    let result = write_dast(&doc);
    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], MAGIC);
    assert_eq!(bytes[4], VERSION);
  }

  #[test]
  fn test_roundtrip_dast() {
    let doc = test_doc();
    let bytes = write_dast(&doc).unwrap();
    let restored = read_dast(&bytes).unwrap();

    assert_eq!(restored.source_path, doc.source_path);
    assert_eq!(restored.doc_type, doc.doc_type);
    assert_eq!(restored.nodes.len(), doc.nodes.len());
    assert_eq!(restored.metadata.title, doc.metadata.title);
    assert_eq!(restored.metadata.total_lines, doc.metadata.total_lines);
  }

  #[test]
  fn test_roundtrip_empty_doc() {
    let doc = Document {
      source_path: "".to_string(),
      doc_type: DocumentType::Markdown,
      nodes: vec![],
      metadata: DocumentMetadata::default(),
    };
    let bytes = write_dast(&doc).unwrap();
    let restored = read_dast(&bytes).unwrap();
    assert!(restored.nodes.is_empty());
  }

  #[test]
  fn test_roundtrip_complex_nodes() {
    let doc = Document {
      source_path: "complex.md".to_string(),
      doc_type: DocumentType::Markdown,
      nodes: vec![
        Node::new(NodeKind::ThematicBreak, Span::empty()),
        Node::new(
          NodeKind::CodeBlock {
            language: Some("rust".to_string()),
            info: Some("example".to_string()),
          },
          Span::empty(),
        ),
        Node::new(
          NodeKind::List {
            ordered: true,
            start: Some(1),
            tight: true,
          },
          Span::empty(),
        ),
      ],
      metadata: DocumentMetadata::default(),
    };
    let bytes = write_dast(&doc).unwrap();
    let restored = read_dast(&bytes).unwrap();
    assert_eq!(restored.nodes.len(), 3);
  }

  #[test]
  fn test_read_invalid_magic() {
    let invalid = b"XXXX\x01\x00";
    let result = read_dast(invalid);
    assert!(result.is_err());
  }

  #[test]
  fn test_json_output() {
    let doc = test_doc();
    let json = to_json(&doc);
    assert!(json.starts_with('{'));
    assert!(json.ends_with('}'));
    assert!(json.contains("test.md"));
    assert!(json.contains("Markdown"));
  }

  #[test]
  fn test_json_pretty_output() {
    let doc = test_doc();
    let json = to_json_pretty(&doc);
    assert!(json.contains('\n'));
    let lines: Vec<&str> = json.lines().collect();
    assert!(lines.len() > 1);
  }
}
