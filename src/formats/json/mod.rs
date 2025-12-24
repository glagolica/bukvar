//! JSON output format.

mod kinds;

use crate::ast::*;

/// Convert document to compact JSON.
#[inline]
pub fn to_json(doc: &Document) -> String {
  JsonWriter::new(false).write_doc(doc)
}

/// Convert document to pretty-printed JSON.
#[inline]
pub fn to_json_pretty(doc: &Document) -> String {
  JsonWriter::new(true).write_doc(doc)
}

/// JSON writer with pre-allocated buffer.
struct JsonWriter {
  out: String,
  pretty: bool,
  depth: usize,
}

impl JsonWriter {
  /// Create a new writer with estimated capacity.
  #[inline]
  fn new(pretty: bool) -> Self {
    // Estimate ~8KB for typical documents, more for pretty
    let capacity = if pretty { 16384 } else { 8192 };
    Self {
      out: String::with_capacity(capacity),
      pretty,
      depth: 0,
    }
  }

  /// Write the complete document to JSON.
  #[inline]
  fn write_doc(mut self, doc: &Document) -> String {
    self.out.push('{');
    self.nl();
    self.depth += 1;
    self.kv_str("source_path", &doc.source_path);
    self.comma();
    self.kv_raw("doc_type", &format!("{:?}", doc.doc_type));
    self.comma();
    self.write_metadata(&doc.metadata);
    self.comma();
    self.key("nodes");
    self.write_array(&doc.nodes, |s, n| s.write_node(n));
    self.depth -= 1;
    self.nl();
    self.out.push('}');
    self.out
  }

  /// Write a single AST node.
  #[inline]
  fn write_node(&mut self, node: &Node) {
    self.out.push('{');
    self.nl();
    self.depth += 1;
    self.key("kind");
    kinds::write_kind(&mut self.out, &node.kind);
    self.comma();
    self.write_span(&node.span);
    if !node.children.is_empty() {
      self.comma();
      self.key("children");
      self.write_array(&node.children, |s, n| s.write_node(n));
    }
    self.depth -= 1;
    self.nl();
    self.out.push('}');
  }

  /// Write an array of items using the provided writer function.
  #[inline]
  fn write_array<T, F>(&mut self, items: &[T], mut writer: F)
  where
    F: FnMut(&mut Self, &T),
  {
    self.out.push('[');
    self.nl();
    self.depth += 1;
    items.iter().enumerate().for_each(|(i, item)| {
      if i > 0 {
        self.comma();
      }
      writer(self, item);
    });
    self.depth -= 1;
    self.nl();
    self.out.push(']');
  }

  /// Write span object inline (no newlines).
  #[inline]
  fn write_span(&mut self, span: &Span) {
    // Build span string directly without format! overhead
    self.out.push_str("\"span\":{\"start\":");
    write_usize(&mut self.out, span.start);
    self.out.push_str(",\"end\":");
    write_usize(&mut self.out, span.end);
    self.out.push_str(",\"line\":");
    write_usize(&mut self.out, span.line);
    self.out.push_str(",\"column\":");
    write_usize(&mut self.out, span.column);
    self.out.push('}');
  }

  /// Write metadata object.
  #[inline]
  fn write_metadata(&mut self, meta: &DocumentMetadata) {
    self.key("metadata");
    self.out.push('{');
    if let Some(t) = meta.title.as_ref() {
      self.out.push_str("\"title\":\"");
      escape_into(&mut self.out, t);
      self.out.push_str("\",");
    }
    if let Some(d) = meta.description.as_ref() {
      self.out.push_str("\"description\":\"");
      escape_into(&mut self.out, d);
      self.out.push_str("\",");
    }
    self.out.push_str("\"total_lines\":");
    write_usize(&mut self.out, meta.total_lines);
    self.out.push_str(",\"total_nodes\":");
    write_usize(&mut self.out, meta.total_nodes);
    self.out.push('}');
  }

  /// Write a JSON key (with colon).
  #[inline]
  fn key(&mut self, k: &str) {
    self.out.push('"');
    self.out.push_str(k);
    self.out.push_str("\":");
  }

  /// Write a key-value pair with string value.
  #[inline]
  fn kv_str(&mut self, k: &str, v: &str) {
    self.out.push('"');
    self.out.push_str(k);
    self.out.push_str("\":\"");
    escape_into(&mut self.out, v);
    self.out.push('"');
  }

  /// Write a key-value pair with raw (unescaped) value.
  #[inline]
  fn kv_raw(&mut self, k: &str, v: &str) {
    self.out.push('"');
    self.out.push_str(k);
    self.out.push_str("\":\"");
    self.out.push_str(v);
    self.out.push('"');
  }

  /// Write comma and newline.
  #[inline]
  fn comma(&mut self) {
    self.out.push(',');
    self.nl();
  }

  /// Write newline and indentation (if pretty mode).
  #[inline]
  fn nl(&mut self) {
    if self.pretty {
      self.out.push('\n');
      // Unrolled indentation for common depths (0-8)
      let indent = self.depth * 2;
      if indent <= 16 {
        self.out.push_str(&"                "[..indent]);
      } else {
        for _ in 0..self.depth {
          self.out.push_str("  ");
        }
      }
    }
  }
}

/// Write usize as decimal string directly into buffer.
/// Avoids format! allocation for numbers.
#[inline]
fn write_usize(out: &mut String, n: usize) {
  if n == 0 {
    out.push('0');
    return;
  }

  // Max usize is ~20 digits
  let mut buf = [0u8; 20];
  let mut i = 20;
  let mut num = n;

  while num > 0 {
    i -= 1;
    buf[i] = b'0' + (num % 10) as u8;
    num /= 10;
  }

  // SAFETY: digits are all ASCII
  out.push_str(unsafe { std::str::from_utf8_unchecked(&buf[i..]) });
}

/// Escape string and append directly to output buffer.
/// Avoids creating intermediate String allocation.
#[inline]
pub fn escape_into(out: &mut String, s: &str) {
  for c in s.chars() {
    match c {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if c.is_control() => {
        out.push_str("\\u");
        let n = c as u32;
        for shift in [12, 8, 4, 0] {
          let digit = ((n >> shift) & 0xF) as u8;
          out.push(if digit < 10 {
            (b'0' + digit) as char
          } else {
            (b'a' + digit - 10) as char
          });
        }
      }
      c => out.push(c),
    }
  }
}

/// Legacy escape function for compatibility.
/// Returns new String (use escape_into for better performance).
pub fn esc(s: &str) -> String {
  let mut out = String::with_capacity(s.len() + 16);
  escape_into(&mut out, s);
  out
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::{Node, Span};

  fn simple_doc() -> Document {
    Document {
      source_path: "test.md".to_string(),
      doc_type: DocumentType::Markdown,
      nodes: vec![Node::new(NodeKind::Paragraph, Span::new(0, 5, 1, 1))],
      metadata: DocumentMetadata {
        title: Some("Test".to_string()),
        description: None,
        total_lines: 1,
        total_nodes: 1,
      },
    }
  }

  #[test]
  fn test_to_json_basic() {
    let doc = simple_doc();
    let json = to_json(&doc);
    assert!(json.contains("\"source_path\":\"test.md\""));
    assert!(json.contains("\"doc_type\":\"Markdown\""));
    assert!(json.contains("\"Paragraph\""));
  }

  #[test]
  fn test_to_json_pretty() {
    let doc = simple_doc();
    let json = to_json_pretty(&doc);
    assert!(json.contains('\n'));
    assert!(json.contains("  ")); // Indentation
  }

  #[test]
  fn test_json_escape_quotes() {
    let result = esc("hello \"world\"");
    assert_eq!(result, "hello \\\"world\\\"");
  }

  #[test]
  fn test_json_escape_backslash() {
    let result = esc("path\\to\\file");
    assert_eq!(result, "path\\\\to\\\\file");
  }

  #[test]
  fn test_json_escape_newline() {
    let result = esc("line1\nline2");
    assert_eq!(result, "line1\\nline2");
  }

  #[test]
  fn test_json_escape_tab() {
    let result = esc("col1\tcol2");
    assert_eq!(result, "col1\\tcol2");
  }

  #[test]
  fn test_json_escape_carriage_return() {
    let result = esc("line\r\n");
    assert_eq!(result, "line\\r\\n");
  }

  #[test]
  fn test_json_escape_control_char() {
    let result = esc("\x00\x1f");
    assert!(result.contains("\\u0000"));
    assert!(result.contains("\\u001f"));
  }

  #[test]
  fn test_json_no_escape_normal() {
    let result = esc("normal text 123");
    assert_eq!(result, "normal text 123");
  }

  #[test]
  fn test_json_with_metadata() {
    let doc = Document {
      source_path: "test.md".to_string(),
      doc_type: DocumentType::Markdown,
      nodes: vec![],
      metadata: DocumentMetadata {
        title: Some("My Title".to_string()),
        description: Some("My Description".to_string()),
        total_lines: 10,
        total_nodes: 5,
      },
    };
    let json = to_json(&doc);
    assert!(json.contains("\"title\":\"My Title\""));
    assert!(json.contains("\"description\":\"My Description\""));
    assert!(json.contains("\"total_lines\":10"));
    assert!(json.contains("\"total_nodes\":5"));
  }

  #[test]
  fn test_json_nested_nodes() {
    let doc = Document {
      source_path: "".to_string(),
      doc_type: DocumentType::Markdown,
      nodes: vec![Node::with_children(
        NodeKind::Paragraph,
        Span::empty(),
        vec![Node::new(
          NodeKind::Text {
            content: "hello".to_string(),
          },
          Span::empty(),
        )],
      )],
      metadata: DocumentMetadata::default(),
    };
    let json = to_json(&doc);
    assert!(json.contains("\"children\""));
    assert!(json.contains("\"Text\""));
    assert!(json.contains("\"hello\""));
  }

  #[test]
  fn test_json_empty_document() {
    let doc = Document {
      source_path: "".to_string(),
      doc_type: DocumentType::Markdown,
      nodes: vec![],
      metadata: DocumentMetadata::default(),
    };
    let json = to_json(&doc);
    assert!(json.contains("\"nodes\":[]") || json.contains("\"nodes\":["));
  }
}
