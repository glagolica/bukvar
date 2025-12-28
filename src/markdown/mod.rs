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

  #[test]
  fn test_alert_note() {
    let input = "> [!NOTE]\n> This is a note.";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_alert = doc.nodes.iter().any(|n| {
      matches!(
        &n.kind,
        NodeKind::Alert {
          alert_type: crate::ast::AlertType::Note
        }
      )
    });
    assert!(has_alert, "Should parse [!NOTE] as Alert node");
  }

  #[test]
  fn test_alert_warning() {
    let input = "> [!WARNING]\n> This is a warning.";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_alert = doc.nodes.iter().any(|n| {
      matches!(
        &n.kind,
        NodeKind::Alert {
          alert_type: crate::ast::AlertType::Warning
        }
      )
    });
    assert!(has_alert, "Should parse [!WARNING] as Alert node");
  }

  #[test]
  fn test_alert_types() {
    for alert in ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"] {
      let input = format!("> [!{}]\n> Content", alert);
      let mut parser = MarkdownParser::new(&input);
      let doc = parser.parse();
      let has_alert = doc
        .nodes
        .iter()
        .any(|n| matches!(&n.kind, NodeKind::Alert { .. }));
      assert!(has_alert, "Should parse [!{}] as Alert node", alert);
    }
  }

  #[test]
  fn test_toc_element() {
    let input = "<toc>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_toc = doc.nodes.iter().any(|n| matches!(&n.kind, NodeKind::Toc));
    assert!(has_toc, "Should parse <toc> element");
  }

  #[test]
  fn test_toc_self_closing() {
    let input = "<toc />";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_toc = doc.nodes.iter().any(|n| matches!(&n.kind, NodeKind::Toc));
    assert!(has_toc, "Should parse <toc /> element");
  }

  #[test]
  fn test_steps_element() {
    let input = "<steps>\n<step>\n### Step 1\n</step>\n</steps>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_steps = doc.nodes.iter().any(|n| matches!(&n.kind, NodeKind::Steps));
    assert!(has_steps, "Should parse <steps> element");
  }

  #[test]
  fn test_tabs_element() {
    let input = "<tabs names=\"JS, Python\">\n```js\nconsole.log()\n```\n</tabs>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let tabs = doc
      .nodes
      .iter()
      .find(|n| matches!(&n.kind, NodeKind::Tabs { .. }));
    assert!(tabs.is_some(), "Should parse <tabs> element");
    if let Some(node) = tabs {
      if let NodeKind::Tabs { names } = &node.kind {
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "JS");
        assert_eq!(names[1], "Python");
      }
    }
  }

  #[test]
  fn test_code_block_highlight() {
    let input = "```go highlight=\"3, 5-7\"\npackage main\n```";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let code_ext = doc.nodes.iter().find(|n| {
      matches!(
        &n.kind,
        NodeKind::CodeBlockExt {
          highlight: Some(_),
          ..
        }
      )
    });
    assert!(code_ext.is_some(), "Should parse highlight attribute");
  }

  #[test]
  fn test_code_block_diff() {
    let input = "```gleam plusdiff=\"5\" minusdiff=\"4\"\ncode\n```";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let code_ext = doc.nodes.iter().find(|n| {
      matches!(
        &n.kind,
        NodeKind::CodeBlockExt {
          plusdiff: Some(_),
          minusdiff: Some(_),
          ..
        }
      )
    });
    assert!(code_ext.is_some(), "Should parse diff attributes");
  }

  #[test]
  fn test_code_block_linenumbers() {
    let input = "```kt linenumbers\nfun main() {}\n```";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let code_ext = doc.nodes.iter().find(|n| {
      matches!(
        &n.kind,
        NodeKind::CodeBlockExt {
          linenumbers: true,
          ..
        }
      )
    });
    assert!(code_ext.is_some(), "Should parse linenumbers attribute");
  }

  // ============================================
  // EDGE CASES: Headings
  // ============================================

  #[test]
  fn test_heading_no_space() {
    let input = "#No space after hash";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Should NOT be a heading without space
    let has_heading = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::Heading { .. }));
    assert!(!has_heading);
  }

  #[test]
  fn test_heading_too_many_hashes() {
    let input = "####### Seven hashes";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // More than 6 hashes - behavior varies by parser, just don't crash
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_heading_empty() {
    let input = "# \n## ";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Empty headings should still be valid
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_heading_with_trailing_hashes() {
    let input = "# Heading #####";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_heading = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::Heading { level: 1, .. }));
    assert!(has_heading);
  }

  #[test]
  fn test_heading_with_inline_formatting() {
    let input = "## **Bold** and *italic* heading";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_heading = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::Heading { .. }));
    assert!(has_heading);
  }

  #[test]
  fn test_setext_heading_h1() {
    let input = "Heading\n=======";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Setext headings may or may not be supported - just don't crash
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_setext_heading_h2() {
    let input = "Heading\n-------";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Setext headings may or may not be supported - just don't crash
    let _ = doc.nodes.len();
  }

  // ============================================
  // EDGE CASES: Emphasis and Strong
  // ============================================

  #[test]
  fn test_emphasis_underscore() {
    let input = "_italic_";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_strong_underscore() {
    let input = "__bold__";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_emphasis_mid_word_asterisk() {
    let input = "foo*bar*baz";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_emphasis_mid_word_underscore() {
    let input = "foo_bar_baz";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Underscores mid-word typically don't create emphasis
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_unmatched_emphasis() {
    let input = "*unclosed emphasis";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Should not panic, treat as literal
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_unmatched_strong() {
    let input = "**unclosed strong";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_emphasis_with_spaces() {
    let input = "* not emphasis *";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Leading/trailing spaces usually prevent emphasis
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_nested_strong_and_emphasis() {
    let input = "***bold and italic***";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_alternating_emphasis() {
    let input = "*a* **b** *c* **d**";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Code spans
  // ============================================

  #[test]
  fn test_code_span_with_backticks_inside() {
    let input = "`` `code` ``";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_span_unclosed() {
    let input = "`unclosed code";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_span_empty() {
    let input = "``";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Empty backticks - behavior varies, just don't crash
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_code_span_multiline() {
    let input = "`code across lines`";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_span_double_backtick() {
    let input = "``double backtick``";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Links
  // ============================================

  #[test]
  fn test_link_with_title() {
    let input = r#"[text](url "title")"#;
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_with_single_quote_title() {
    let input = "[text](url 'title')";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_empty_text() {
    let input = "[](url)";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_empty_url() {
    let input = "[text]()";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_with_parens_in_url() {
    let input = "[text](url_(with)_parens)";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_with_angle_brackets() {
    let input = "[text](<url with spaces>)";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_unclosed() {
    let input = "[text](url with spaces";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Should handle unclosed links gracefully - don't crash
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_nested_brackets() {
    let input = "[[nested]]";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_reference_undefined() {
    let input = "[text][undefined-ref]";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_reference_shortcut() {
    let input = "[ref]\n\n[ref]: http://example.com";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_link_reference_collapsed() {
    let input = "[ref][]\n\n[ref]: http://example.com";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Images
  // ============================================

  #[test]
  fn test_image_with_title() {
    let input = r#"![alt](image.png "title")"#;
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_image_empty_alt() {
    let input = "![](image.png)";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_image_reference() {
    let input = "![alt][ref]\n\n[ref]: image.png";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Lists
  // ============================================

  #[test]
  fn test_list_different_markers() {
    let input = "- dash\n* asterisk\n+ plus";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_nested_list() {
    let input = "- item\n  - nested\n    - deeply nested";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_list_with_blank_lines() {
    let input = "- item 1\n\n- item 2\n\n- item 3";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_ordered_list_not_starting_at_1() {
    let input = "5. fifth\n6. sixth\n7. seventh";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_ordered_list_same_numbers() {
    let input = "1. first\n1. second\n1. third";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_list_with_paragraphs() {
    let input = "- item 1\n\n  paragraph in item\n\n- item 2";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_task_list_variations() {
    let input = "- [ ] unchecked\n- [x] checked\n- [X] also checked";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_list_multiline_item() {
    let input = "- first line\n  continuation\n  more text";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Code Blocks
  // ============================================

  #[test]
  fn test_code_block_four_spaces() {
    let input = "    indented code\n    more code";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_block_tilde() {
    let input = "~~~\ncode block\n~~~";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_block_more_backticks() {
    let input = "````\n```\ncode with backticks\n```\n````";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_block_unclosed() {
    let input = "```\nunclosed code block";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Should handle gracefully
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_block_empty() {
    let input = "```\n```";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_block_with_info_string() {
    let input = "```javascript:file.js\nconsole.log();\n```";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_code_block_all_attributes() {
    let input = "```rust highlight=\"1-3\" plusdiff=\"4\" minusdiff=\"5\" linenumbers\ncode\n```";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let code_ext = doc.nodes.iter().find(|n| {
      matches!(
        &n.kind,
        NodeKind::CodeBlockExt {
          highlight: Some(_),
          plusdiff: Some(_),
          minusdiff: Some(_),
          linenumbers: true,
          ..
        }
      )
    });
    assert!(code_ext.is_some(), "Should parse all code block attributes");
  }

  // ============================================
  // EDGE CASES: Blockquotes
  // ============================================

  #[test]
  fn test_blockquote_nested() {
    let input = "> level 1\n>> level 2\n>>> level 3";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_blockquote_lazy_continuation() {
    let input = "> first line\ncontinued without marker";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_blockquote_with_code() {
    let input = "> ```\n> code\n> ```";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_blockquote_empty() {
    let input = ">\n>\n>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Alerts (GitHub style)
  // ============================================

  #[test]
  fn test_alert_with_multiple_paragraphs() {
    let input = "> [!NOTE]\n> First paragraph\n>\n> Second paragraph";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_alert = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::Alert { .. }));
    assert!(has_alert);
  }

  #[test]
  fn test_alert_lowercase() {
    let input = "> [!note]\n> lowercase alert";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Should still work case-insensitively
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_alert_invalid_type() {
    let input = "> [!INVALID]\n> Not a real alert type";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Should fall back to regular blockquote
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_alert_no_space() {
    let input = ">[!NOTE]\n>No space";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Tables
  // ============================================

  #[test]
  fn test_table_basic() {
    let input = "| a | b |\n|---|---|\n| 1 | 2 |";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_table_alignment() {
    let input = "| left | center | right |\n|:-----|:------:|------:|\n| a | b | c |";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_table_no_outer_pipes() {
    let input = "a | b\n--|--\n1 | 2";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_table_escaped_pipe() {
    let input = "| a \\| b | c |\n|--------|---|\n| 1 | 2 |";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_table_empty_cells() {
    let input = "| a | | c |\n|---|---|---|\n| | b | |";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_table_single_column() {
    let input = "| a |\n|---|\n| 1 |";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Horizontal Rules
  // ============================================

  #[test]
  fn test_hr_asterisks() {
    let input = "***";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_hr = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::ThematicBreak));
    assert!(has_hr);
  }

  #[test]
  fn test_hr_underscores() {
    let input = "___";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_hr = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::ThematicBreak));
    assert!(has_hr);
  }

  #[test]
  fn test_hr_with_spaces() {
    let input = "- - - - -";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_hr = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::ThematicBreak));
    assert!(has_hr);
  }

  #[test]
  fn test_hr_many_chars() {
    let input = "----------------------------------------";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_hr = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::ThematicBreak));
    assert!(has_hr);
  }

  // ============================================
  // EDGE CASES: Math
  // ============================================

  #[test]
  fn test_math_inline_with_spaces() {
    let input = "$ x + y $";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_math_block_empty() {
    let input = "$$\n$$";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_math_with_latex() {
    let input = "$$\n\\frac{a}{b} \\int_{0}^{\\infty} e^{-x^2} dx\n$$";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_math_inline_unclosed() {
    let input = "$unclosed math";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Footnotes
  // ============================================

  #[test]
  fn test_footnote_multiline() {
    let input = "Text[^1]\n\n[^1]: First line\n    Second line\n    Third line";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_footnote_with_code() {
    let input = "Text[^1]\n\n[^1]: Contains `code`";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_footnote_numeric() {
    let input = "Text[^123]\n\n[^123]: Numeric footnote";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_footnote_alphanumeric() {
    let input = "Text[^abc123]\n\n[^abc123]: Alphanumeric footnote";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Frontmatter
  // ============================================

  #[test]
  fn test_frontmatter_toml() {
    let input = "+++\ntitle = \"Test\"\n+++\n\n# Content";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_frontmatter_not_at_start() {
    let input = "Some text\n---\ntitle: Test\n---";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    // Should not parse as frontmatter if not at start
    let has_fm = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::Frontmatter { .. }));
    assert!(!has_fm);
  }

  #[test]
  fn test_frontmatter_empty() {
    let input = "---\n---\n\nContent";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_frontmatter_complex_yaml() {
    let input = "---\ntitle: Test\ntags:\n  - rust\n  - parser\ndate: 2024-01-01\n---\n\n# Content";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Special Characters & Unicode
  // ============================================

  #[test]
  fn test_unicode_text() {
    let input = "こんにちは世界 🎉 Привет мир";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_unicode_in_heading() {
    let input = "# 中文标题";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_heading = doc
      .nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::Heading { .. }));
    assert!(has_heading);
  }

  #[test]
  fn test_emoji_in_text() {
    let input = "I ❤️ Rust 🦀";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_escaped_characters() {
    let input = r"\* \_ \` \[ \] \# \+ \- \. \!";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_html_entities() {
    let input = "&amp; &lt; &gt; &quot;";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_special_url_characters() {
    let input = "[link](https://example.com/path?query=value&other=123#anchor)";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Custom Elements (Glagolica)
  // ============================================

  #[test]
  fn test_toc_with_whitespace() {
    let input = "<toc  />";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_toc = doc.nodes.iter().any(|n| matches!(&n.kind, NodeKind::Toc));
    assert!(has_toc);
  }

  #[test]
  fn test_steps_numbered() {
    let input = "<steps>\n1. Step one\n2. Step two\n</steps>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let has_steps = doc.nodes.iter().any(|n| matches!(&n.kind, NodeKind::Steps));
    assert!(has_steps);
  }

  #[test]
  fn test_steps_empty() {
    let input = "<steps>\n</steps>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_tabs_empty_names() {
    let input = "<tabs names=\"\">\ncontent\n</tabs>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_tabs_single_tab() {
    let input = "<tabs names=\"Single\">\ncontent\n</tabs>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let tabs = doc
      .nodes
      .iter()
      .find(|n| matches!(&n.kind, NodeKind::Tabs { .. }));
    assert!(tabs.is_some());
  }

  #[test]
  fn test_tabs_many_tabs() {
    let input = "<tabs names=\"A, B, C, D, E, F\">\ncontent\n</tabs>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let tabs = doc
      .nodes
      .iter()
      .find(|n| matches!(&n.kind, NodeKind::Tabs { .. }));
    if let Some(node) = tabs {
      if let NodeKind::Tabs { names } = &node.kind {
        assert_eq!(names.len(), 6);
      }
    }
  }

  // ============================================
  // EDGE CASES: Malformed / Edge Input
  // ============================================

  #[test]
  fn test_only_newlines() {
    let input = "\n\n\n\n\n";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_only_spaces() {
    let input = "          ";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_mixed_line_endings() {
    let input = "line1\r\nline2\nline3\rline4";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_tabs_vs_spaces() {
    let input = "\tindented with tab\n    indented with spaces";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_very_long_line() {
    let input = "a".repeat(10000);
    let mut parser = MarkdownParser::new(&input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_deeply_nested_lists() {
    let input = "- a\n  - b\n    - c\n      - d\n        - e\n          - f";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_null_byte() {
    let input = "text with \0 null byte";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_trailing_whitespace() {
    let input = "text with trailing spaces   \nmore text";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_single_char() {
    let input = "a";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: HTML
  // ============================================

  #[test]
  fn test_inline_html() {
    let input = "<span>inline html</span>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_html_block() {
    let input = "<div>\n\nContent inside\n\n</div>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_html_comment() {
    let input = "<!-- comment -->\n\nContent";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_script_tag() {
    let input = "<script>alert('xss')</script>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Autolinks
  // ============================================

  #[test]
  fn test_autolink_email() {
    let input = "<user@example.com>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_bare_url() {
    let input = "Visit https://example.com for more info.";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_autolink_with_path() {
    let input = "<https://example.com/path/to/page>";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Definition Lists
  // ============================================

  #[test]
  fn test_definition_multiple_definitions() {
    let input = "Term\n: Definition 1\n: Definition 2";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_definition_multiple_terms() {
    let input = "Term 1\nTerm 2\n: Shared definition";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // EDGE CASES: Hard Line Breaks
  // ============================================

  #[test]
  fn test_hard_break_two_spaces() {
    let input = "line 1  \nline 2";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_hard_break_backslash() {
    let input = "line 1\\\nline 2";
    let mut parser = MarkdownParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }
}
