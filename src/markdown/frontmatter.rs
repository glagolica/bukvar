//! Frontmatter parsing (YAML/TOML).

use super::scanner::Scanner;
use crate::ast::{FrontmatterFormat, Node, NodeKind, Span};

/// Try to parse YAML/TOML frontmatter at document start.
pub fn try_parse(scanner: &mut Scanner) -> Option<Node> {
  if scanner.pos() != 0 {
    return None;
  }

  let input = scanner.remaining();

  try_yaml(scanner, input).or_else(|| try_toml(scanner, input))
}

fn try_yaml(scanner: &mut Scanner, input: &str) -> Option<Node> {
  if !input.starts_with("---\n") || input.len() <= 4 {
    return None;
  }

  let search = &input[4..];
  let end_idx = search.find("\n---")?;
  let content = input[4..4 + end_idx].trim().to_string();
  let total_len = 4 + end_idx + 4;

  let node = Node::new(
    NodeKind::Frontmatter {
      format: FrontmatterFormat::Yaml,
      content,
    },
    Span::new(0, total_len, 1, 1),
  );

  scanner.advance_n(total_len);
  scanner.consume(b'\n');

  Some(node)
}

fn try_toml(scanner: &mut Scanner, input: &str) -> Option<Node> {
  if !input.starts_with("+++\n") || input.len() <= 4 {
    return None;
  }

  let end_idx = input[4..].find("\n+++")?;
  let content = input[4..4 + end_idx].trim().to_string();
  let total_len = 4 + end_idx + 4;

  let node = Node::new(
    NodeKind::Frontmatter {
      format: FrontmatterFormat::Toml,
      content,
    },
    Span::new(0, total_len, 1, 1),
  );

  scanner.advance_n(total_len);
  scanner.consume(b'\n');

  Some(node)
}

/// Skip over frontmatter when re-scanning.
pub fn skip(scanner: &mut Scanner) {
  let input = scanner.remaining();
  let delim = if input.starts_with("---") {
    "---"
  } else {
    "+++"
  };

  scanner.advance_n(3);
  scanner.skip_line();

  while !scanner.is_eof() {
    let line = scanner.scan_line();
    if line.trim() == delim {
      return;
    }
  }
}
