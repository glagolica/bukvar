//! NumPy-style docstring parser.

use super::{parse_markdown_inline, DocItem};
use crate::ast::{Node, NodeKind, Span};

/// Parse NumPy-style docstring content.
pub fn parse(content: &str) -> Vec<Node> {
  let mut nodes = Vec::new();
  let lines: Vec<&str> = content.lines().collect();
  let mut i = 0;
  let mut state = ParseState::default();

  while i < lines.len() {
    let line = lines[i];
    let trimmed = line.trim();

    if is_section_header(&lines, i) {
      flush_previous(&mut state, &mut nodes);
      state.current_section = detect_section(trimmed);
      state.section_content.clear();
      state.description.clear();
      i += 2; // Skip header and underline
      continue;
    }

    if state.current_section.is_some() {
      append_line(&mut state.section_content, line);
    } else {
      append_line(&mut state.description, trimmed);
    }
    i += 1;
  }

  finalize(&mut state, &mut nodes);
  nodes
}

#[derive(Default)]
struct ParseState {
  current_section: Option<&'static str>,
  description: String,
  section_content: String,
}

fn is_section_header(lines: &[&str], i: usize) -> bool {
  i + 1 < lines.len() && {
    let next = lines[i + 1].trim();
    !next.is_empty() && next.chars().all(|c| c == '-')
  }
}

fn detect_section(header: &str) -> Option<&'static str> {
  match header.to_lowercase().as_str() {
    "parameters" => Some("parameters"),
    "returns" => Some("returns"),
    "yields" => Some("yields"),
    "raises" => Some("raises"),
    "attributes" => Some("attributes"),
    "examples" | "example" => Some("example"),
    "notes" | "note" => Some("note"),
    "see also" => Some("see_also"),
    "references" => Some("references"),
    _ => Some("other"),
  }
}

fn flush_previous(state: &mut ParseState, nodes: &mut Vec<Node>) {
  if let Some(prev_section) = state.current_section {
    nodes.extend(process_section(prev_section, &state.section_content));
  } else if !state.description.trim().is_empty() {
    nodes.push(make_description_node(&state.description));
  }
}

fn finalize(state: &mut ParseState, nodes: &mut Vec<Node>) {
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
    "parameters" | "attributes" => parse_items(content)
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

    "returns" | "yields" => parse_items(content)
      .into_iter()
      .take(1)
      .map(|item| {
        Node::new(
          NodeKind::DocReturn {
            return_type: item.item_type,
            description: item.description,
          },
          Span::empty(),
        )
      })
      .collect(),

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

    "see_also" => content
      .lines()
      .map(str::trim)
      .filter(|s| !s.is_empty())
      .map(|s| {
        Node::new(
          NodeKind::DocSee {
            reference: s.to_string(),
          },
          Span::empty(),
        )
      })
      .collect(),

    _ => vec![Node::new(
      NodeKind::DocTag {
        name: section.to_string(),
        content: Some(content.trim().to_string()),
      },
      Span::empty(),
    )],
  }
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
  // NumPy format: "name : type" or just "name"
  match line.find(" : ") {
    Some(pos) => DocItem::new(
      line[..pos].trim().to_string(),
      Some(line[pos + 3..].trim().to_string()),
      None,
    ),
    None => DocItem::new(line.to_string(), None, None),
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
