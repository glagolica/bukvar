//! AST validation - check for broken links, missing refs

use crate::ast::{Document, Node, NodeKind};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct ValidationResult {
  pub warnings: Vec<ValidationWarning>,
  pub errors: Vec<ValidationError>,
}

#[derive(Debug)]
pub struct ValidationWarning {
  pub line: usize,
  pub message: String,
}

#[derive(Debug)]
pub struct ValidationError {
  pub line: usize,
  pub message: String,
}

impl ValidationResult {
  pub fn is_ok(&self) -> bool {
    self.errors.is_empty()
  }

  pub fn has_warnings(&self) -> bool {
    !self.warnings.is_empty()
  }
}

/// Validate a document for common issues
pub fn validate(doc: &Document) -> ValidationResult {
  let mut result = ValidationResult::default();
  let mut link_defs = HashSet::new();
  let mut footnote_defs = HashSet::new();
  let mut link_refs = Vec::new();
  let mut footnote_refs = Vec::new();

  // Collect definitions and references
  collect_refs(
    &doc.nodes,
    &mut link_defs,
    &mut footnote_defs,
    &mut link_refs,
    &mut footnote_refs,
  );

  // Check for undefined link references
  for (label, line) in link_refs {
    if !link_defs.contains(&label.to_lowercase()) {
      result.warnings.push(ValidationWarning {
        line,
        message: format!("undefined link reference: [{}]", label),
      });
    }
  }

  // Check for undefined footnote references
  for (label, line) in footnote_refs {
    if !footnote_defs.contains(&label.to_lowercase()) {
      result.warnings.push(ValidationWarning {
        line,
        message: format!("undefined footnote: [^{}]", label),
      });
    }
  }

  // Check for empty links
  check_empty_links(&doc.nodes, &mut result);

  result
}

fn collect_refs(
  nodes: &[Node],
  link_defs: &mut HashSet<String>,
  footnote_defs: &mut HashSet<String>,
  link_refs: &mut Vec<(String, usize)>,
  footnote_refs: &mut Vec<(String, usize)>,
) {
  for node in nodes {
    match &node.kind {
      NodeKind::LinkDefinition { label, .. } => {
        link_defs.insert(label.to_lowercase());
      }
      NodeKind::LinkReference { label, .. } => {
        link_refs.push((label.clone(), node.span.line));
      }
      NodeKind::FootnoteDefinition { label } => {
        footnote_defs.insert(label.to_lowercase());
      }
      NodeKind::FootnoteReference { label } => {
        footnote_refs.push((label.clone(), node.span.line));
      }
      NodeKind::Footnote { label } => {
        footnote_defs.insert(label.to_lowercase());
      }
      _ => {}
    }
    collect_refs(
      &node.children,
      link_defs,
      footnote_defs,
      link_refs,
      footnote_refs,
    );
  }
}

fn check_empty_links(nodes: &[Node], result: &mut ValidationResult) {
  for node in nodes {
    match &node.kind {
      NodeKind::Link { url, .. } if url.is_empty() => {
        result.warnings.push(ValidationWarning {
          line: node.span.line,
          message: "empty link URL".to_string(),
        });
      }
      NodeKind::Image { url, .. } if url.is_empty() => {
        result.warnings.push(ValidationWarning {
          line: node.span.line,
          message: "empty image URL".to_string(),
        });
      }
      _ => {}
    }
    check_empty_links(&node.children, result);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::{Document, DocumentMetadata, DocumentType};

  fn empty_doc() -> Document {
    Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes: vec![],
      metadata: DocumentMetadata::default(),
    }
  }

  #[test]
  fn test_valid_doc() {
    let doc = empty_doc();
    let result = validate(&doc);
    assert!(result.is_ok());
    assert!(!result.has_warnings());
  }

  #[test]
  fn test_validation_result_errors() {
    let mut result = ValidationResult::default();
    assert!(result.errors.is_empty());
    result.errors.push(ValidationError {
      line: 1,
      message: "Test error".to_string(),
    });
    assert!(!result.is_ok());
  }

  #[test]
  fn test_validation_result_warnings() {
    let mut result = ValidationResult::default();
    assert!(result.warnings.is_empty());
    result.warnings.push(ValidationWarning {
      line: 1,
      message: "Test warning".to_string(),
    });
    assert!(result.has_warnings());
    assert!(result.is_ok()); // warnings don't make it not ok
  }

  #[test]
  fn test_broken_link_reference() {
    use crate::ast::{Node, NodeKind, ReferenceType, Span};
    let doc = Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes: vec![Node::new(
        NodeKind::LinkReference {
          label: "nonexistent".to_string(),
          ref_type: ReferenceType::Full,
        },
        Span::empty(),
      )],
      metadata: DocumentMetadata::default(),
    };
    let result = validate(&doc);
    assert!(!result.errors.is_empty() || !result.warnings.is_empty());
  }

  #[test]
  fn test_broken_footnote_reference() {
    use crate::ast::{Node, NodeKind, Span};
    let doc = Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes: vec![Node::new(
        NodeKind::FootnoteReference {
          label: "missing".to_string(),
        },
        Span::empty(),
      )],
      metadata: DocumentMetadata::default(),
    };
    let result = validate(&doc);
    assert!(!result.errors.is_empty() || !result.warnings.is_empty());
  }

  #[test]
  fn test_empty_link() {
    use crate::ast::{Node, NodeKind, ReferenceType, Span};
    let doc = Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes: vec![Node::new(
        NodeKind::Link {
          url: "".to_string(),
          title: None,
          ref_type: ReferenceType::Full,
        },
        Span::empty(),
      )],
      metadata: DocumentMetadata::default(),
    };
    let result = validate(&doc);
    assert!(result.has_warnings());
  }

  #[test]
  fn test_valid_link() {
    use crate::ast::{Node, NodeKind, ReferenceType, Span};
    let doc = Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes: vec![Node::new(
        NodeKind::Link {
          url: "https://example.com".to_string(),
          title: Some("Example".to_string()),
          ref_type: ReferenceType::Full,
        },
        Span::empty(),
      )],
      metadata: DocumentMetadata::default(),
    };
    let result = validate(&doc);
    assert!(result.is_ok());
  }

  #[test]
  fn test_matching_link_definition() {
    use crate::ast::{Node, NodeKind, ReferenceType, Span};
    let doc = Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes: vec![
        Node::new(
          NodeKind::LinkReference {
            label: "example".to_string(),
            ref_type: ReferenceType::Full,
          },
          Span::empty(),
        ),
        Node::new(
          NodeKind::LinkDefinition {
            label: "example".to_string(),
            url: "https://example.com".to_string(),
            title: None,
          },
          Span::empty(),
        ),
      ],
      metadata: DocumentMetadata::default(),
    };
    let result = validate(&doc);
    assert!(result.is_ok());
  }

  #[test]
  fn test_matching_footnote() {
    use crate::ast::{Node, NodeKind, Span};
    let doc = Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes: vec![
        Node::new(
          NodeKind::FootnoteReference {
            label: "1".to_string(),
          },
          Span::empty(),
        ),
        Node::new(
          NodeKind::FootnoteDefinition {
            label: "1".to_string(),
          },
          Span::empty(),
        ),
      ],
      metadata: DocumentMetadata::default(),
    };
    let result = validate(&doc);
    assert!(result.is_ok());
  }

  #[test]
  fn test_nested_validation() {
    use crate::ast::{Node, NodeKind, Span};
    let doc = Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes: vec![Node::with_children(
        NodeKind::Paragraph,
        Span::empty(),
        vec![Node::new(
          NodeKind::FootnoteReference {
            label: "missing".to_string(),
          },
          Span::empty(),
        )],
      )],
      metadata: DocumentMetadata::default(),
    };
    let result = validate(&doc);
    // Should find the broken reference in children
    assert!(!result.errors.is_empty() || !result.warnings.is_empty());
  }
}
