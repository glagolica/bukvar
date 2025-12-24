//! Google-style docstring parser.

use super::{parse_markdown_inline, DocItem};
use crate::ast::{Node, NodeKind, Span};

/// Parse Google-style docstring content.
pub fn parse(content: &str) -> Vec<Node> {
  let mut nodes = Vec::new();
  let mut state = ParseState::default();

  for line in content.lines() {
    process_line(line, &mut state, &mut nodes);
  }

  finalize_section(&mut state, &mut nodes);
  nodes
}

#[derive(Default)]
struct ParseState {
  current_section: Option<&'static str>,
  description: String,
  section_content: String,
}

fn process_line(line: &str, state: &mut ParseState, nodes: &mut Vec<Node>) {
  let trimmed = line.trim();

  if let Some(section) = detect_section(trimmed) {
    flush_previous(state, nodes);
    state.current_section = Some(section);
    state.section_content.clear();
    state.description.clear();
  } else if state.current_section.is_some() {
    append_line(&mut state.section_content, line);
  } else {
    append_line(&mut state.description, trimmed);
  }
}

fn detect_section(line: &str) -> Option<&'static str> {
  match line {
    "Args:" | "Arguments:" => Some("args"),
    "Returns:" => Some("returns"),
    "Yields:" => Some("yields"),
    "Raises:" => Some("raises"),
    "Attributes:" => Some("attributes"),
    "Example:" | "Examples:" => Some("example"),
    "Note:" | "Notes:" => Some("note"),
    "Todo:" => Some("todo"),
    _ => None,
  }
}

fn flush_previous(state: &mut ParseState, nodes: &mut Vec<Node>) {
  if let Some(prev_section) = state.current_section {
    nodes.extend(process_section(prev_section, &state.section_content));
  } else if !state.description.trim().is_empty() {
    nodes.push(make_description_node(&state.description));
  }
}

fn finalize_section(state: &mut ParseState, nodes: &mut Vec<Node>) {
  if let Some(section) = state.current_section {
    nodes.extend(process_section(section, &state.section_content));
  } else if !state.description.trim().is_empty() {
    nodes.push(make_description_node(&state.description));
  }
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

fn process_section(section: &str, content: &str) -> Vec<Node> {
  match section {
    "args" | "attributes" => parse_items(content)
      .into_iter()
      .map(|item| {
        Node::new(
          NodeKind::DocParam {
            name: item.name,
            param_type: item.item_type,
            description: item.description,
          },
          Span::empty(),
        )
      })
      .collect(),

    "returns" | "yields" => {
      let (return_type, desc) = parse_return_content(content);
      vec![Node::new(
        NodeKind::DocReturn {
          return_type,
          description: desc,
        },
        Span::empty(),
      )]
    }

    "raises" => parse_items(content)
      .into_iter()
      .map(|item| {
        Node::new(
          NodeKind::DocThrows {
            exception_type: item.name,
            description: item.description,
          },
          Span::empty(),
        )
      })
      .collect(),

    "example" => vec![Node::new(
      NodeKind::DocExample {
        content: content.trim().to_string(),
      },
      Span::empty(),
    )],

    _ => vec![Node::new(
      NodeKind::DocTag {
        name: section.to_string(),
        content: Some(content.trim().to_string()),
      },
      Span::empty(),
    )],
  }
}

fn parse_return_content(content: &str) -> (Option<String>, Option<String>) {
  let content = content.trim();
  content
    .find(':')
    .filter(|&pos| !content[..pos].contains(' '))
    .map(|pos| {
      (
        Some(content[..pos].trim().to_string()),
        Some(content[pos + 1..].trim().to_string()),
      )
    })
    .unwrap_or((None, Some(content.to_string())))
}

fn parse_items(content: &str) -> Vec<DocItem> {
  let mut items = Vec::new();
  let mut current: Option<DocItem> = None;

  for line in content.lines() {
    let trimmed = line.trim();
    let is_continuation = line.starts_with("    ") || line.starts_with("\t");

    if !trimmed.is_empty() && !is_continuation {
      if let Some(item) = current.take() {
        items.push(item);
      }
      current = Some(parse_item_line(trimmed));
    } else if let Some(ref mut item) = current {
      append_to_description(item, trimmed);
    }
  }

  if let Some(item) = current {
    items.push(item);
  }

  items
}

fn parse_item_line(line: &str) -> DocItem {
  let colon_pos = line.find(':');
  let (name, item_type, desc) = match colon_pos {
    Some(pos) => {
      let before = &line[..pos];
      let after = line[pos + 1..].trim().to_string();
      let (name, item_type) = parse_name_type(before);
      (
        name,
        item_type,
        if after.is_empty() { None } else { Some(after) },
      )
    }
    None => (line.to_string(), None, None),
  };
  DocItem::new(name, item_type, desc)
}

fn parse_name_type(s: &str) -> (String, Option<String>) {
  match (s.find('('), s.find(')')) {
    (Some(start), Some(end)) if start < end => (
      s[..start].trim().to_string(),
      Some(s[start + 1..end].trim().to_string()),
    ),
    _ => (s.trim().to_string(), None),
  }
}

fn append_to_description(item: &mut DocItem, text: &str) {
  match &mut item.description {
    Some(desc) => {
      desc.push(' ');
      desc.push_str(text);
    }
    None if !text.is_empty() => item.description = Some(text.to_string()),
    _ => {}
  }
}

fn append_line(target: &mut String, line: &str) {
  if !target.is_empty() {
    target.push('\n');
  }
  target.push_str(line);
}
