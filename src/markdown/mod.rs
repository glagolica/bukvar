//! GFM markdown parser.
//!
//! Two-pass: first collects link defs, then parses blocks/inlines.

mod block;
mod frontmatter;
mod inline;
mod linkdef;
mod scanner;

use crate::ast::{Document, DocumentMetadata, DocumentType, Node};

pub use block::BlockParser;
pub use inline::InlineParser;
pub use linkdef::LinkDef;
pub use scanner::Scanner;

/// Main parser. Create with `new()`, call `parse()`.
pub struct MarkdownParser<'a> {
  scanner: Scanner<'a>,
  link_defs: Vec<LinkDef>,
  frontmatter: Option<Node>,
}

impl<'a> MarkdownParser<'a> {
  pub fn new(input: &'a str) -> Self {
    Self {
      scanner: Scanner::new(input),
      link_defs: Vec::new(),
      frontmatter: None,
    }
  }

  /// Parse input into Document AST.
  pub fn parse(&mut self) -> Document {
    self.frontmatter = frontmatter::try_parse(&mut self.scanner);
    self.link_defs = linkdef::collect_definitions(&mut self.scanner);
    self.scanner.reset();

    if self.frontmatter.is_some() {
      frontmatter::skip(&mut self.scanner);
    }

    let mut block_parser = BlockParser::new(&mut self.scanner, &self.link_defs);
    let mut nodes = block_parser.parse_blocks();

    if let Some(fm) = self.frontmatter.take() {
      nodes.insert(0, fm);
    }

    let total_nodes: usize = nodes.iter().map(|n| n.count_nodes()).sum();

    Document {
      source_path: String::new(),
      doc_type: DocumentType::Markdown,
      nodes,
      metadata: DocumentMetadata {
        title: None,
        description: None,
        total_lines: self.scanner.line(),
        total_nodes,
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::NodeKind;

  #[test]
  fn test_empty_input() {
    let mut parser = MarkdownParser::new("");
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_simple_paragraph() {
    let mut parser = MarkdownParser::new("Hello world");
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 1);
  }

  #[test]
  fn test_heading() {
    let mut parser = MarkdownParser::new("# Heading 1\n\n## Heading 2");
    let doc = parser.parse();
    assert!(doc.nodes.len() >= 2);
  }

  #[test]
  fn test_emphasis() {
    let mut parser = MarkdownParser::new("*italic* and **bold**");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_block() {
    let mut parser = MarkdownParser::new("```rust\nfn main() {}\n```");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link() {
    let mut parser = MarkdownParser::new("[text](http://example.com)");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_list() {
    let mut parser = MarkdownParser::new("- item 1\n- item 2\n- item 3");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_math_block() {
    let mut parser = MarkdownParser::new("$$\nx^2 + y^2 = z^2\n$$");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
    let has_math = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::MathBlock { .. }));
    assert!(has_math);
  }

  #[test]
  fn test_definition_list() {
    let mut parser = MarkdownParser::new("Term\n: Definition");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
    let has_def = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::DefinitionList));
    assert!(has_def);
  }

  #[test]
  fn test_frontmatter_yaml() {
    let input = "---\ntitle: Test\n---\n\n# Content";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty(), "Document should have nodes");
    let has_frontmatter = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::Frontmatter { .. }));
    assert!(has_frontmatter, "Document should contain frontmatter node");
  }

  #[test]
  fn test_blockquote() {
    let mut parser = MarkdownParser::new("> This is a quote\n> with multiple lines");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
    let has_quote = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::BlockQuote));
    assert!(has_quote);
  }

  #[test]
  fn test_ordered_list() {
    let mut parser = MarkdownParser::new("1. First\n2. Second\n3. Third");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_horizontal_rule() {
    let mut parser = MarkdownParser::new("---\n\nContent after");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_inline_code() {
    let mut parser = MarkdownParser::new("Use `code` here");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_image() {
    let mut parser = MarkdownParser::new("![alt text](image.png)");
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_reference() {
    let input = "[text][ref]\n\n[ref]: http://example.com";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_multiple_paragraphs() {
    let input = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.len() >= 3);
  }

  #[test]
  fn test_heading_levels() {
    let input = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 6);
  }

  #[test]
  fn test_code_block_with_language() {
    let input = "```python\nprint('hello')\n```";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let code_block = doc.nodes.iter().find(|n| {
      matches!(&n.kind, NodeKind::CodeBlock { language, .. } | NodeKind::FencedCodeBlock { language, .. } if language.is_some())
    });
    assert!(code_block.is_some());
  }

  #[test]
  fn test_nested_emphasis() {
    let input = "***bold and italic***";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_strikethrough() {
    let input = "~~deleted~~";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_autolink() {
    let input = "<https://example.com>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_task_list() {
    let input = "- [ ] Unchecked\n- [x] Checked";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_footnote() {
    let input = "Text[^1]\n\n[^1]: Footnote content";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_inline_math() {
    let input = "The formula $E = mc^2$ is famous.";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_document_type() {
    let mut parser = MarkdownParser::new("# Test");
    let doc = parser.parse();
    assert_eq!(doc.doc_type, DocumentType::Markdown);
  }

  #[test]
  fn test_metadata_lines() {
    let input = "Line 1\nLine 2\nLine 3";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(doc.metadata.total_lines > 0);
  }

  #[test]
  fn test_whitespace_only() {
    let input = "   \n\n   \n";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Should handle gracefully - either empty or contains whitespace nodes
    let _ = doc.nodes.len(); // Just verify it doesn't panic
  }

  #[test]
  fn test_mixed_content() {
    let input = r#"# Title

Some **bold** and *italic* text.

- List item 1
- List item 2

> A blockquote

```code
block
```
"#;
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.len() >= 4);
  }
}
