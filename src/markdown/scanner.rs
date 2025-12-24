//! Low-level byte scanner for parsing.

/// Scanner for byte-level parsing with position tracking.
pub struct Scanner<'a> {
  input: &'a str,  // Original string (for slicing)
  bytes: &'a [u8], // Byte view for fast access
  pos: usize,      // Current byte position
  line: usize,     // Current line (1-indexed)
  column: usize,   // Current column (1-indexed)
}

impl<'a> Scanner<'a> {
  /// Create a new scanner for the input string.
  #[inline]
  pub fn new(input: &'a str) -> Self {
    Self {
      input,
      bytes: input.as_bytes(),
      pos: 0,
      line: 1,
      column: 1,
    }
  }

  /// Reset scanner to the beginning.
  #[inline]
  pub fn reset(&mut self) {
    self.pos = 0;
    self.line = 1;
    self.column = 1;
  }

  // === Position Accessors ===

  #[inline(always)]
  pub fn pos(&self) -> usize {
    self.pos
  }

  #[inline(always)]
  pub fn set_pos(&mut self, pos: usize) {
    self.pos = pos;
  }

  #[inline(always)]
  pub fn line(&self) -> usize {
    self.line
  }

  #[inline(always)]
  pub fn column(&self) -> usize {
    self.column
  }

  #[inline(always)]
  pub fn is_eof(&self) -> bool {
    self.pos >= self.bytes.len()
  }

  /// Get the total length of input.
  #[inline(always)]
  #[allow(dead_code)]
  pub fn len(&self) -> usize {
    self.bytes.len()
  }

  // === Peek & Check ===

  /// Peek at current byte without consuming.
  #[inline(always)]
  pub fn peek(&self) -> Option<u8> {
    // SAFETY: bounds check via get()
    self.bytes.get(self.pos).copied()
  }

  /// Peek at byte at offset from current position.
  #[inline(always)]
  pub fn peek_at(&self, offset: usize) -> Option<u8> {
    self.bytes.get(self.pos + offset).copied()
  }

  /// Check if current byte matches expected.
  #[inline(always)]
  pub fn check(&self, expected: u8) -> bool {
    self.peek() == Some(expected)
  }

  /// Check if remaining input starts with expected bytes.
  #[inline]
  #[allow(dead_code)]
  pub fn check_str(&self, expected: &[u8]) -> bool {
    self
      .bytes
      .get(self.pos..)
      .is_some_and(|s| s.starts_with(expected))
  }

  // === Advance & Consume ===

  /// Advance one byte, tracking line/column.
  #[inline(always)]
  pub fn advance(&mut self) {
    if self.pos < self.bytes.len() {
      if self.bytes[self.pos] == b'\n' {
        self.line += 1;
        self.column = 1;
      } else {
        self.column += 1;
      }
      self.pos += 1;
    }
  }

  /// Advance n bytes (optimized for small n).
  #[inline]
  #[allow(dead_code)]
  pub fn advance_n(&mut self, n: usize) {
    for _ in 0..n {
      self.advance();
    }
  }

  /// Consume byte if it matches expected.
  #[inline(always)]
  pub fn consume(&mut self, expected: u8) -> bool {
    if self.check(expected) {
      self.advance();
      true
    } else {
      false
    }
  }

  // === Skip (Optimized) ===

  /// Skip inline whitespace (space and tab only).
  /// Uses unrolled loop for better performance on typical indentation.
  #[inline]
  pub fn skip_whitespace_inline(&mut self) {
    // Fast path: check common case of no whitespace
    if self.pos >= self.bytes.len() || !matches!(self.bytes[self.pos], b' ' | b'\t') {
      return;
    }

    // Unrolled loop for typical indentation (1-8 spaces)
    while self.pos < self.bytes.len() {
      match self.bytes[self.pos] {
        b' ' | b'\t' => {
          self.column += 1;
          self.pos += 1;
        }
        _ => break,
      }
    }
  }

  /// Skip to end of current line and consume newline.
  #[inline]
  pub fn skip_line(&mut self) {
    // Fast scan for newline using find_byte pattern
    if let Some(rel_pos) = self.find_byte_in_remaining(b'\n') {
      self.pos += rel_pos;
      self.column += rel_pos;
      // Consume the newline
      self.pos += 1;
      self.line += 1;
      self.column = 1;
    } else {
      // No newline found, go to EOF
      let remaining = self.bytes.len() - self.pos;
      self.column += remaining;
      self.pos = self.bytes.len();
    }
  }

  /// Skip consecutive blank lines efficiently.
  #[inline]
  pub fn skip_blank_lines(&mut self) {
    loop {
      let start = self.pos;
      self.skip_whitespace_inline();
      if !self.consume(b'\n') {
        self.pos = start;
        break;
      }
    }
  }

  // === Scan (Optimized) ===

  /// Find byte in remaining input, returns relative position.
  #[inline]
  fn find_byte_in_remaining(&self, needle: u8) -> Option<usize> {
    let remaining = &self.bytes[self.pos..];
    // Manual search - typically faster than iterator for small searches
    remaining.iter().position(|&b| b == needle)
  }

  /// Scan until delimiter (not including it), return content.
  #[inline]
  pub fn scan_until(&mut self, delim: u8) -> Option<String> {
    let start = self.pos;

    // Fast scan for delimiter or newline
    while self.pos < self.bytes.len() {
      let b = self.bytes[self.pos];
      if b == delim {
        return Some(self.input[start..self.pos].to_string());
      }
      if b == b'\n' {
        return None;
      }
      self.column += 1;
      self.pos += 1;
    }
    None
  }

  /// Scan non-whitespace characters.
  #[inline]
  pub fn scan_non_whitespace(&mut self) -> String {
    let start = self.pos;
    while self.pos < self.bytes.len() && !self.bytes[self.pos].is_ascii_whitespace() {
      self.column += 1;
      self.pos += 1;
    }
    self.input[start..self.pos].to_string()
  }

  /// Get a slice of the input between positions.
  #[inline(always)]
  pub fn slice(&self, start: usize, end: usize) -> &'a str {
    let end = end.min(self.input.len());
    // SAFETY: start/end come from our byte positions which respect char boundaries
    &self.input[start..end]
  }

  /// Get remaining unparsed input.
  #[inline(always)]
  #[allow(dead_code)]
  pub fn remaining(&self) -> &'a str {
    &self.input[self.pos..]
  }

  /// Scan and return current line, advancing past it.
  /// Optimized for the common case of scanning entire lines.
  #[inline]
  pub fn scan_line(&mut self) -> &'a str {
    let start = self.pos;

    // Fast scan for newline
    if let Some(rel_pos) = self.find_byte_in_remaining(b'\n') {
      let end = self.pos + rel_pos;
      self.pos = end + 1; // Skip past newline
      self.line += 1;
      self.column = 1;
      &self.input[start..end]
    } else {
      // No newline - return rest of input
      let end = self.bytes.len();
      self.pos = end;
      &self.input[start..end]
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_scanner_new() {
    let s = Scanner::new("hello");
    assert_eq!(s.pos(), 0);
    assert_eq!(s.line(), 1);
    assert!(!s.is_eof());
  }

  #[test]
  fn test_peek_and_advance() {
    let mut s = Scanner::new("ab");
    assert_eq!(s.peek(), Some(b'a'));
    s.advance();
    assert_eq!(s.peek(), Some(b'b'));
    s.advance();
    assert!(s.is_eof());
  }

  #[test]
  fn test_consume() {
    let mut s = Scanner::new("abc");
    assert!(s.consume(b'a'));
    assert!(!s.consume(b'a'));
    assert!(s.consume(b'b'));
  }

  #[test]
  fn test_line_tracking() {
    let mut s = Scanner::new("a\nb");
    s.advance(); // a
    assert_eq!(s.line(), 1);
    s.advance(); // \n
    assert_eq!(s.line(), 2);
  }
}
