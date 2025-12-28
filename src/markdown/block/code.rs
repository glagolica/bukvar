//! Code block parsing: fenced and indented.

use super::BlockParser;
use crate::ast::{Node, NodeKind, Span};

/// Parsed code block attributes from the info string.
struct CodeBlockAttrs {
  language: Option<String>,
  highlight: Option<String>,
  plusdiff: Option<String>,
  minusdiff: Option<String>,
  linenumbers: bool,
}

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

    let attrs = parse_code_attrs(&info);
    let code = self.scan_fenced_content(fence_char, fence_len);

    // Use CodeBlockExt if any extended attributes are present
    let kind = if attrs.highlight.is_some()
      || attrs.plusdiff.is_some()
      || attrs.minusdiff.is_some()
      || attrs.linenumbers
    {
      NodeKind::CodeBlockExt {
        language: attrs.language,
        highlight: attrs.highlight,
        plusdiff: attrs.plusdiff,
        minusdiff: attrs.minusdiff,
        linenumbers: attrs.linenumbers,
      }
    } else {
      NodeKind::FencedCodeBlock {
        language: attrs.language,
        info: None,
      }
    };

    Some(Node::with_children(
      kind,
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

fn parse_code_attrs(info: &str) -> CodeBlockAttrs {
  let info = info.trim();
  if info.is_empty() {
    return CodeBlockAttrs {
      language: None,
      highlight: None,
      plusdiff: None,
      minusdiff: None,
      linenumbers: false,
    };
  }

  let mut language = None;
  let mut highlight = None;
  let mut plusdiff = None;
  let mut minusdiff = None;
  let mut linenumbers = false;

  let mut chars = info.chars().peekable();
  let mut current = String::new();

  // First token is the language (if not an attribute)
  while let Some(&ch) = chars.peek() {
    if ch.is_whitespace() {
      break;
    }
    current.push(ch);
    chars.next();
  }

  if !current.is_empty() && !current.contains('=') {
    language = Some(current.clone());
    current.clear();
  }

  // Parse remaining attributes
  while chars.peek().is_some() {
    // Skip whitespace
    while let Some(&ch) = chars.peek() {
      if !ch.is_whitespace() {
        break;
      }
      chars.next();
    }

    // Read attribute name
    let mut attr_name = String::new();
    while let Some(&ch) = chars.peek() {
      if ch == '=' || ch.is_whitespace() {
        break;
      }
      attr_name.push(ch);
      chars.next();
    }

    if attr_name.is_empty() {
      break;
    }

    // Check for boolean attribute (no =)
    if chars.peek() != Some(&'=') {
      if attr_name.to_lowercase() == "linenumbers" {
        linenumbers = true;
      }
      continue;
    }

    chars.next(); // skip =

    // Read attribute value
    let mut attr_value = String::new();
    let quote_char = chars.peek().copied();

    if quote_char == Some('"') || quote_char == Some('\'') {
      chars.next(); // skip opening quote
      while let Some(&ch) = chars.peek() {
        if ch == quote_char.unwrap() {
          chars.next(); // skip closing quote
          break;
        }
        attr_value.push(ch);
        chars.next();
      }
    } else {
      // Unquoted value
      while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
          break;
        }
        attr_value.push(ch);
        chars.next();
      }
    }

    match attr_name.to_lowercase().as_str() {
      "highlight" => highlight = Some(attr_value),
      "plusdiff" => plusdiff = Some(attr_value),
      "minusdiff" => minusdiff = Some(attr_value),
      "linenumbers" => linenumbers = !attr_value.is_empty() && attr_value != "false",
      _ => {}
    }
  }

  CodeBlockAttrs {
    language,
    highlight,
    plusdiff,
    minusdiff,
    linenumbers,
  }
}
