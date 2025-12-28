//! Emphasis and strong parsing (*em*, **strong**, _em_, __strong__).

use super::InlineParser;
use crate::ast::{Node, NodeKind, Span};

impl<'a> InlineParser<'a> {
  /// Try to parse emphasis or strong.
  #[inline]
  pub fn try_emphasis(&mut self) -> Option<Node> {
    let start = self.pos;
    let delimiter = self.bytes[self.pos];
    let count = self.count_delimiters(delimiter);

    // Need at least one delimiter and content after
    if count == 0 || self.pos >= self.bytes.len() {
      self.pos = start;
      return None;
    }

    let content_start = self.pos;

    // Find closing delimiter - optimized search
    let close_pos = find_close_fast(&self.bytes[self.pos..], delimiter, count)?;
    let close_abs = content_start + close_pos;

    self.pos = close_abs + count;

    // Parse nested content recursively
    let children = InlineParser::new(&self.input[content_start..close_abs], self.link_defs).parse();

    let kind = if count >= 2 {
      NodeKind::Strong
    } else {
      NodeKind::Emphasis
    };

    Some(Node::with_children(
      kind,
      Span::new(start, self.pos, 0, 0),
      children,
    ))
  }

  /// Count consecutive delimiter characters and advance position.
  #[inline]
  fn count_delimiters(&mut self, delimiter: u8) -> usize {
    let start = self.pos;
    while self.pos < self.bytes.len() && self.bytes[self.pos] == delimiter {
      self.pos += 1;
    }
    self.pos - start
  }

  /// Try to parse inline code span (`code`).
  ///
  /// Handles variable backtick counts for nested code.
  #[inline]
  pub fn try_code_span(&mut self) -> Option<Node> {
    let start = self.pos;
    let backtick_count = self.count_delimiters(b'`');
    let content_start = self.pos;

    // Find exact matching backtick sequence
    let close_pos = match find_backticks_fast(&self.bytes[self.pos..], backtick_count) {
      Some(pos) => pos,
      None => {
        // Reset position on failure
        self.pos = start;
        return None;
      }
    };

    let content = self.input[content_start..content_start + close_pos]
      .trim()
      .to_string();

    self.pos = content_start + close_pos + backtick_count;
    Some(Node::new(
      NodeKind::CodeSpan { content },
      Span::new(start, self.pos, 0, 0),
    ))
  }

  /// Try to parse strikethrough (~~text~~).
  #[inline]
  pub fn try_strike(&mut self) -> Option<Node> {
    let start = self.pos;
    self.pos += 2; // Skip opening ~~

    // Fast search for closing ~~
    let remaining = &self.bytes[self.pos..];
    let close_pos = find_double_tilde(remaining)?;

    let children =
      InlineParser::new(&self.input[self.pos..self.pos + close_pos], self.link_defs).parse();

    self.pos += close_pos + 2;
    Some(Node::with_children(
      NodeKind::Strikethrough,
      Span::new(start, self.pos, 0, 0),
      children,
    ))
  }
}

/// Find closing delimiter sequence matching opening count.
///
/// Optimized for common cases (1-2 delimiters) with early exit.
#[inline]
fn find_close_fast(bytes: &[u8], delimiter: u8, count: usize) -> Option<usize> {
  let len = bytes.len();
  let mut i = 0;

  while i < len {
    // Fast scan for potential delimiter
    if bytes[i] == delimiter {
      let close_start = i;
      let mut c = 1;
      i += 1;

      // Count consecutive delimiters
      while i < len && bytes[i] == delimiter {
        c += 1;
        i += 1;
      }

      // Match found if we have at least the required count
      if c >= count {
        return Some(close_start);
      }
    } else {
      i += 1;
    }
  }
  None
}

/// Find matching backtick sequence for code span.
///
/// Must find exact count match (not more, not less).
#[inline]
fn find_backticks_fast(bytes: &[u8], count: usize) -> Option<usize> {
  let len = bytes.len();
  let mut i = 0;

  while i < len {
    if bytes[i] == b'`' {
      let close_start = i;
      let mut c = 1;
      i += 1;

      while i < len && bytes[i] == b'`' {
        c += 1;
        i += 1;
      }

      // Exact match required for code spans
      if c == count {
        return Some(close_start);
      }
    } else {
      i += 1;
    }
  }
  None
}

/// Find closing ~~ for strikethrough.
#[inline]
fn find_double_tilde(bytes: &[u8]) -> Option<usize> {
  let len = bytes.len();
  if len < 2 {
    return None;
  }

  let mut i = 0;
  while i < len - 1 {
    if bytes[i] == b'~' && bytes[i + 1] == b'~' {
      return Some(i);
    }
    i += 1;
  }
  None
}
