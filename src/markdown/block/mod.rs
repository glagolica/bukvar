//! Block-level markdown parsing (headings, lists, code blocks, etc).

mod code;
mod container;
mod leaf;

use super::{InlineParser, LinkDef, Scanner};
use crate::ast::Node;

/// Parser for block-level elements.
pub struct BlockParser<'a, 'b> {
  scanner: &'a mut Scanner<'b>,
  link_defs: &'a [LinkDef],
}

impl<'a, 'b> BlockParser<'a, 'b> {
  #[inline]
  pub fn new(scanner: &'a mut Scanner<'b>, link_defs: &'a [LinkDef]) -> Self {
    Self { scanner, link_defs }
  }

  /// Parse all blocks until EOF.
  #[inline]
  pub fn parse_blocks(&mut self) -> Vec<Node> {
    let mut nodes = Vec::with_capacity(32);

    while !self.scanner.is_eof() {
      self.scanner.skip_blank_lines();
      if self.scanner.is_eof() {
        break;
      }
      if let Some(node) = self.parse_block() {
        nodes.push(node);
      }
    }
    nodes
  }

  /// Parse a single block element.
  #[inline]
  pub fn parse_block(&mut self) -> Option<Node> {
    let start_pos = self.scanner.pos();
    let start_line = self.scanner.line();
    let start_col = self.scanner.column();

    self.scanner.skip_whitespace_inline();
    let indent = self.scanner.pos() - start_pos;

    // Fast path: check first character to narrow down possibilities
    let first_char = self.scanner.peek();

    // Try block types in precedence order with character-based dispatch
    match first_char {
      // Thematic breaks: ---, ***, ___
      Some(b'-' | b'*' | b'_') => {
        if let Some(node) = self.try_thematic_break(start_line, start_col) {
          return Some(node);
        }
        // Fall through to check list or emphasis start
      }
      // Headings: # ## ### etc
      Some(b'#') => {
        if let Some(node) = self.try_atx_heading(start_line, start_col) {
          return Some(node);
        }
      }
      // Fenced code: ``` or ~~~
      Some(b'`' | b'~') => {
        if let Some(node) = self.try_fenced_code(start_line, start_col) {
          return Some(node);
        }
      }
      // Math blocks: $$
      Some(b'$') => {
        if let Some(node) = self.try_math_block(start_line, start_col) {
          return Some(node);
        }
      }
      // Blockquotes: >
      Some(b'>') => {
        return Some(self.parse_blockquote(start_line, start_col));
      }
      _ => {}
    }

    // Check for lists (-, *, +)
    if let Some(node) = self.try_list(start_line, start_col) {
      return Some(node);
    }

    // HTML blocks
    if let Some(node) = self.try_html_block(start_line, start_col) {
      return Some(node);
    }

    // Tables (GFM)
    if let Some(node) = self.try_table(start_line, start_col) {
      return Some(node);
    }

    // Indented code blocks (4+ spaces)
    if let Some(node) = self.try_indented_code(indent, start_pos, start_line, start_col) {
      return Some(node);
    }

    // Definition lists
    self.scanner.set_pos(start_pos);
    if let Some(node) = self.try_definition_list(start_line, start_col) {
      return Some(node);
    }

    // Fall back to paragraph
    self.scanner.set_pos(start_pos);
    self.parse_paragraph(start_line, start_col)
  }

  #[inline]
  #[allow(dead_code)]
  fn try_blockquote(&mut self, line: usize, col: usize) -> Option<Node> {
    if self.scanner.check(b'>') {
      Some(self.parse_blockquote(line, col))
    } else {
      None
    }
  }

  #[inline]
  fn try_list(&mut self, _line: usize, _col: usize) -> Option<Node> {
    let ch = self.scanner.peek()?;
    if matches!(ch, b'-' | b'*' | b'+') && self.scanner.peek_at(1) == Some(b' ') {
      return Some(self.parse_list(false));
    }
    None
  }

  #[inline]
  fn try_html_block(&mut self, _line: usize, _col: usize) -> Option<Node> {
    None
  }

  #[inline]
  fn try_table(&mut self, _line: usize, _col: usize) -> Option<Node> {
    None
  }

  #[inline]
  fn try_indented_code(
    &mut self,
    indent: usize,
    _start_pos: usize,
    line: usize,
    col: usize,
  ) -> Option<Node> {
    if indent >= 4 {
      Some(self.parse_indented_code(line, col))
    } else {
      None
    }
  }

  #[inline]
  pub(crate) fn parse_inline(&self, text: &str) -> Vec<Node> {
    InlineParser::new(text, self.link_defs).parse()
  }

  #[inline]
  pub(crate) fn scan_line_content(&mut self) -> String {
    let start = self.scanner.pos();
    while !self.scanner.is_eof() && !self.scanner.check(b'\n') {
      self.scanner.advance();
    }
    self
      .scanner
      .slice(start, self.scanner.pos())
      .trim()
      .to_string()
  }
}
