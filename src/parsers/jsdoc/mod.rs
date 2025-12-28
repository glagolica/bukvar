//! JSDoc parser for JavaScript/TypeScript files

mod tags;

use crate::ast::*;
use crate::markdown::MarkdownParser;

pub struct JsDocParser<'a> {
  input: &'a str,
  bytes: &'a [u8],
  pos: usize,
  line: usize,
  column: usize,
}

impl<'a> JsDocParser<'a> {
  pub fn new(input: &'a str) -> Self {
    Self {
      input,
      bytes: input.as_bytes(),
      pos: 0,
      line: 1,
      column: 1,
    }
  }

  pub fn parse(&mut self) -> Document {
    let nodes = self.collect_comments();
    let total_nodes: usize = nodes.iter().map(|n| n.count_nodes()).sum();

    Document {
      source_path: String::new(),
      doc_type: DocumentType::JavaScript,
      nodes,
      metadata: DocumentMetadata {
        title: None,
        description: None,
        total_lines: self.line,
        total_nodes,
      },
    }
  }

  fn collect_comments(&mut self) -> Vec<Node> {
    let mut nodes = Vec::new();
    while !self.is_eof() {
      if self.check_str(b"/**") && !self.check_str(b"/***") {
        if let Some(n) = self.parse_jsdoc_comment() {
          nodes.push(n);
        }
      } else {
        self.advance();
      }
    }
    nodes
  }

  fn parse_jsdoc_comment(&mut self) -> Option<Node> {
    let start_pos = self.pos;
    let start_line = self.line;
    let start_col = self.column;

    self.advance_n(3); // Skip /**

    let content = self.extract_comment_content()?;
    let children = self.parse_jsdoc_content(&content);

    Some(Node::with_children(
      NodeKind::DocComment {
        style: DocStyle::JSDoc,
      },
      Span::new(start_pos, self.pos, start_line, start_col),
      children,
    ))
  }

  fn extract_comment_content(&mut self) -> Option<String> {
    let mut content = String::new();

    while !self.is_eof() {
      if self.check_str(b"*/") {
        self.advance_n(2);
        return Some(content);
      }

      if self.check(b'\n') {
        content.push('\n');
        self.advance();
        self.skip_line_prefix();
      } else {
        if let Some(c) = self.input[self.pos..].chars().next() {
          content.push(c);
          self.advance_n(c.len_utf8());
        } else {
          self.advance();
        }
      }
    }

    None
  }

  fn skip_line_prefix(&mut self) {
    self.skip_whitespace_inline();
    if self.check(b'*') && !self.check_str(b"*/") {
      self.advance();
      if self.check(b' ') {
        self.advance();
      }
    }
  }

  fn parse_jsdoc_content(&self, content: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut description = String::new();
    let mut in_description = true;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
      let line = lines[i].trim();

      if line.starts_with('@') {
        self.flush_description(&mut description, &mut nodes, &mut in_description);
        if let Some(n) = tags::parse_tag(self, line, &lines, &mut i) {
          nodes.push(n);
        }
      } else if in_description {
        if !description.is_empty() {
          description.push('\n');
        }
        description.push_str(line);
      }

      i += 1;
    }

    self.flush_description(&mut description, &mut nodes, &mut in_description);
    nodes
  }

  fn flush_description(&self, desc: &mut String, nodes: &mut Vec<Node>, in_desc: &mut bool) {
    if *in_desc && !desc.trim().is_empty() {
      let desc_nodes = self.parse_markdown_inline(desc);
      nodes.push(Node::with_children(
        NodeKind::DocDescription {
          content: desc.trim().to_string(),
        },
        Span::empty(),
        desc_nodes,
      ));
      desc.clear();
    }
    *in_desc = false;
  }

  pub(crate) fn parse_markdown_inline(&self, content: &str) -> Vec<Node> {
    MarkdownParser::new(content).parse().nodes
  }

  #[inline(always)]
  fn is_eof(&self) -> bool {
    self.pos >= self.bytes.len()
  }

  #[inline(always)]
  fn check(&self, expected: u8) -> bool {
    self.bytes.get(self.pos).copied() == Some(expected)
  }

  fn check_str(&self, expected: &[u8]) -> bool {
    self.bytes[self.pos..].starts_with(expected)
  }

  #[inline(always)]
  fn advance(&mut self) {
    if !self.is_eof() {
      if self.bytes[self.pos] == b'\n' {
        self.line += 1;
        self.column = 1;
      } else {
        self.column += 1;
      }
      self.pos += 1;
    }
  }

  fn advance_n(&mut self, n: usize) {
    (0..n).for_each(|_| self.advance());
  }

  fn skip_whitespace_inline(&mut self) {
    while self
      .bytes
      .get(self.pos)
      .is_some_and(|&b| b == b' ' || b == b'\t')
    {
      self.advance();
    }
  }
}
