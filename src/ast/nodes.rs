//! AST nodes.

use super::types::AlertType;
use super::{Alignment, DocStyle, ListMarker, ReferenceType, Span};

/// AST node: kind + span + children.
#[derive(Debug, Clone)]
pub struct Node {
  pub kind: NodeKind,
  pub span: Span,
  pub children: Vec<Node>,
}

impl Node {
  #[inline]
  pub fn new(kind: NodeKind, span: Span) -> Self {
    Self {
      kind,
      span,
      children: Vec::new(),
    }
  }

  #[inline]
  pub fn with_children(kind: NodeKind, span: Span, children: Vec<Node>) -> Self {
    Self {
      kind,
      span,
      children,
    }
  }

  pub fn count_nodes(&self) -> usize {
    1 + self.children.iter().map(|c| c.count_nodes()).sum::<usize>()
  }

  #[inline]
  #[allow(dead_code)]
  pub fn is_leaf(&self) -> bool {
    self.children.is_empty()
  }
}

/// All possible node types in the AST.
///
/// Organized by category:
/// - Block elements (headings, paragraphs, lists, etc.)
/// - Inline elements (emphasis, links, code spans)
/// - GFM extensions (tables, strikethrough, task lists)
/// - Documentation comments (JSDoc, JavaDoc, PyDoc tags)
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[allow(dead_code)] // Many variants part of public API
pub enum NodeKind {
  // === Document Root ===
  Document,

  // === Block Elements ===
  /// ATX or Setext heading (level 1-6)
  Heading {
    level: u8,
    id: Option<String>,
  },
  /// Paragraph of text
  Paragraph,
  /// Block quote (> prefix)
  BlockQuote,
  /// Generic code block
  CodeBlock {
    language: Option<String>,
    info: Option<String>,
  },
  /// Fenced code block (``` or ~~~)
  FencedCodeBlock {
    language: Option<String>,
    info: Option<String>,
  },
  /// Indented code block (4+ spaces)
  IndentedCodeBlock,
  /// Raw HTML block
  HtmlBlock {
    block_type: u8,
  },
  /// Horizontal rule (---, ***, ___)
  ThematicBreak,

  // === Lists ===
  /// Ordered or unordered list container
  List {
    ordered: bool,
    start: Option<u32>,
    tight: bool,
  },
  /// Individual list item
  ListItem {
    marker: ListMarker,
    checked: Option<bool>,
  },

  // === Tables (GFM) ===
  Table,
  TableHead,
  TableBody,
  TableRow,
  TableCell {
    alignment: Alignment,
    is_header: bool,
  },

  // === Inline Elements ===
  /// Plain text content
  Text {
    content: String,
  },
  /// Emphasis (*text* or _text_)
  Emphasis,
  /// Strong emphasis (**text** or __text__)
  Strong,
  /// Strikethrough (~~text~~) - GFM
  Strikethrough,
  /// Inline code (not in span)
  Code {
    content: String,
  },
  /// Code span (`code`)
  CodeSpan {
    content: String,
  },
  /// Inline link or image reference
  Link {
    url: String,
    title: Option<String>,
    ref_type: ReferenceType,
  },
  /// Image
  Image {
    url: String,
    alt: String,
    title: Option<String>,
  },
  /// Autolink (`<url>`)
  AutoLink {
    url: String,
  },
  /// Hard line break (trailing spaces or \)
  HardBreak,
  /// Soft line break (single newline)
  SoftBreak,
  /// Raw inline HTML
  HtmlInline {
    content: String,
  },

  // === References ===
  LinkReference {
    label: String,
    ref_type: ReferenceType,
  },
  LinkDefinition {
    label: String,
    url: String,
    title: Option<String>,
  },
  FootnoteReference {
    label: String,
  },
  FootnoteDefinition {
    label: String,
  },

  // === GFM Extensions ===
  TaskListMarker {
    checked: bool,
  },
  Emoji {
    shortcode: String,
  },
  Mention {
    username: String,
  },
  IssueReference {
    number: u32,
  },

  // === Documentation Comments ===
  DocComment {
    style: DocStyle,
  },
  DocTag {
    name: String,
    content: Option<String>,
  },
  DocParam {
    name: String,
    param_type: Option<String>,
    description: Option<String>,
  },
  DocReturn {
    return_type: Option<String>,
    description: Option<String>,
  },
  DocThrows {
    exception_type: String,
    description: Option<String>,
  },
  DocExample {
    content: String,
  },
  DocSee {
    reference: String,
  },
  DocDeprecated {
    message: Option<String>,
  },
  DocSince {
    version: String,
  },
  DocAuthor {
    name: String,
  },
  DocVersion {
    version: String,
  },
  DocDescription {
    content: String,
  },
  DocType {
    type_expr: String,
  },
  DocProperty {
    name: String,
    prop_type: Option<String>,
    description: Option<String>,
  },
  DocCallback {
    name: String,
  },
  DocTypedef {
    name: String,
    type_expr: Option<String>,
  },

  // === Extended Markdown ===
  /// YAML/TOML frontmatter block
  Frontmatter {
    format: FrontmatterFormat,
    content: String,
  },
  /// Inline math ($...$)
  MathInline {
    content: String,
  },
  /// Block math ($$...$$)
  MathBlock {
    content: String,
  },
  /// Footnote definition [^label]: content
  Footnote {
    label: String,
  },
  /// Definition list container
  DefinitionList,
  /// Definition term
  DefinitionTerm,
  /// Definition description
  DefinitionDescription,
  /// Auto-detected URL (without angle brackets)
  AutoUrl {
    url: String,
  },

  // === Glagolica Extensions ===
  /// Alert blockquote (`> [!NOTE]`, `> [!TIP]`, etc.)
  Alert {
    alert_type: AlertType,
  },
  /// Steps container (`<steps>`)
  Steps,
  /// Individual step (`<step>`)
  Step,
  /// Table of contents placeholder (`<toc>` or `<toc />`)
  Toc,
  /// Tabbed code blocks container (`<tabs names="...">`)
  Tabs {
    names: Vec<String>,
  },
  /// Code block with extended attributes
  CodeBlockExt {
    language: Option<String>,
    highlight: Option<String>,
    plusdiff: Option<String>,
    minusdiff: Option<String>,
    linenumbers: bool,
  },
}

/// Frontmatter format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FrontmatterFormat {
  Yaml,
  Toml,
  Json,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_node_new() {
    let node = Node::new(NodeKind::Paragraph, Span::empty());
    assert!(node.is_leaf());
    assert_eq!(node.count_nodes(), 1);
  }

  #[test]
  fn test_node_with_children() {
    let child = Node::new(
      NodeKind::Text {
        content: "hello".into(),
      },
      Span::empty(),
    );
    let parent = Node::with_children(NodeKind::Paragraph, Span::empty(), vec![child]);
    assert!(!parent.is_leaf());
    assert_eq!(parent.count_nodes(), 2);
  }

  #[test]
  fn test_count_nested_nodes() {
    let leaf = Node::new(
      NodeKind::Text {
        content: "x".into(),
      },
      Span::empty(),
    );
    let mid = Node::with_children(NodeKind::Strong, Span::empty(), vec![leaf]);
    let root = Node::with_children(NodeKind::Paragraph, Span::empty(), vec![mid]);
    assert_eq!(root.count_nodes(), 3);
  }
}
