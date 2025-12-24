//! JavaDoc tag parsing.

use crate::ast::*;

pub fn parse_tag(line: &str, lines: &[&str], index: &mut usize) -> Option<Node> {
  let parts: Vec<&str> = line[1..].splitn(2, char::is_whitespace).collect();
  let tag_name = parts[0].to_lowercase();
  let rest = parts.get(1).map(|s| s.trim()).unwrap_or("");

  let content = collect_continuation(rest, lines, index);

  match tag_name.as_str() {
    "param" => parse_param(&content),
    "return" | "returns" => Some(make_return(&content)),
    "throws" | "exception" => parse_throws(&content),
    "see" => Some(make_see(&content)),
    "deprecated" => Some(make_deprecated(&content)),
    "since" => Some(make_since(&content)),
    "author" => Some(make_author(&content)),
    "version" => Some(make_version(&content)),
    _ => Some(make_generic_tag(tag_name, content)),
  }
}

fn collect_continuation(initial: &str, lines: &[&str], index: &mut usize) -> String {
  let mut content = initial.to_string();
  while *index + 1 < lines.len() {
    let next_line = lines[*index + 1].trim();
    if next_line.starts_with('@') || next_line.is_empty() {
      break;
    }
    content.push(' ');
    content.push_str(next_line);
    *index += 1;
  }
  content
}

fn parse_param(content: &str) -> Option<Node> {
  let parts: Vec<&str> = content.splitn(2, char::is_whitespace).collect();
  Some(Node::new(
    NodeKind::DocParam {
      name: parts[0].to_string(),
      param_type: None,
      description: parts.get(1).map(|s| s.trim().to_string()),
    },
    Span::empty(),
  ))
}

fn parse_throws(content: &str) -> Option<Node> {
  let parts: Vec<&str> = content.splitn(2, char::is_whitespace).collect();
  Some(Node::new(
    NodeKind::DocThrows {
      exception_type: parts[0].to_string(),
      description: parts.get(1).map(|s| s.trim().to_string()),
    },
    Span::empty(),
  ))
}

fn make_return(content: &str) -> Node {
  Node::new(
    NodeKind::DocReturn {
      return_type: None,
      description: Some(content.to_string()),
    },
    Span::empty(),
  )
}

fn make_see(content: &str) -> Node {
  Node::new(
    NodeKind::DocSee {
      reference: content.to_string(),
    },
    Span::empty(),
  )
}

fn make_deprecated(content: &str) -> Node {
  Node::new(
    NodeKind::DocDeprecated {
      message: non_empty_str(content),
    },
    Span::empty(),
  )
}

fn make_since(content: &str) -> Node {
  Node::new(
    NodeKind::DocSince {
      version: content.to_string(),
    },
    Span::empty(),
  )
}

fn make_author(content: &str) -> Node {
  Node::new(
    NodeKind::DocAuthor {
      name: content.to_string(),
    },
    Span::empty(),
  )
}

fn make_version(content: &str) -> Node {
  Node::new(
    NodeKind::DocVersion {
      version: content.to_string(),
    },
    Span::empty(),
  )
}

fn make_generic_tag(name: String, content: String) -> Node {
  Node::new(
    NodeKind::DocTag {
      name,
      content: non_empty_str(&content),
    },
    Span::empty(),
  )
}

fn non_empty_str(s: &str) -> Option<String> {
  let trimmed = s.trim();
  (!trimmed.is_empty()).then(|| trimmed.to_string())
}
