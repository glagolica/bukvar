//! Special inline elements: math, footnotes, autolinks, escapes.

use super::InlineParser;
use crate::ast::{Node, NodeKind, ReferenceType, Span};

impl<'a> InlineParser<'a> {
  /// Try to parse inline math $...$ or $$...$$
  pub fn try_math(&mut self) -> Option<Node> {
    let start = self.pos;
    let is_block = self.peek_at(1) == Some(b'$');

    if is_block {
      self.try_math_block(start)
    } else {
      self.try_math_inline(start)
    }
  }

  fn try_math_block(&mut self, start: usize) -> Option<Node> {
    self.pos += 2;
    let content_start = self.pos;
    let end = self.input[self.pos..].find("$$")?;
    let content = self.input[content_start..content_start + end].to_string();
    self.pos = content_start + end + 2;
    Some(Node::new(
      NodeKind::MathBlock { content },
      Span::new(start, self.pos, 0, 0),
    ))
  }

  fn try_math_inline(&mut self, start: usize) -> Option<Node> {
    self.pos += 1;
    let content_start = self.pos;

    while self.pos < self.bytes.len() {
      if self.bytes[self.pos] == b'$' && !self.is_escaped() {
        let content = self.input[content_start..self.pos].to_string();
        self.pos += 1;
        return Some(Node::new(
          NodeKind::MathInline { content },
          Span::new(start, self.pos, 0, 0),
        ));
      }
      self.pos += 1;
    }

    self.pos = start;
    None
  }

  fn is_escaped(&self) -> bool {
    self.pos > 0 && self.bytes[self.pos - 1] == b'\\'
  }

  /// Try to parse footnote reference [^label]
  pub fn try_footnote_ref(&mut self) -> Option<Node> {
    let start = self.pos;
    self.pos += 2; // skip [^

    let label_end = self.bytes[self.pos..].iter().position(|&b| b == b']')?;
    let label = self.input[self.pos..self.pos + label_end].to_string();
    self.pos += label_end + 1;

    Some(Node::new(
      NodeKind::FootnoteReference { label },
      Span::new(start, self.pos, 0, 0),
    ))
  }

  /// Check if we're at start of a URL (for auto-linking)
  pub fn check_autourl(&self) -> bool {
    let rest = &self.input[self.pos..];
    rest.starts_with("http://") || rest.starts_with("https://")
  }

  /// Try to parse auto-detected URL
  pub fn try_autourl(&mut self) -> Option<Node> {
    let start = self.pos;
    while self.pos < self.bytes.len() && !is_url_terminator(self.bytes[self.pos]) {
      self.pos += 1;
    }
    let url = self.input[start..self.pos].to_string();
    Some(Node::new(
      NodeKind::AutoUrl { url },
      Span::new(start, self.pos, 0, 0),
    ))
  }

  /// Try to parse autolink (`<url>` or `<email>`).
  pub fn try_autolink(&mut self) -> Option<Node> {
    let start = self.pos;
    self.pos += 1; // skip <

    let end = self.bytes[self.pos..].iter().position(|&b| b == b'>')?;
    let url = &self.input[self.pos..self.pos + end];
    self.pos += end + 1;

    if !is_valid_autolink(url) {
      self.pos = start;
      return None;
    }

    let full_url = normalize_autolink(url);
    Some(Node::new(
      NodeKind::Link {
        url: full_url,
        title: None,
        ref_type: ReferenceType::Full,
      },
      Span::new(start, self.pos, 0, 0),
    ))
  }

  /// Try to parse backslash escape.
  pub fn try_escape(&mut self) -> Option<Node> {
    let start = self.pos;
    self.pos += 1;

    if self.pos < self.bytes.len() && is_escapable(self.bytes[self.pos]) {
      let content = (self.bytes[self.pos] as char).to_string();
      self.pos += 1;
      return Some(Node::new(
        NodeKind::Text { content },
        Span::new(start, self.pos, 0, 0),
      ));
    }

    self.pos = start;
    None
  }
}

#[inline(always)]
fn is_url_terminator(b: u8) -> bool {
  matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b')' | b']' | b'>')
}

#[inline(always)]
fn is_valid_autolink(url: &str) -> bool {
  url.contains('@') || url.starts_with("http") || url.starts_with("mailto:")
}

fn normalize_autolink(url: &str) -> String {
  if url.contains('@') && !url.starts_with("mailto:") {
    format!("mailto:{}", url)
  } else {
    url.to_string()
  }
}

#[inline(always)]
fn is_escapable(b: u8) -> bool {
  b"\\`*_{}[]()#+-.!|<>~".contains(&b)
}
