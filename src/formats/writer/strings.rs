//! String table and interning for DAST binary format.

use crate::ast::{Document, Node, NodeKind};
use std::collections::HashMap;

/// Collect all strings from document into the string table.
pub fn collect_strings(strings: &mut Vec<String>, map: &mut HashMap<String, u32>, doc: &Document) {
  let mut intern = |s: &str| {
    if !map.contains_key(s) {
      let idx = strings.len() as u32;
      strings.push(s.to_string());
      map.insert(s.to_string(), idx);
    }
  };

  intern(&doc.source_path);
  if let Some(s) = doc.metadata.title.as_ref() {
    intern(s);
  }
  if let Some(s) = doc.metadata.description.as_ref() {
    intern(s);
  }

  doc
    .nodes
    .iter()
    .for_each(|n| collect_node_strings(n, strings, map));
}

fn collect_node_strings(node: &Node, strings: &mut Vec<String>, map: &mut HashMap<String, u32>) {
  collect_kind_strings(&node.kind, strings, map);
  node
    .children
    .iter()
    .for_each(|c| collect_node_strings(c, strings, map));
}

fn collect_kind_strings(
  kind: &NodeKind,
  strings: &mut Vec<String>,
  map: &mut HashMap<String, u32>,
) {
  let mut intern = |s: &str| {
    if !map.contains_key(s) {
      let idx = strings.len() as u32;
      strings.push(s.to_string());
      map.insert(s.to_string(), idx);
    }
  };

  match kind {
    NodeKind::Heading { id, .. } => {
      if let Some(s) = id.as_ref() {
        intern(s);
      }
    }
    NodeKind::CodeBlock { language, info } | NodeKind::FencedCodeBlock { language, info } => {
      if let Some(s) = language.as_ref() {
        intern(s);
      }
      if let Some(s) = info.as_ref() {
        intern(s);
      }
    }
    NodeKind::Text { content }
    | NodeKind::Code { content }
    | NodeKind::CodeSpan { content }
    | NodeKind::HtmlInline { content }
    | NodeKind::DocExample { content }
    | NodeKind::DocDescription { content } => {
      intern(content);
    }
    NodeKind::Link { url, title, .. } => {
      intern(url);
      if let Some(s) = title.as_ref() {
        intern(s);
      }
    }
    NodeKind::Image { url, alt, title } => {
      intern(url);
      intern(alt);
      if let Some(s) = title.as_ref() {
        intern(s);
      }
    }
    NodeKind::AutoLink { url } => {
      intern(url);
    }
    NodeKind::LinkReference { label, .. }
    | NodeKind::FootnoteReference { label }
    | NodeKind::FootnoteDefinition { label } => {
      intern(label);
    }
    NodeKind::LinkDefinition { label, url, title } => {
      intern(label);
      intern(url);
      if let Some(s) = title.as_ref() {
        intern(s);
      }
    }
    NodeKind::Emoji { shortcode } => {
      intern(shortcode);
    }
    NodeKind::Mention { username } => {
      intern(username);
    }
    NodeKind::DocTag { name, content } => {
      intern(name);
      if let Some(s) = content.as_ref() {
        intern(s);
      }
    }
    NodeKind::DocParam {
      name,
      param_type,
      description,
    }
    | NodeKind::DocProperty {
      name,
      prop_type: param_type,
      description,
    } => {
      intern(name);
      if let Some(s) = param_type.as_ref() {
        intern(s);
      }
      if let Some(s) = description.as_ref() {
        intern(s);
      }
    }
    NodeKind::DocReturn {
      return_type,
      description,
    } => {
      if let Some(s) = return_type.as_ref() {
        intern(s);
      }
      if let Some(s) = description.as_ref() {
        intern(s);
      }
    }
    NodeKind::DocThrows {
      exception_type,
      description,
    } => {
      intern(exception_type);
      if let Some(s) = description.as_ref() {
        intern(s);
      }
    }
    NodeKind::DocSee { reference } => {
      intern(reference);
    }
    NodeKind::DocDeprecated { message } => {
      if let Some(s) = message.as_ref() {
        intern(s);
      }
    }
    NodeKind::DocSince { version } | NodeKind::DocVersion { version } => {
      intern(version);
    }
    NodeKind::DocAuthor { name } | NodeKind::DocCallback { name } => {
      intern(name);
    }
    NodeKind::DocType { type_expr } => {
      intern(type_expr);
    }
    NodeKind::DocTypedef { name, type_expr } => {
      intern(name);
      if let Some(s) = type_expr.as_ref() {
        intern(s);
      }
    }
    _ => {}
  }
}
