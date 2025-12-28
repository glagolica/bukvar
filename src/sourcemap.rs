//! Source map generation for mapping AST positions to source.
//!
//! Provides bidirectional mapping between AST node positions
//! and original source file locations.

use crate::ast::{Document, Node};

/// A single source map entry.
#[derive(Debug, Clone)]
pub struct SourceMapEntry {
  /// Start offset in source
  pub source_start: usize,
  /// End offset in source
  pub source_end: usize,
  /// Line number (1-based)
  pub line: usize,
  /// Column number (1-based)
  pub column: usize,
  /// Node type name
  pub node_type: String,
}

/// Source map for a document.
#[derive(Debug, Default)]
pub struct SourceMap {
  /// Source file path
  pub source_path: String,
  /// All mappings from AST nodes to source positions
  pub entries: Vec<SourceMapEntry>,
}

impl SourceMap {
  /// Create a new source map from a parsed document.
  pub fn from_document(doc: &Document) -> Self {
    let mut map = Self {
      source_path: doc.source_path.clone(),
      entries: Vec::new(),
    };
    map.collect_entries(&doc.nodes);
    map
  }

  /// Collect entries from nodes recursively.
  fn collect_entries(&mut self, nodes: &[Node]) {
    for node in nodes {
      let span = &node.span;
      if !span.is_empty() {
        self.entries.push(SourceMapEntry {
          source_start: span.start,
          source_end: span.end,
          line: span.line,
          column: span.column,
          node_type: node_type_name(&node.kind),
        });
      }
      self.collect_entries(&node.children);
    }
  }

  /// Find node at a given source offset.
  #[allow(dead_code)]
  pub fn find_at_offset(&self, offset: usize) -> Option<&SourceMapEntry> {
    self
      .entries
      .iter()
      .find(|e| offset >= e.source_start && offset < e.source_end)
  }

  /// Find nodes at a given line.
  #[allow(dead_code)]
  pub fn find_at_line(&self, line: usize) -> Vec<&SourceMapEntry> {
    self.entries.iter().filter(|e| e.line == line).collect()
  }

  /// Convert to JSON format.
  pub fn to_json(&self) -> String {
    let mut s = String::with_capacity(256);
    s.push_str("{\"source\":\"");
    s.push_str(&escape_json(&self.source_path));
    s.push_str("\",\"mappings\":[");
    for (i, entry) in self.entries.iter().enumerate() {
      if i > 0 {
        s.push(',');
      }
      s.push_str(&format!(
        "{{\"start\":{},\"end\":{},\"line\":{},\"col\":{},\"type\":\"{}\"}}",
        entry.source_start, entry.source_end, entry.line, entry.column, entry.node_type
      ));
    }
    s.push_str("]}");
    s
  }
}

/// Get node type name for source map.
fn node_type_name(kind: &crate::ast::NodeKind) -> String {
  use crate::ast::NodeKind::*;
  match kind {
    Document => "Document",
    Heading { .. } => "Heading",
    Paragraph => "Paragraph",
    Text { .. } => "Text",
    Emphasis => "Emphasis",
    Strong => "Strong",
    Link { .. } => "Link",
    Image { .. } => "Image",
    CodeSpan { .. } => "CodeSpan",
    CodeBlock { .. } => "CodeBlock",
    FencedCodeBlock { .. } => "FencedCodeBlock",
    IndentedCodeBlock => "IndentedCodeBlock",
    BlockQuote => "BlockQuote",
    List { .. } => "List",
    ListItem { .. } => "ListItem",
    ThematicBreak => "ThematicBreak",
    HardBreak => "HardBreak",
    SoftBreak => "SoftBreak",
    HtmlBlock { .. } => "HtmlBlock",
    Table => "Table",
    TableHead => "TableHead",
    TableBody => "TableBody",
    TableRow => "TableRow",
    TableCell { .. } => "TableCell",
    Strikethrough => "Strikethrough",
    AutoLink { .. } => "AutoLink",
    MathInline { .. } => "MathInline",
    MathBlock { .. } => "MathBlock",
    Frontmatter { .. } => "Frontmatter",
    Footnote { .. } => "Footnote",
    FootnoteReference { .. } => "FootnoteReference",
    FootnoteDefinition { .. } => "FootnoteDefinition",
    DefinitionList => "DefinitionList",
    DefinitionTerm => "DefinitionTerm",
    DefinitionDescription => "DefinitionDescription",
    AutoUrl { .. } => "AutoUrl",
    Alert { .. } => "Alert",
    Steps => "Steps",
    Step => "Step",
    Toc => "Toc",
    Tabs { .. } => "Tabs",
    CodeBlockExt { .. } => "CodeBlockExt",
    _ => "Unknown",
  }
  .to_string()
}

/// Escape string for JSON.
fn escape_json(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  for ch in s.chars() {
    match ch {
      '"' => result.push_str("\\\""),
      '\\' => result.push_str("\\\\"),
      '\n' => result.push_str("\\n"),
      '\r' => result.push_str("\\r"),
      '\t' => result.push_str("\\t"),
      c => result.push(c),
    }
  }
  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::{DocumentMetadata, DocumentType, NodeKind, Span};

  fn create_test_doc() -> Document {
    let mut doc = Document {
      source_path: "test.md".to_string(),
      doc_type: DocumentType::Markdown,
      metadata: DocumentMetadata::default(),
      nodes: vec![
        Node::new(
          NodeKind::Heading { level: 1, id: None },
          Span::new(0, 10, 1, 1),
        ),
        Node::new(NodeKind::Paragraph, Span::new(12, 50, 3, 1)),
        Node::new(NodeKind::Paragraph, Span::new(52, 80, 5, 1)),
      ],
    };
    doc.metadata.total_nodes = 3;
    doc
  }

  #[test]
  fn test_source_map_creation() {
    let doc = create_test_doc();
    let map = SourceMap::from_document(&doc);
    assert_eq!(map.source_path, "test.md");
    assert_eq!(map.entries.len(), 3);
    assert_eq!(map.entries[0].node_type, "Heading");
  }

  #[test]
  fn test_find_at_offset() {
    let doc = create_test_doc();
    let map = SourceMap::from_document(&doc);

    // Find heading at offset 5
    let entry = map.find_at_offset(5);
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().node_type, "Heading");

    // Find first paragraph at offset 20
    let entry = map.find_at_offset(20);
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().node_type, "Paragraph");
    assert_eq!(entry.unwrap().line, 3);

    // Find second paragraph at offset 60
    let entry = map.find_at_offset(60);
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().line, 5);

    // No node at offset 100
    let entry = map.find_at_offset(100);
    assert!(entry.is_none());
  }

  #[test]
  fn test_find_at_line() {
    let doc = create_test_doc();
    let map = SourceMap::from_document(&doc);

    // Find nodes at line 1 (heading)
    let entries = map.find_at_line(1);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].node_type, "Heading");

    // Find nodes at line 3 (first paragraph)
    let entries = map.find_at_line(3);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].node_type, "Paragraph");

    // Find nodes at line 5 (second paragraph)
    let entries = map.find_at_line(5);
    assert_eq!(entries.len(), 1);

    // No nodes at line 2
    let entries = map.find_at_line(2);
    assert!(entries.is_empty());
  }
}
