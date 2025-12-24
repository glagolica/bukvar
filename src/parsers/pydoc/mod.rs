//! PyDoc parser for Python files
//! Supports standard docstrings, Google style, and NumPy style

mod google;
mod item;
mod numpy;
mod sphinx;

use crate::ast::*;

pub use self::item::DocItem;

/// PyDoc parser for extracting documentation from Python source files.
pub struct PyDocParser<'a> {
  input: &'a str,
  bytes: &'a [u8],
  pos: usize,
  line: usize,
  column: usize,
}

impl<'a> PyDocParser<'a> {
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
    let nodes = self.collect_docstrings();
    let total_nodes: usize = nodes.iter().map(Node::count_nodes).sum();

    Document {
      source_path: String::new(),
      doc_type: DocumentType::Python,
      nodes,
      metadata: DocumentMetadata {
        title: None,
        description: None,
        total_lines: self.line,
        total_nodes,
      },
    }
  }

  fn collect_docstrings(&mut self) -> Vec<Node> {
    let mut nodes = Vec::new();
    while !self.is_eof() {
      if let Some(node) = self.try_parse_docstring() {
        nodes.push(node);
      } else {
        self.advance();
      }
    }
    nodes
  }

  fn try_parse_docstring(&mut self) -> Option<Node> {
    let delimiter = self.detect_delimiter()?;
    self.parse_docstring_with_delimiter(delimiter)
  }

  fn detect_delimiter(&self) -> Option<&'static [u8]> {
    if self.check_str(b"\"\"\"") {
      Some(b"\"\"\"")
    } else if self.check_str(b"'''") {
      Some(b"'''")
    } else {
      None
    }
  }

  fn parse_docstring_with_delimiter(&mut self, delimiter: &[u8]) -> Option<Node> {
    let (start_pos, start_line, start_col) = (self.pos, self.line, self.column);
    self.advance_n(3);

    let content = self.consume_until_delimiter(delimiter)?;
    self.advance_n(3);

    let (style, children) = self.detect_and_parse_style(&content);
    Some(Node::with_children(
      NodeKind::DocComment { style },
      Span::new(start_pos, self.pos, start_line, start_col),
      children,
    ))
  }

  fn consume_until_delimiter(&mut self, delimiter: &[u8]) -> Option<String> {
    let content_start = self.pos;
    while !self.is_eof() && !self.check_str(delimiter) {
      self.advance();
    }
    if self.is_eof() {
      return None;
    }
    Some(self.input[content_start..self.pos].to_string())
  }

  fn detect_and_parse_style(&self, content: &str) -> (DocStyle, Vec<Node>) {
    let content = dedent(content);

    if is_google_style(&content) {
      return (DocStyle::PyDocGoogle, google::parse(&content));
    }
    if is_numpy_style(&content) {
      return (DocStyle::PyDocNumpy, numpy::parse(&content));
    }
    if is_sphinx_style(&content) {
      return (DocStyle::PyDoc, sphinx::parse(&content));
    }

    (DocStyle::PyDoc, parse_plain_docstring(&content))
  }

  #[inline(always)]
  fn is_eof(&self) -> bool {
    self.pos >= self.bytes.len()
  }

  #[inline(always)]
  fn check_str(&self, expected: &[u8]) -> bool {
    self.bytes[self.pos..].starts_with(expected)
  }

  #[inline(always)]
  fn advance(&mut self) {
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

  #[inline(always)]
  fn advance_n(&mut self, n: usize) {
    for _ in 0..n {
      self.advance();
    }
  }
}

// Style detection functions
fn is_google_style(content: &str) -> bool {
  const MARKERS: [&str; 6] = [
    "\nArgs:",
    "\nReturns:",
    "\nRaises:",
    "\nExample:",
    "\nAttributes:",
    "\nYields:",
  ];
  MARKERS.iter().any(|m| content.contains(m))
}

fn is_numpy_style(content: &str) -> bool {
  const MARKERS: [&str; 3] = [
    "\nParameters\n----------",
    "\nReturns\n-------",
    "\nRaises\n------",
  ];
  MARKERS.iter().any(|m| content.contains(m))
}

fn is_sphinx_style(content: &str) -> bool {
  content.contains(":param ") || content.contains(":returns:") || content.contains(":raises:")
}

/// Remove common leading indentation from docstring content.
pub fn dedent(content: &str) -> String {
  let lines: Vec<&str> = content.lines().collect();
  if lines.is_empty() {
    return String::new();
  }

  let min_indent = lines
    .iter()
    .skip(1)
    .filter(|line| !line.trim().is_empty())
    .map(|line| line.len() - line.trim_start().len())
    .min()
    .unwrap_or(0);

  lines
    .iter()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        line.trim().to_string()
      } else if line.trim().is_empty() {
        String::new()
      } else if line.len() >= min_indent {
        line[min_indent..].to_string()
      } else {
        line.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
    .trim()
    .to_string()
}

fn parse_plain_docstring(content: &str) -> Vec<Node> {
  use crate::markdown::MarkdownParser;
  let mut parser = MarkdownParser::new(content);
  let doc = parser.parse();
  vec![Node::with_children(
    NodeKind::DocDescription {
      content: content.to_string(),
    },
    Span::empty(),
    doc.nodes,
  )]
}

pub fn parse_markdown_inline(content: &str) -> Vec<Node> {
  use crate::markdown::MarkdownParser;
  let mut parser = MarkdownParser::new(content);
  parser.parse().nodes
}
