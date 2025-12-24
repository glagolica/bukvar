//! Code block parsing: fenced and indented.

use super::BlockParser;
use crate::ast::{Node, NodeKind, Span};

impl<'a, 'b> BlockParser<'a, 'b> {
  pub fn try_fenced_code(&mut self, line: usize, col: usize) -> Option<Node> {
    let fence_char = self.scanner.peek()?;
    if !matches!(fence_char, b'`' | b'~') {
      return None;
    }

    let start = self.scanner.pos();
    let fence_len = self.count_fence_chars(fence_char);

    if fence_len < 3 {
      self.scanner.set_pos(start);
      return None;
    }

    self.scanner.skip_whitespace_inline();
    let info = self.scan_line_content();
    self.scanner.consume(b'\n');

    let (language, info_str) = parse_code_info(&info);
    let code = self.scan_fenced_content(fence_char, fence_len);

    Some(Node::with_children(
      NodeKind::FencedCodeBlock {
        language,
        info: info_str,
      },
      Span::new(start, self.scanner.pos(), line, col),
      vec![Node::new(NodeKind::Text { content: code }, Span::empty())],
    ))
  }

  fn count_fence_chars(&mut self, ch: u8) -> usize {
    let mut count = 0;
    while self.scanner.consume(ch) {
      count += 1;
    }
    count
  }

  fn scan_fenced_content(&mut self, fence_char: u8, fence_len: usize) -> String {
    let start = self.scanner.pos();
    let mut end = start;

    loop {
      if self.scanner.is_eof() {
        break;
      }

      let line_start = self.scanner.pos();
      self.scanner.skip_whitespace_inline();

      if self.is_closing_fence(fence_char, fence_len) {
        self.scanner.skip_whitespace_inline();
        if self.scanner.is_eof() || self.scanner.check(b'\n') {
          self.scanner.consume(b'\n');
          break;
        }
      }

      self.scanner.set_pos(line_start);
      self.scanner.skip_line();
      end = self.scanner.pos();
    }

    self.scanner.slice(start, end).to_string()
  }

  fn is_closing_fence(&mut self, fence_char: u8, fence_len: usize) -> bool {
    let mut close_len = 0;
    while self.scanner.check(fence_char) {
      self.scanner.advance();
      close_len += 1;
    }
    close_len >= fence_len
  }

  pub fn try_math_block(&mut self, line: usize, col: usize) -> Option<Node> {
    if !self.scanner.check_str(b"$$") {
      return None;
    }

    let start = self.scanner.pos();
    self.scanner.advance_n(2);
    self.scanner.consume(b'\n');

    let content = self.scan_math_content()?;
    if content.is_none() {
      self.scanner.set_pos(start);
      return None;
    }

    Some(Node::new(
      NodeKind::MathBlock {
        content: content.unwrap(),
      },
      Span::new(start, self.scanner.pos(), line, col),
    ))
  }

  fn scan_math_content(&mut self) -> Option<Option<String>> {
    let content_start = self.scanner.pos();

    loop {
      if self.scanner.is_eof() {
        return Some(None);
      }
      if self.scanner.check_str(b"$$") {
        let content = self
          .scanner
          .slice(content_start, self.scanner.pos())
          .trim_end()
          .to_string();
        self.scanner.advance_n(2);
        self.scanner.consume(b'\n');
        return Some(Some(content));
      }
      self.scanner.advance();
    }
  }

  pub fn parse_indented_code(&mut self, line: usize, col: usize) -> Node {
    let start = self.scanner.pos();
    let content = self.collect_indented_lines();

    Node::with_children(
      NodeKind::IndentedCodeBlock,
      Span::new(start, self.scanner.pos(), line, col),
      vec![Node::new(NodeKind::Text { content }, Span::empty())],
    )
  }

  fn collect_indented_lines(&mut self) -> String {
    let mut content = String::new();

    loop {
      let indent = self.skip_indent(4);
      if indent < 4 && !self.scanner.check(b'\n') && !self.scanner.is_eof() {
        break;
      }

      self.append_line_chars(&mut content);
      content.push('\n');

      if !self.scanner.consume(b'\n') {
        break;
      }
    }

    content
  }

  fn skip_indent(&mut self, max: usize) -> usize {
    let mut indent = 0;
    while indent < max && (self.scanner.consume(b' ') || self.scanner.consume(b'\t')) {
      indent += 1;
    }
    indent
  }

  fn append_line_chars(&mut self, content: &mut String) {
    while !self.scanner.is_eof() && !self.scanner.check(b'\n') {
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
  }
}

fn parse_code_info(info: &str) -> (Option<String>, Option<String>) {
  if info.is_empty() {
    return (None, None);
  }

  let mut parts = info.splitn(2, char::is_whitespace);
  let lang = parts.next().map(String::from);
  let extra = parts.next().map(String::from);
  (lang, extra)
}
