//! JSDoc tag parsing.

use super::JsDocParser;
use crate::ast::*;

pub fn parse_tag(
  _parser: &JsDocParser,
  line: &str,
  lines: &[&str],
  index: &mut usize,
) -> Option<Node> {
  let parts: Vec<&str> = line[1..].splitn(2, char::is_whitespace).collect();
  let tag_name = parts[0].to_lowercase();
  let rest = parts.get(1).map(|s| s.trim()).unwrap_or("");

  let content = collect_continuation(rest, lines, index);

  match tag_name.as_str() {
    "param" | "arg" | "argument" => parse_param(&content),
    "returns" | "return" => parse_return(&content),
    "throws" | "exception" => parse_throws(&content),
    "type" => Some(make_type(&content)),
    "typedef" => Some(make_typedef(&content)),
    "callback" => Some(make_callback(&content)),
    "property" | "prop" => parse_property(&content),
    "example" => Some(make_example(&content)),
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
  let (param_type, rest) = extract_type_prefix(content);
  let parts: Vec<&str> = rest
    .splitn(2, |c: char| c == '-' || c.is_whitespace())
    .collect();

  Some(Node::new(
    NodeKind::DocParam {
      name: parts[0].trim().to_string(),
      param_type,
      description: parts
        .get(1)
        .map(|s| s.trim().trim_start_matches('-').trim().to_string()),
    },
    Span::empty(),
  ))
}

fn parse_return(content: &str) -> Option<Node> {
  let (return_type, description) = extract_type_prefix(content);
  Some(Node::new(
    NodeKind::DocReturn {
      return_type,
      description: non_empty_str(description),
    },
    Span::empty(),
  ))
}

fn parse_throws(content: &str) -> Option<Node> {
  let (exception_type, description) = extract_type_prefix(content);
  let exception =
    exception_type.unwrap_or_else(|| content.split_whitespace().next().unwrap_or("").to_string());

  Some(Node::new(
    NodeKind::DocThrows {
      exception_type: exception,
      description: non_empty_str(description),
    },
    Span::empty(),
  ))
}

fn parse_property(content: &str) -> Option<Node> {
  let (prop_type, rest) = extract_type_prefix(content);
  let parts: Vec<&str> = rest
    .splitn(2, |c: char| c == '-' || c.is_whitespace())
    .collect();

  Some(Node::new(
    NodeKind::DocProperty {
      name: parts[0].trim().to_string(),
      prop_type,
      description: parts
        .get(1)
        .map(|s| s.trim().trim_start_matches('-').trim().to_string()),
    },
    Span::empty(),
  ))
}

fn make_type(content: &str) -> Node {
  Node::new(
    NodeKind::DocType {
      type_expr: content.to_string(),
    },
    Span::empty(),
  )
}

fn make_typedef(content: &str) -> Node {
  let (type_expr, rest) = extract_type_prefix(content);
  let name = rest.split_whitespace().next().unwrap_or("").to_string();
  Node::new(NodeKind::DocTypedef { name, type_expr }, Span::empty())
}

fn make_callback(content: &str) -> Node {
  Node::new(
    NodeKind::DocCallback {
      name: content.to_string(),
    },
    Span::empty(),
  )
}

fn make_example(content: &str) -> Node {
  Node::new(
    NodeKind::DocExample {
      content: content.to_string(),
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

fn extract_type_prefix(content: &str) -> (Option<String>, &str) {
  let content = content.trim();
  content
    .starts_with('{')
    .then(|| content.find('}'))
    .flatten()
    .map(|end| (Some(content[1..end].to_string()), content[end + 1..].trim()))
    .unwrap_or((None, content))
}

fn non_empty_str(s: &str) -> Option<String> {
  let trimmed = s.trim();
  (!trimmed.is_empty()).then(|| trimmed.to_string())
}
