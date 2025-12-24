//! Source location tracking for AST nodes.
//!
//! [`Span`] tracks where in the source text a node originated,
//! enabling error messages and source mapping.

/// Represents a location range in source text.
///
/// Tracks byte offsets (`start`, `end`) and human-readable
/// position (`line`, `column`) for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
  pub start: usize,  // Byte offset start
  pub end: usize,    // Byte offset end (exclusive)
  pub line: usize,   // 1-indexed line number
  pub column: usize, // 1-indexed column number
}

impl Span {
  /// Create a new span with the given positions.
  #[inline]
  pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
    Self {
      start,
      end,
      line,
      column,
    }
  }

  /// Create an empty span at position 0.
  #[inline]
  pub fn empty() -> Self {
    Self::default()
  }

  /// Merge two spans into one covering both ranges.
  /// Useful for combining tokens into a larger node.
  #[inline]
  #[allow(dead_code)] // Part of public API
  pub fn merge(self, other: Self) -> Self {
    Self {
      start: self.start.min(other.start),
      end: self.end.max(other.end),
      line: self.line.min(other.line),
      column: if self.line <= other.line {
        self.column
      } else {
        other.column
      },
    }
  }

  /// Get the byte length of this span.
  #[inline]
  #[allow(dead_code)]
  pub fn len(&self) -> usize {
    self.end.saturating_sub(self.start)
  }

  /// Check if span covers zero bytes.
  #[inline]
  #[allow(dead_code)]
  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_span_new() {
    let span = Span::new(10, 20, 1, 5);
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 20);
    assert_eq!(span.line, 1);
    assert_eq!(span.column, 5);
  }

  #[test]
  fn test_span_empty() {
    let span = Span::empty();
    assert!(span.is_empty());
    assert_eq!(span.len(), 0);
  }

  #[test]
  fn test_span_merge() {
    let a = Span::new(10, 20, 1, 5);
    let b = Span::new(15, 30, 2, 1);
    let merged = a.merge(b);
    assert_eq!(merged.start, 10);
    assert_eq!(merged.end, 30);
    assert_eq!(merged.line, 1);
  }

  #[test]
  fn test_span_len() {
    let span = Span::new(10, 25, 1, 1);
    assert_eq!(span.len(), 15);
  }
}
