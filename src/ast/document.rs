//! Document types and metadata for parsed files.
//!
//! A [`Document`] is the root container for all parsed content,
//! storing the AST nodes plus metadata about the source.

/// Represents a fully parsed source file.
///
/// Contains the AST nodes and metadata about the document.
#[derive(Debug, Clone)]
pub struct Document {
  /// Path to the source file (may be empty for strings)
  pub source_path: String,
  /// Type of document (Markdown, JS, Python, etc.)
  pub doc_type: DocumentType,
  /// Root-level AST nodes
  pub nodes: Vec<super::Node>,
  /// Document metadata (title, line count, etc.)
  pub metadata: DocumentMetadata,
}

impl Document {
  /// Create a new empty document of the given type.
  #[allow(dead_code)] // Part of public API
  pub fn new(doc_type: DocumentType) -> Self {
    Self {
      source_path: String::new(),
      doc_type,
      nodes: Vec::new(),
      metadata: DocumentMetadata::default(),
    }
  }

  /// Count total nodes in the document tree.
  #[allow(dead_code)]
  pub fn node_count(&self) -> usize {
    self.nodes.iter().map(|n| n.count_nodes()).sum()
  }
}

/// Type of document being parsed.
///
/// Determines which parser is used and affects output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
  Markdown,
  JavaScript,
  TypeScript,
  Java,
  Python,
}

impl DocumentType {
  /// Determine document type from file extension.
  ///
  /// # Examples
  /// ```ignore
  /// assert_eq!(DocumentType::from_extension("md"), Some(DocumentType::Markdown));
  /// assert_eq!(DocumentType::from_extension("py"), Some(DocumentType::Python));
  /// ```
  pub fn from_extension(ext: &str) -> Option<Self> {
    match ext.to_lowercase().as_str() {
      "md" | "markdown" | "mdown" | "mkd" => Some(Self::Markdown),
      "js" | "mjs" | "cjs" => Some(Self::JavaScript),
      "ts" | "tsx" | "mts" | "cts" => Some(Self::TypeScript),
      "java" => Some(Self::Java),
      "py" | "pyi" | "pyw" => Some(Self::Python),
      _ => None,
    }
  }

  /// Get canonical file extension for this document type.
  #[allow(dead_code)]
  pub fn extension(&self) -> &'static str {
    match self {
      Self::Markdown => "md",
      Self::JavaScript => "js",
      Self::TypeScript => "ts",
      Self::Java => "java",
      Self::Python => "py",
    }
  }
}

/// Metadata extracted from a parsed document.
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
  /// Document title (from first heading or frontmatter)
  pub title: Option<String>,
  /// Document description
  pub description: Option<String>,
  /// Total lines in source
  pub total_lines: usize,
  /// Total AST nodes generated
  pub total_nodes: usize,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_document_type_from_extension() {
    assert_eq!(
      DocumentType::from_extension("md"),
      Some(DocumentType::Markdown)
    );
    assert_eq!(
      DocumentType::from_extension("js"),
      Some(DocumentType::JavaScript)
    );
    assert_eq!(
      DocumentType::from_extension("py"),
      Some(DocumentType::Python)
    );
    assert_eq!(DocumentType::from_extension("unknown"), None);
  }

  #[test]
  fn test_document_type_extension() {
    assert_eq!(DocumentType::Markdown.extension(), "md");
    assert_eq!(DocumentType::Python.extension(), "py");
  }
}
