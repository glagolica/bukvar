//! Sphinx/reST-style docstring parser.

use super::parse_markdown_inline;
use crate::ast::{Node, NodeKind, Span};

/// Parse Sphinx/reST-style docstring content.
pub fn parse(content: &str) -> Vec<Node> {
  let mut nodes = Vec::new();
  let lines: Vec<&str> = content.lines().collect();
  let mut description = String::new();
  let mut in_description = true;
  let mut i = 0;

  while i < lines.len() {
    let line = lines[i].trim();

    if line.starts_with(':') {
      if in_description && !description.trim().is_empty() {
        nodes.push(make_description_node(&description));
        description.clear();
      }
      in_description = false;

      if let Some(node) = parse_directive(line, &lines, &mut i) {
        nodes.push(node);
      }
    } else if in_description {
      append_line(&mut description, line);
    }

    i += 1;
  }

  if !description.trim().is_empty() {
    nodes.push(make_description_node(&description));
  }

  nodes
}

fn make_description_node(content: &str) -> Node {
  let desc_nodes = parse_markdown_inline(content);
  Node::with_children(
    NodeKind::DocDescription {
      content: content.trim().to_string(),
    },
    Span::empty(),
    desc_nodes,
  )
}

fn parse_directive(line: &str, lines: &[&str], index: &mut usize) -> Option<Node> {
  let line = &line[1..]; // Skip first ':'
  let colon_pos = line.find(':')?;
  let directive = &line[..colon_pos];
  let rest = &line[colon_pos + 1..];

  let (directive_name, directive_arg) = split_directive(directive);
  let content = collect_content(rest, lines, index);

  create_node(directive_name, directive_arg, content)
}

fn split_directive(directive: &str) -> (&str, Option<String>) {
  match directive.split_once(' ') {
    Some((name, arg)) => (name, Some(arg.to_string())),
    None => (directive, None),
  }
}

fn collect_content(initial: &str, lines: &[&str], index: &mut usize) -> String {
  let mut content = initial.trim().to_string();

  while *index + 1 < lines.len() {
    let next = lines[*index + 1];
    let is_continuation =
      next.starts_with("    ") || next.starts_with("\t") || is_content_continuation(next);

    if is_continuation {
      content.push(' ');
      content.push_str(next.trim());
      *index += 1;
    } else {
      break;
    }
  }

  content
}

fn is_content_continuation(line: &str) -> bool {
  let trimmed = line.trim();
  !trimmed.is_empty() && !trimmed.starts_with(':')
}

fn create_node(name: &str, arg: Option<String>, content: String) -> Option<Node> {
  let node = match name {
    "param" | "parameter" | "arg" | "argument" => Node::new(
      NodeKind::DocParam {
        name: arg.unwrap_or_default(),
        param_type: None,
        description: Some(content),
      },
      Span::empty(),
    ),

    "type" => Node::new(NodeKind::DocType { type_expr: content }, Span::empty()),

    "returns" | "return" => Node::new(
      NodeKind::DocReturn {
        return_type: None,
        description: Some(content),
      },
      Span::empty(),
    ),

    "rtype" => Node::new(
      NodeKind::DocReturn {
        return_type: Some(content),
        description: None,
      },
      Span::empty(),
    ),

    "raises" | "raise" | "except" | "exception" => Node::new(
      NodeKind::DocThrows {
        exception_type: arg.unwrap_or_default(),
        description: Some(content),
      },
      Span::empty(),
    ),

    _ => Node::new(
      NodeKind::DocTag {
        name: name.to_string(),
        content: make_tag_content(arg, content),
      },
      Span::empty(),
    ),
  };

  Some(node)
}

fn make_tag_content(arg: Option<String>, content: String) -> Option<String> {
  if content.is_empty() && arg.is_none() {
    return None;
  }
  Some(
    format!("{} {}", arg.unwrap_or_default(), content)
      .trim()
      .to_string(),
  )
}

fn append_line(target: &mut String, line: &str) {
  if !target.is_empty() {
    target.push('\n');
  }
  target.push_str(line);
}
