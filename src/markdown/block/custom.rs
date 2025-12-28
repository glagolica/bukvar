//! Custom HTML elements: steps, toc, tabs.

use super::BlockParser;
use crate::ast::{Node, NodeKind, Span};

impl<'a, 'b> BlockParser<'a, 'b> {
  /// Try to parse custom Glagolica elements: `<steps>`, `<toc>`, `<tabs>`.
  pub fn try_custom_element(&mut self, line: usize, col: usize) -> Option<Node> {
    if !self.scanner.check(b'<') {
      return None;
    }

    let start = self.scanner.pos();

    // Try each custom element type
    if let Some(node) = self.try_toc(start, line, col) {
      return Some(node);
    }

    if let Some(node) = self.try_steps(start, line, col) {
      return Some(node);
    }

    if let Some(node) = self.try_tabs(start, line, col) {
      return Some(node);
    }

    None
  }

  fn try_toc(&mut self, start: usize, line: usize, col: usize) -> Option<Node> {
    self.scanner.set_pos(start);

    // Match <toc> or <toc /> or <toc/>
    if !self.scanner.check_str(b"<toc") {
      return None;
    }

    self.scanner.advance_n(4);
    self.scanner.skip_whitespace_inline();

    // Self-closing: <toc /> or <toc/>
    if self.scanner.consume(b'/') {
      if !self.scanner.consume(b'>') {
        self.scanner.set_pos(start);
        return None;
      }
      self.scanner.skip_whitespace_inline();
      self.scanner.consume(b'\n');
      return Some(Node::new(
        NodeKind::Toc,
        Span::new(start, self.scanner.pos(), line, col),
      ));
    }

    // Opening tag: <toc>
    if !self.scanner.consume(b'>') {
      self.scanner.set_pos(start);
      return None;
    }

    self.scanner.skip_whitespace_inline();
    self.scanner.consume(b'\n');

    Some(Node::new(
      NodeKind::Toc,
      Span::new(start, self.scanner.pos(), line, col),
    ))
  }

  fn try_steps(&mut self, start: usize, line: usize, col: usize) -> Option<Node> {
    self.scanner.set_pos(start);

    if !self.scanner.check_str(b"<steps>") && !self.scanner.check_str(b"<steps ") {
      return None;
    }

    // Skip <steps and find >
    self.scanner.advance_n(6);
    while !self.scanner.is_eof() && !self.scanner.check(b'>') {
      self.scanner.advance();
    }
    if !self.scanner.consume(b'>') {
      self.scanner.set_pos(start);
      return None;
    }
    self.scanner.consume(b'\n');

    let mut steps = Vec::new();

    // Parse inner content looking for <step> elements
    while !self.scanner.is_eof() {
      self.scanner.skip_blank_lines();
      self.scanner.skip_whitespace_inline();

      // Check for closing </steps>
      if self.scanner.check_str(b"</steps>") {
        self.scanner.advance_n(8);
        self.scanner.consume(b'\n');
        break;
      }

      // Try to parse a <step> element
      if let Some(step) = self.try_step() {
        steps.push(step);
      } else {
        // Skip unknown content
        self.scanner.skip_line();
      }
    }

    Some(Node::with_children(
      NodeKind::Steps,
      Span::new(start, self.scanner.pos(), line, col),
      steps,
    ))
  }

  fn try_step(&mut self) -> Option<Node> {
    let start = self.scanner.pos();
    let line = self.scanner.line();
    let col = self.scanner.column();

    if !self.scanner.check_str(b"<step>") && !self.scanner.check_str(b"<step ") {
      return None;
    }

    // Skip <step and find >
    self.scanner.advance_n(5);
    while !self.scanner.is_eof() && !self.scanner.check(b'>') {
      self.scanner.advance();
    }
    if !self.scanner.consume(b'>') {
      self.scanner.set_pos(start);
      return None;
    }
    self.scanner.consume(b'\n');

    // Collect content until </step>
    let content = self.collect_until_close_tag(b"</step>");

    // Parse the inner content as markdown
    let mut inner = super::super::MarkdownParser::new(&content);
    let inner_doc = inner.parse();

    Some(Node::with_children(
      NodeKind::Step,
      Span::new(start, self.scanner.pos(), line, col),
      inner_doc.nodes,
    ))
  }

  fn try_tabs(&mut self, start: usize, line: usize, col: usize) -> Option<Node> {
    self.scanner.set_pos(start);

    if !self.scanner.check_str(b"<tabs") {
      return None;
    }

    self.scanner.advance_n(5);
    self.scanner.skip_whitespace_inline();

    // Parse names attribute
    let names = self.parse_tabs_names()?;

    // Find closing >
    while !self.scanner.is_eof() && !self.scanner.check(b'>') {
      self.scanner.advance();
    }
    if !self.scanner.consume(b'>') {
      self.scanner.set_pos(start);
      return None;
    }
    self.scanner.consume(b'\n');

    // Collect content until </tabs>
    let content = self.collect_until_close_tag(b"</tabs>");

    // Parse inner content (code blocks)
    let mut inner = super::super::MarkdownParser::new(&content);
    let inner_doc = inner.parse();

    Some(Node::with_children(
      NodeKind::Tabs { names },
      Span::new(start, self.scanner.pos(), line, col),
      inner_doc.nodes,
    ))
  }

  fn parse_tabs_names(&mut self) -> Option<Vec<String>> {
    // Look for names="..."
    if !self.scanner.check_str(b"names=") {
      return Some(Vec::new());
    }

    self.scanner.advance_n(6);

    let quote = self.scanner.peek()?;
    if quote != b'"' && quote != b'\'' {
      return Some(Vec::new());
    }
    self.scanner.advance();

    let start = self.scanner.pos();
    while !self.scanner.is_eof() && self.scanner.peek() != Some(quote) {
      self.scanner.advance();
    }

    let names_str = self.scanner.slice(start, self.scanner.pos());
    self.scanner.advance(); // skip closing quote

    let names: Vec<String> = names_str
      .split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    Some(names)
  }

  fn collect_until_close_tag(&mut self, close_tag: &[u8]) -> String {
    let mut content = String::new();
    let mut depth = 1;

    // Determine the open tag from close tag (e.g., </step> -> <step)
    let open_tag: Vec<u8> = {
      let mut tag = vec![b'<'];
      tag.extend_from_slice(&close_tag[2..close_tag.len() - 1]);
      tag
    };

    while !self.scanner.is_eof() {
      // Check for closing tag at depth 1
      if depth == 1 {
        let pos = self.scanner.pos();
        self.scanner.skip_whitespace_inline();
        if self.scanner.check_str(close_tag) {
          self.scanner.advance_n(close_tag.len());
          self.scanner.consume(b'\n');
          break;
        }
        self.scanner.set_pos(pos);
      }

      // Track nested tags
      if self.scanner.check_str(&open_tag) {
        depth += 1;
      } else if self.scanner.check_str(close_tag) {
        depth -= 1;
        if depth == 0 {
          self.scanner.advance_n(close_tag.len());
          self.scanner.consume(b'\n');
          break;
        }
      }

      // Append current character
      if let Some(ch) = self
        .scanner
        .slice(self.scanner.pos(), self.scanner.pos() + 1)
        .chars()
        .next()
      {
        content.push(ch);
      }
      self.scanner.advance();
    }

    content
  }
}
