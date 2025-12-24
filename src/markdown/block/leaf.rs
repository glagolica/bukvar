//! Leaf block elements: headings, thematic breaks, paragraphs.

use super::BlockParser;
use crate::ast::{Node, NodeKind, Span};

impl<'a, 'b> BlockParser<'a, 'b> {
  pub fn try_thematic_break(&mut self, line: usize, col: usize) -> Option<Node> {
    let start = self.scanner.pos();
    let ch = self.scanner.peek()?;

    if !matches!(ch, b'-' | b'*' | b'_') {
      return None;
    }

    let count = self.count_thematic_chars(ch);
    if count < 3 {
      self.scanner.set_pos(start);
      return None;
    }

    self.scanner.consume(b'\n');
    Some(Node::new(
      NodeKind::ThematicBreak,
      Span::new(start, self.scanner.pos(), line, col),
    ))
  }

  fn count_thematic_chars(&mut self, ch: u8) -> usize {
    let mut count = 0;
    while !self.scanner.is_eof() && !self.scanner.check(b'\n') {
      if self.scanner.check(ch) {
        count += 1;
        self.scanner.advance();
      } else if matches!(self.scanner.peek(), Some(b' ' | b'\t')) {
        self.scanner.advance();
      } else {
        return 0; // Invalid character
      }
    }
    count
  }

  pub fn try_atx_heading(&mut self, line: usize, col: usize) -> Option<Node> {
    if !self.scanner.check(b'#') {
      return None;
    }

    let start = self.scanner.pos();
    let level = self.count_hashes();

    if level == 0 || !self.is_valid_heading_start() {
      self.scanner.set_pos(start);
      return None;
    }

    self.scanner.skip_whitespace_inline();
    let content = self.scan_heading_content();
    self.scanner.consume(b'\n');

    let (text, id) = extract_heading_id(&content);
    let inline = self.parse_inline(text);

    Some(Node::with_children(
      NodeKind::Heading { level, id },
      Span::new(start, self.scanner.pos(), line, col),
      inline,
    ))
  }

  fn count_hashes(&mut self) -> u8 {
    let mut level = 0u8;
    while self.scanner.consume(b'#') && level < 6 {
      level += 1;
    }
    level
  }

  fn is_valid_heading_start(&self) -> bool {
    self.scanner.is_eof()
      || self.scanner.check(b' ')
      || self.scanner.check(b'\t')
      || self.scanner.check(b'\n')
  }

  fn scan_heading_content(&mut self) -> String {
    let start = self.scanner.pos();
    let mut end = start;

    while !self.scanner.is_eof() && !self.scanner.check(b'\n') {
      if !self.scanner.check(b'#') && !matches!(self.scanner.peek(), Some(b' ' | b'\t')) {
        self.scanner.advance();
        end = self.scanner.pos();
      } else {
        self.scanner.advance();
      }
    }

    self.scanner.slice(start, end).trim().to_string()
  }

  pub fn parse_paragraph(&mut self, line: usize, col: usize) -> Option<Node> {
    let start = self.scanner.pos();
    let content = self.scan_line_content();
    self.scanner.consume(b'\n');

    if content.trim().is_empty() {
      return None;
    }

    let inline = self.parse_inline(&content);
    Some(Node::with_children(
      NodeKind::Paragraph,
      Span::new(start, self.scanner.pos(), line, col),
      inline,
    ))
  }

  pub fn try_definition_list(&mut self, line: usize, col: usize) -> Option<Node> {
    let start = self.scanner.pos();
    let term_content = self.scan_line_content();

    if term_content.trim().is_empty() {
      return None;
    }
    self.scanner.consume(b'\n');

    if !self.is_definition_marker() {
      self.scanner.set_pos(start);
      return None;
    }

    self.skip_definition_marker();
    let items = self.collect_definition_items(&term_content, start, line, col);

    Some(Node::with_children(
      NodeKind::DefinitionList,
      Span::new(start, self.scanner.pos(), line, col),
      items,
    ))
  }

  fn is_definition_marker(&self) -> bool {
    self.scanner.check(b':')
  }

  fn skip_definition_marker(&mut self) {
    self.scanner.advance(); // skip ':'
    if matches!(self.scanner.peek(), Some(b' ' | b'\t')) {
      self.scanner.advance();
    }
  }

  fn collect_definition_items(
    &mut self,
    term_content: &str,
    start: usize,
    line: usize,
    col: usize,
  ) -> Vec<Node> {
    let mut items = Vec::new();

    // Add term node
    let term_inline = self.parse_inline(term_content);
    items.push(Node::with_children(
      NodeKind::DefinitionTerm,
      Span::new(start, self.scanner.pos(), line, col),
      term_inline,
    ));

    // Parse descriptions
    loop {
      let desc_start = self.scanner.pos();
      let desc_line = self.scanner.line();
      let desc_col = self.scanner.column();
      let desc_content = self.scan_line_content();
      self.scanner.consume(b'\n');

      let desc_inline = self.parse_inline(&desc_content);
      items.push(Node::with_children(
        NodeKind::DefinitionDescription,
        Span::new(desc_start, self.scanner.pos(), desc_line, desc_col),
        desc_inline,
      ));

      if !self.is_definition_marker() {
        break;
      }
      self.skip_definition_marker();
    }

    items
  }
}

fn extract_heading_id(content: &str) -> (&str, Option<String>) {
  content
    .rfind("{#")
    .filter(|_| content.ends_with('}'))
    .map(|pos| {
      let id = content[pos + 2..content.len() - 1].to_string();
      (content[..pos].trim(), Some(id))
    })
    .unwrap_or((content, None))
}
