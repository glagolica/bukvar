//! DAST binary format reader.

mod decode;
mod helpers;

use crate::ast::*;
use std::io::{self, Read};

use super::{MAGIC, VERSION};
use decode::*;
use helpers::*;

/// Reads a Document from DAST binary format.
pub struct DastReader {
  strings: Vec<String>,
}

impl DastReader {
  pub fn new() -> Self {
    Self {
      strings: Vec::new(),
    }
  }

  pub fn read<R: Read>(&mut self, r: &mut R) -> io::Result<Document> {
    self.read_header(r)?;
    self.read_string_table(r)?;
    self.read_document(r)
  }

  fn read_header<R: Read>(&self, r: &mut R) -> io::Result<()> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Invalid DAST magic",
      ));
    }
    let mut ver = [0u8; 2];
    r.read_exact(&mut ver)?;
    if ver[0] != VERSION {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Unsupported version",
      ));
    }
    Ok(())
  }

  fn read_string_table<R: Read>(&mut self, r: &mut R) -> io::Result<()> {
    let count = read_u32(r)? as usize;
    self.strings = (0..count)
      .map(|_| {
        let len = read_u32(r)? as usize;
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
      })
      .collect::<io::Result<Vec<_>>>()?;
    Ok(())
  }

  fn read_document<R: Read>(&mut self, r: &mut R) -> io::Result<Document> {
    let source_path = self.read_str(r)?;
    let doc_type = u8_to_doc_type(read_u8(r)?);
    let title = self.read_opt_str(r)?;
    let description = self.read_opt_str(r)?;
    let total_lines = read_u32(r)? as usize;
    let total_nodes = read_u32(r)? as usize;
    let node_count = read_u32(r)? as usize;
    let nodes = (0..node_count)
      .map(|_| self.read_node(r))
      .collect::<io::Result<Vec<_>>>()?;

    Ok(Document {
      source_path,
      doc_type,
      nodes,
      metadata: DocumentMetadata {
        title,
        description,
        total_lines,
        total_nodes,
      },
    })
  }

  fn read_node<R: Read>(&mut self, r: &mut R) -> io::Result<Node> {
    let tag = read_u8(r)?;
    let span = read_span(r)?;
    let kind = self.read_kind(tag, r)?;
    let child_count = read_u32(r)? as usize;
    let children = (0..child_count)
      .map(|_| self.read_node(r))
      .collect::<io::Result<Vec<_>>>()?;
    Ok(Node {
      kind,
      span,
      children,
    })
  }

  fn read_kind<R: Read>(&mut self, tag: u8, r: &mut R) -> io::Result<NodeKind> {
    Ok(match tag {
      0 => NodeKind::Document,
      1 => NodeKind::Heading {
        level: read_u8(r)?,
        id: self.read_opt_str(r)?,
      },
      2 => NodeKind::Paragraph,
      3 => NodeKind::BlockQuote,
      4 => NodeKind::CodeBlock {
        language: self.read_opt_str(r)?,
        info: self.read_opt_str(r)?,
      },
      5 => NodeKind::FencedCodeBlock {
        language: self.read_opt_str(r)?,
        info: self.read_opt_str(r)?,
      },
      6 => NodeKind::IndentedCodeBlock,
      7 => NodeKind::HtmlBlock {
        block_type: read_u8(r)?,
      },
      8 => NodeKind::ThematicBreak,
      9 => NodeKind::List {
        ordered: read_u8(r)? != 0,
        tight: read_u8(r)? != 0,
        start: read_opt_u32(r)?,
      },
      10 => NodeKind::ListItem {
        marker: read_marker(r)?,
        checked: read_opt_bool(r)?,
      },
      11 => NodeKind::Table,
      12 => NodeKind::TableHead,
      13 => NodeKind::TableBody,
      14 => NodeKind::TableRow,
      15 => NodeKind::TableCell {
        alignment: u8_to_alignment(read_u8(r)?),
        is_header: read_u8(r)? != 0,
      },
      16 => NodeKind::Text {
        content: self.read_str(r)?,
      },
      17 => NodeKind::Emphasis,
      18 => NodeKind::Strong,
      19 => NodeKind::Strikethrough,
      20 => NodeKind::Code {
        content: self.read_str(r)?,
      },
      21 => NodeKind::Link {
        url: self.read_str(r)?,
        title: self.read_opt_str(r)?,
        ref_type: u8_to_ref_type(read_u8(r)?),
      },
      22 => NodeKind::Image {
        url: self.read_str(r)?,
        alt: self.read_str(r)?,
        title: self.read_opt_str(r)?,
      },
      23 => NodeKind::AutoLink {
        url: self.read_str(r)?,
      },
      24 => NodeKind::HardBreak,
      25 => NodeKind::SoftBreak,
      26 => NodeKind::HtmlInline {
        content: self.read_str(r)?,
      },
      27 => NodeKind::LinkReference {
        label: self.read_str(r)?,
        ref_type: u8_to_ref_type(read_u8(r)?),
      },
      28 => NodeKind::LinkDefinition {
        label: self.read_str(r)?,
        url: self.read_str(r)?,
        title: self.read_opt_str(r)?,
      },
      29 => NodeKind::FootnoteReference {
        label: self.read_str(r)?,
      },
      30 => NodeKind::FootnoteDefinition {
        label: self.read_str(r)?,
      },
      31 => NodeKind::TaskListMarker {
        checked: read_u8(r)? != 0,
      },
      32 => NodeKind::Emoji {
        shortcode: self.read_str(r)?,
      },
      33 => NodeKind::Mention {
        username: self.read_str(r)?,
      },
      34 => NodeKind::IssueReference {
        number: read_u32(r)?,
      },
      35 => NodeKind::DocComment {
        style: u8_to_doc_style(read_u8(r)?),
      },
      36 => NodeKind::DocTag {
        name: self.read_str(r)?,
        content: self.read_opt_str(r)?,
      },
      37 => NodeKind::DocParam {
        name: self.read_str(r)?,
        param_type: self.read_opt_str(r)?,
        description: self.read_opt_str(r)?,
      },
      38 => NodeKind::DocReturn {
        return_type: self.read_opt_str(r)?,
        description: self.read_opt_str(r)?,
      },
      39 => NodeKind::DocThrows {
        exception_type: self.read_str(r)?,
        description: self.read_opt_str(r)?,
      },
      40 => NodeKind::DocExample {
        content: self.read_str(r)?,
      },
      41 => NodeKind::DocSee {
        reference: self.read_str(r)?,
      },
      42 => NodeKind::DocDeprecated {
        message: self.read_opt_str(r)?,
      },
      43 => NodeKind::DocSince {
        version: self.read_str(r)?,
      },
      44 => NodeKind::DocAuthor {
        name: self.read_str(r)?,
      },
      45 => NodeKind::DocVersion {
        version: self.read_str(r)?,
      },
      46 => NodeKind::DocDescription {
        content: self.read_str(r)?,
      },
      47 => NodeKind::DocType {
        type_expr: self.read_str(r)?,
      },
      48 => NodeKind::DocProperty {
        name: self.read_str(r)?,
        prop_type: self.read_opt_str(r)?,
        description: self.read_opt_str(r)?,
      },
      49 => NodeKind::DocCallback {
        name: self.read_str(r)?,
      },
      50 => NodeKind::DocTypedef {
        name: self.read_str(r)?,
        type_expr: self.read_opt_str(r)?,
      },
      51 => NodeKind::CodeSpan {
        content: self.read_str(r)?,
      },
      52 => NodeKind::Frontmatter {
        format: u8_to_frontmatter_format(read_u8(r)?),
        content: self.read_str(r)?,
      },
      53 => NodeKind::MathInline {
        content: self.read_str(r)?,
      },
      54 => NodeKind::MathBlock {
        content: self.read_str(r)?,
      },
      55 => NodeKind::Footnote {
        label: self.read_str(r)?,
      },
      56 => NodeKind::DefinitionList,
      57 => NodeKind::DefinitionTerm,
      58 => NodeKind::DefinitionDescription,
      59 => NodeKind::AutoUrl {
        url: self.read_str(r)?,
      },
      60 => NodeKind::Alert {
        alert_type: u8_to_alert_type(read_u8(r)?),
      },
      61 => NodeKind::Steps,
      62 => NodeKind::Step,
      63 => NodeKind::Toc,
      64 => NodeKind::Tabs {
        names: {
          let count = read_u32(r)? as usize;
          let mut names = Vec::with_capacity(count);
          for _ in 0..count {
            names.push(self.read_str(r)?);
          }
          names
        },
      },
      65 => NodeKind::CodeBlockExt {
        language: self.read_opt_str(r)?,
        highlight: self.read_opt_str(r)?,
        plusdiff: self.read_opt_str(r)?,
        minusdiff: self.read_opt_str(r)?,
        linenumbers: read_u8(r)? != 0,
      },
      _ => {
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          "Unknown node tag",
        ))
      }
    })
  }

  fn read_str<R: Read>(&self, r: &mut R) -> io::Result<String> {
    let idx = read_u32(r)? as usize;
    Ok(self.strings.get(idx).cloned().unwrap_or_default())
  }

  fn read_opt_str<R: Read>(&self, r: &mut R) -> io::Result<Option<String>> {
    Ok(match read_u8(r)? {
      0 => None,
      _ => Some(self.read_str(r)?),
    })
  }
}

fn u8_to_alert_type(v: u8) -> AlertType {
  match v {
    0 => AlertType::Note,
    1 => AlertType::Tip,
    2 => AlertType::Important,
    3 => AlertType::Warning,
    4 => AlertType::Caution,
    _ => AlertType::Note,
  }
}

fn u8_to_frontmatter_format(v: u8) -> FrontmatterFormat {
  match v {
    0 => FrontmatterFormat::Yaml,
    1 => FrontmatterFormat::Toml,
    2 => FrontmatterFormat::Json,
    _ => FrontmatterFormat::Yaml,
  }
}
