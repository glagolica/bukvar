//! Inline-level markdown parsing (emphasis, links, code spans, etc).

mod emphasis;
mod links;
mod special;

use super::LinkDef;
use crate::ast::{Node, NodeKind, Span};

/// Returns true if byte might start a special inline element.
#[inline(always)]
fn is_special_char(b: u8) -> bool {
  matches!(
    b,
    b'*' | b'_' | b'`' | b'[' | b'!' | b'~' | b'<' | b'\\' | b'$' | b'h'
  )
}

/// Parser for inline elements within block content.
pub struct InlineParser<'a> {
  input: &'a str,
  bytes: &'a [u8],
  pos: usize,
  link_defs: &'a [LinkDef],
}

impl<'a> InlineParser<'a> {
  /// Create a new inline parser.
  #[inline]
  pub fn new(input: &'a str, link_defs: &'a [LinkDef]) -> Self {
    Self {
      input,
      bytes: input.as_bytes(),
      pos: 0,
      link_defs,
    }
  }

  /// Parse inline content and return nodes.
  ///
  /// Scans the input accumulating plain text, and when a special
  /// character is found, attempts to parse it as an inline element.
  #[inline]
  pub fn parse(&mut self) -> Vec<Node> {
    // Pre-allocate with reasonable estimate (1 node per 50 chars)
    let mut nodes = Vec::with_capacity((self.bytes.len() / 50).max(4));
    let mut text_start = self.pos;

    while self.pos < self.bytes.len() {
      let b = self.bytes[self.pos];

      // Fast path: skip non-special characters quickly
      if !is_special_char(b) {
        self.pos += 1;
        continue;
      }

      // Potential special character - try to parse it
      if let Some(node) = self.try_special() {
        self.flush_text(text_start, &mut nodes);
        nodes.push(node);
        text_start = self.pos;
      } else {
        self.pos += 1;
      }
    }

    self.flush_text(text_start, &mut nodes);
    nodes
  }

  /// Flush accumulated text as a text node.
  #[inline]
  fn flush_text(&self, start: usize, nodes: &mut Vec<Node>) {
    if start < self.pos {
      nodes.push(self.text_node(start, self.pos));
    }
  }

  /// Try to parse a special inline element at current position.
  #[inline]
  fn try_special(&mut self) -> Option<Node> {
    // SAFETY: We know pos < bytes.len() from the caller's while condition
    let ch = self.bytes[self.pos];

    match ch {
      b'*' | b'_' => self.try_emphasis(),
      b'`' => self.try_code_span(),
      b'[' => self.try_link_or_footnote(),
      b'!' if self.peek_at(1) == Some(b'[') => self.try_link(true),
      b'~' if self.peek_at(1) == Some(b'~') => self.try_strike(),
      b'<' => self.try_autolink(),
      b'\\' => self.try_escape(),
      b'$' => self.try_math(),
      b'h' if self.check_autourl() => self.try_autourl(),
      _ => None,
    }
  }

  /// Try link or footnote reference based on next char.
  #[inline]
  fn try_link_or_footnote(&mut self) -> Option<Node> {
    if self.peek_at(1) == Some(b'^') {
      self.try_footnote_ref()
    } else {
      self.try_link(false)
    }
  }

  /// Create a text node from a slice range.
  #[inline]
  fn text_node(&self, s: usize, e: usize) -> Node {
    Node::new(
      NodeKind::Text {
        content: self.input[s..e].to_string(),
      },
      Span::new(s, e, 0, 0),
    )
  }

  /// Peek at byte at offset from current position.
  #[inline(always)]
  pub(crate) fn peek_at(&self, offset: usize) -> Option<u8> {
    self.bytes.get(self.pos + offset).copied()
  }

  /// Skip inline whitespace (space and tab).
  #[inline(always)]
  pub(crate) fn skip_ws(&mut self) {
    while self.pos < self.bytes.len() && matches!(self.bytes[self.pos], b' ' | b'\t') {
      self.pos += 1;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_plain_text() {
    let nodes = InlineParser::new("hello world", &[]).parse();
    assert_eq!(nodes.len(), 1);
  }

  #[test]
  fn test_emphasis() {
    let nodes = InlineParser::new("*italic*", &[]).parse();
    assert!(!nodes.is_empty());
  }

  #[test]
  fn test_strong() {
    let nodes = InlineParser::new("**bold**", &[]).parse();
    assert!(!nodes.is_empty());
  }

  #[test]
  fn test_code_span() {
    let nodes = InlineParser::new("`code`", &[]).parse();
    assert!(!nodes.is_empty());
  }

  #[test]
  fn test_strikethrough() {
    let nodes = InlineParser::new("~~deleted~~", &[]).parse();
    assert!(!nodes.is_empty());
  }

  #[test]
  fn test_inline_math() {
    let nodes = InlineParser::new("Equation $x^2$", &[]).parse();
    assert!(nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::MathInline { .. })));
  }

  #[test]
  fn test_footnote_ref() {
    let nodes = InlineParser::new("Text[^1]", &[]).parse();
    assert!(nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::FootnoteReference { .. })));
  }

  #[test]
  fn test_autourl() {
    let nodes = InlineParser::new("Visit https://example.com today", &[]).parse();
    assert!(nodes
      .iter()
      .any(|n| matches!(&n.kind, NodeKind::AutoUrl { .. })));
  }
}
