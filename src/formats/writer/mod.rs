//! DAST binary format writer

mod encode;
mod helpers;
mod strings;

use crate::ast::*;
use std::collections::HashMap;
use std::io::{self, Write};

use super::{MAGIC, VERSION};
use encode::*;
use helpers::*;

/// Writes a Document to DAST binary format.
pub struct DastWriter {
  strings: Vec<String>,
  string_map: HashMap<String, u32>,
}

impl DastWriter {
  pub fn new() -> Self {
    Self {
      strings: Vec::new(),
      string_map: HashMap::new(),
    }
  }

  pub fn write<W: Write>(&mut self, doc: &Document, w: &mut W) -> io::Result<()> {
    strings::collect_strings(&mut self.strings, &mut self.string_map, doc);
    self.write_header(w)?;
    self.write_string_table(w)?;
    self.write_document(doc, w)
  }

  fn write_header<W: Write>(&self, w: &mut W) -> io::Result<()> {
    w.write_all(MAGIC)?;
    w.write_all(&[VERSION, 0])
  }

  fn write_string_table<W: Write>(&self, w: &mut W) -> io::Result<()> {
    w.write_all(&(self.strings.len() as u32).to_le_bytes())?;
    self.strings.iter().try_for_each(|s| {
      let b = s.as_bytes();
      w.write_all(&(b.len() as u32).to_le_bytes())?;
      w.write_all(b)
    })
  }

  fn write_document<W: Write>(&self, doc: &Document, w: &mut W) -> io::Result<()> {
    self.write_str(&doc.source_path, w)?;
    w.write_all(&[doc_type_u8(&doc.doc_type)])?;
    self.write_opt_str(&doc.metadata.title, w)?;
    self.write_opt_str(&doc.metadata.description, w)?;
    w.write_all(&(doc.metadata.total_lines as u32).to_le_bytes())?;
    w.write_all(&(doc.metadata.total_nodes as u32).to_le_bytes())?;
    w.write_all(&(doc.nodes.len() as u32).to_le_bytes())?;
    doc.nodes.iter().try_for_each(|n| self.write_node(n, w))
  }

  fn write_node<W: Write>(&self, node: &Node, w: &mut W) -> io::Result<()> {
    w.write_all(&[node_kind_u8(&node.kind)])?;
    write_span(&node.span, w)?;
    self.write_kind_data(&node.kind, w)?;
    w.write_all(&(node.children.len() as u32).to_le_bytes())?;
    node.children.iter().try_for_each(|c| self.write_node(c, w))
  }

  fn write_kind_data<W: Write>(&self, kind: &NodeKind, w: &mut W) -> io::Result<()> {
    match kind {
      NodeKind::Heading { level, id } => {
        w.write_all(&[*level])?;
        self.write_opt_str(id, w)
      }
      NodeKind::CodeBlock { language, info } | NodeKind::FencedCodeBlock { language, info } => {
        self.write_opt_str(language, w)?;
        self.write_opt_str(info, w)
      }
      NodeKind::HtmlBlock { block_type } => w.write_all(&[*block_type]),
      NodeKind::List {
        ordered,
        start,
        tight,
      } => {
        w.write_all(&[*ordered as u8, *tight as u8])?;
        write_opt_u32(start, w)
      }
      NodeKind::ListItem { marker, checked } => {
        write_marker(marker, w)?;
        write_opt_bool(checked, w)
      }
      NodeKind::TableCell {
        alignment,
        is_header,
      } => w.write_all(&[alignment_u8(alignment), *is_header as u8]),
      NodeKind::Text { content }
      | NodeKind::Code { content }
      | NodeKind::CodeSpan { content }
      | NodeKind::HtmlInline { content } => self.write_str(content, w),
      NodeKind::Link {
        url,
        title,
        ref_type,
      } => {
        self.write_str(url, w)?;
        self.write_opt_str(title, w)?;
        w.write_all(&[ref_type_u8(ref_type)])
      }
      NodeKind::Image { url, alt, title } => {
        self.write_str(url, w)?;
        self.write_str(alt, w)?;
        self.write_opt_str(title, w)
      }
      NodeKind::AutoLink { url } => self.write_str(url, w),
      NodeKind::LinkReference { label, ref_type } => {
        self.write_str(label, w)?;
        w.write_all(&[ref_type_u8(ref_type)])
      }
      NodeKind::LinkDefinition { label, url, title } => {
        self.write_str(label, w)?;
        self.write_str(url, w)?;
        self.write_opt_str(title, w)
      }
      NodeKind::FootnoteReference { label } | NodeKind::FootnoteDefinition { label } => {
        self.write_str(label, w)
      }
      NodeKind::TaskListMarker { checked } => w.write_all(&[*checked as u8]),
      NodeKind::Emoji { shortcode } => self.write_str(shortcode, w),
      NodeKind::Mention { username } => self.write_str(username, w),
      NodeKind::IssueReference { number } => w.write_all(&number.to_le_bytes()),
      NodeKind::DocComment { style } => w.write_all(&[doc_style_u8(style)]),
      NodeKind::DocTag { name, content } => {
        self.write_str(name, w)?;
        self.write_opt_str(content, w)
      }
      NodeKind::DocParam {
        name,
        param_type,
        description,
      } => {
        self.write_str(name, w)?;
        self.write_opt_str(param_type, w)?;
        self.write_opt_str(description, w)
      }
      NodeKind::DocReturn {
        return_type,
        description,
      } => {
        self.write_opt_str(return_type, w)?;
        self.write_opt_str(description, w)
      }
      NodeKind::DocThrows {
        exception_type,
        description,
      } => {
        self.write_str(exception_type, w)?;
        self.write_opt_str(description, w)
      }
      NodeKind::DocExample { content } | NodeKind::DocDescription { content } => {
        self.write_str(content, w)
      }
      NodeKind::DocSee { reference } => self.write_str(reference, w),
      NodeKind::DocDeprecated { message } => self.write_opt_str(message, w),
      NodeKind::DocSince { version } | NodeKind::DocVersion { version } => {
        self.write_str(version, w)
      }
      NodeKind::DocAuthor { name } => self.write_str(name, w),
      NodeKind::DocType { type_expr } => self.write_str(type_expr, w),
      NodeKind::DocProperty {
        name,
        prop_type,
        description,
      } => {
        self.write_str(name, w)?;
        self.write_opt_str(prop_type, w)?;
        self.write_opt_str(description, w)
      }
      NodeKind::DocCallback { name } => self.write_str(name, w),
      NodeKind::DocTypedef { name, type_expr } => {
        self.write_str(name, w)?;
        self.write_opt_str(type_expr, w)
      }
      NodeKind::Alert { alert_type } => w.write_all(&[alert_type_u8(alert_type)]),
      NodeKind::Tabs { names } => {
        w.write_all(&(names.len() as u32).to_le_bytes())?;
        for name in names {
          self.write_str(name, w)?;
        }
        Ok(())
      }
      NodeKind::CodeBlockExt {
        language,
        highlight,
        plusdiff,
        minusdiff,
        linenumbers,
      } => {
        self.write_opt_str(language, w)?;
        self.write_opt_str(highlight, w)?;
        self.write_opt_str(plusdiff, w)?;
        self.write_opt_str(minusdiff, w)?;
        w.write_all(&[*linenumbers as u8])
      }
      _ => Ok(()),
    }
  }

  fn write_str<W: Write>(&self, s: &str, w: &mut W) -> io::Result<()> {
    let idx = self.string_map.get(s).copied().unwrap_or(0);
    w.write_all(&idx.to_le_bytes())
  }

  fn write_opt_str<W: Write>(&self, s: &Option<String>, w: &mut W) -> io::Result<()> {
    match s {
      Some(s) => {
        w.write_all(&[1])?;
        self.write_str(s, w)
      }
      None => w.write_all(&[0]),
    }
  }
}
