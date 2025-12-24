//! Link, image, and reference parsing.

use super::InlineParser;
use crate::ast::{Node, NodeKind, ReferenceType, Span};

impl<'a> InlineParser<'a> {
  /// Try to parse link `[text](url)` or image `![alt](url)`.
  pub fn try_link(&mut self, is_image: bool) -> Option<Node> {
    let start = self.pos;
    if is_image {
      self.pos += 1;
    }
    self.pos += 1; // skip [

    let text_end = self.find_bracket()?;
    let text = self.input[self.pos..text_end].to_string();
    self.pos = text_end + 1;

    // Try inline destination: (url "title")
    if self.try_inline_link(&text, start, is_image) {
      return self.build_inline_link(start, is_image);
    }

    // Try reference link from definitions
    if let Some(node) = self.try_reference_link(&text, start, is_image) {
      return Some(node);
    }

    self.pos = start;
    None
  }

  fn try_inline_link(&mut self, _text: &str, _start: usize, _is_image: bool) -> bool {
    self.pos < self.bytes.len() && self.bytes[self.pos] == b'('
  }

  fn build_inline_link(&mut self, start: usize, is_image: bool) -> Option<Node> {
    // Re-parse text position since we already advanced
    let text_start = start + if is_image { 2 } else { 1 };
    let bracket_pos = self.bytes[text_start..]
      .iter()
      .position(|&b| b == b']')
      .map(|p| text_start + p)?;
    let text = self.input[text_start..bracket_pos].to_string();

    self.pos += 1; // skip (
    let (url, title) = self.parse_dest()?;

    let children = InlineParser::new(&text, self.link_defs).parse();
    let kind = if is_image {
      NodeKind::Image {
        url,
        title,
        alt: text,
      }
    } else {
      NodeKind::Link {
        url,
        title,
        ref_type: ReferenceType::Full,
      }
    };

    Some(Node::with_children(
      kind,
      Span::new(start, self.pos, 0, 0),
      children,
    ))
  }

  fn try_reference_link(&self, text: &str, start: usize, is_image: bool) -> Option<Node> {
    let def = self
      .link_defs
      .iter()
      .find(|d| d.label.eq_ignore_ascii_case(text))?;
    let children = InlineParser::new(text, self.link_defs).parse();

    let kind = if is_image {
      NodeKind::Image {
        url: def.url.clone(),
        title: def.title.clone(),
        alt: text.to_string(),
      }
    } else {
      NodeKind::Link {
        url: def.url.clone(),
        title: def.title.clone(),
        ref_type: ReferenceType::Shortcut,
      }
    };

    Some(Node::with_children(
      kind,
      Span::new(start, self.pos, 0, 0),
      children,
    ))
  }

  /// Find matching closing bracket, handling nesting.
  pub fn find_bracket(&self) -> Option<usize> {
    let mut depth = 1;
    let mut i = self.pos;

    while i < self.bytes.len() {
      match self.bytes[i] {
        b'[' => depth += 1,
        b']' if depth == 1 => return Some(i),
        b']' => depth -= 1,
        b'\\' => i += 1, // skip escaped char
        _ => {}
      }
      i += 1;
    }
    None
  }

  /// Parse link destination and optional title.
  pub fn parse_dest(&mut self) -> Option<(String, Option<String>)> {
    self.skip_ws();
    let url = self.scan_url()?;
    self.skip_ws();
    let title = self.scan_title();
    self.skip_ws();

    if self.pos < self.bytes.len() && self.bytes[self.pos] == b')' {
      self.pos += 1;
      Some((url, title))
    } else {
      None
    }
  }

  /// Scan URL from destination (handles <> wrapped URLs).
  fn scan_url(&mut self) -> Option<String> {
    if self.bytes.get(self.pos) == Some(&b'<') {
      self.scan_angle_url()
    } else {
      self.scan_bare_url()
    }
  }

  fn scan_angle_url(&mut self) -> Option<String> {
    self.pos += 1;
    let end = self.bytes[self.pos..].iter().position(|&b| b == b'>')?;
    let url = self.input[self.pos..self.pos + end].to_string();
    self.pos += end + 1;
    Some(url)
  }

  fn scan_bare_url(&mut self) -> Option<String> {
    let start = self.pos;
    while self.pos < self.bytes.len()
      && !matches!(self.bytes[self.pos], b' ' | b'\t' | b')' | b'"' | b'\'')
    {
      self.pos += 1;
    }
    Some(self.input[start..self.pos].to_string())
  }

  /// Scan quoted title string.
  fn scan_title(&mut self) -> Option<String> {
    let delim = match self.bytes.get(self.pos)? {
      b'"' => b'"',
      b'\'' => b'\'',
      _ => return None,
    };
    self.pos += 1;

    let len = self.bytes[self.pos..].iter().position(|&b| b == delim)?;
    let title = self.input[self.pos..self.pos + len].to_string();
    self.pos += len + 1;
    Some(title)
  }
}
