//! Type encoding for DAST binary format.

use crate::ast::*;

pub fn node_kind_u8(k: &NodeKind) -> u8 {
  match k {
    NodeKind::Document => 0,
    NodeKind::Heading { .. } => 1,
    NodeKind::Paragraph => 2,
    NodeKind::BlockQuote => 3,
    NodeKind::CodeBlock { .. } => 4,
    NodeKind::FencedCodeBlock { .. } => 5,
    NodeKind::IndentedCodeBlock => 6,
    NodeKind::HtmlBlock { .. } => 7,
    NodeKind::ThematicBreak => 8,
    NodeKind::List { .. } => 9,
    NodeKind::ListItem { .. } => 10,
    NodeKind::Table => 11,
    NodeKind::TableHead => 12,
    NodeKind::TableBody => 13,
    NodeKind::TableRow => 14,
    NodeKind::TableCell { .. } => 15,
    NodeKind::Text { .. } => 16,
    NodeKind::Emphasis => 17,
    NodeKind::Strong => 18,
    NodeKind::Strikethrough => 19,
    NodeKind::Code { .. } => 20,
    NodeKind::Link { .. } => 21,
    NodeKind::Image { .. } => 22,
    NodeKind::AutoLink { .. } => 23,
    NodeKind::HardBreak => 24,
    NodeKind::SoftBreak => 25,
    NodeKind::HtmlInline { .. } => 26,
    NodeKind::LinkReference { .. } => 27,
    NodeKind::LinkDefinition { .. } => 28,
    NodeKind::FootnoteReference { .. } => 29,
    NodeKind::FootnoteDefinition { .. } => 30,
    NodeKind::TaskListMarker { .. } => 31,
    NodeKind::Emoji { .. } => 32,
    NodeKind::Mention { .. } => 33,
    NodeKind::IssueReference { .. } => 34,
    NodeKind::DocComment { .. } => 35,
    NodeKind::DocTag { .. } => 36,
    NodeKind::DocParam { .. } => 37,
    NodeKind::DocReturn { .. } => 38,
    NodeKind::DocThrows { .. } => 39,
    NodeKind::DocExample { .. } => 40,
    NodeKind::DocSee { .. } => 41,
    NodeKind::DocDeprecated { .. } => 42,
    NodeKind::DocSince { .. } => 43,
    NodeKind::DocAuthor { .. } => 44,
    NodeKind::DocVersion { .. } => 45,
    NodeKind::DocDescription { .. } => 46,
    NodeKind::DocType { .. } => 47,
    NodeKind::DocProperty { .. } => 48,
    NodeKind::DocCallback { .. } => 49,
    NodeKind::DocTypedef { .. } => 50,
    NodeKind::CodeSpan { .. } => 51,
    NodeKind::Frontmatter { .. } => 52,
    NodeKind::MathInline { .. } => 53,
    NodeKind::MathBlock { .. } => 54,
    NodeKind::Footnote { .. } => 55,
    NodeKind::DefinitionList => 56,
    NodeKind::DefinitionTerm => 57,
    NodeKind::DefinitionDescription => 58,
    NodeKind::AutoUrl { .. } => 59,
  }
}

pub fn doc_type_u8(dt: &DocumentType) -> u8 {
  match dt {
    DocumentType::Markdown => 0,
    DocumentType::JavaScript => 1,
    DocumentType::TypeScript => 2,
    DocumentType::Java => 3,
    DocumentType::Python => 4,
  }
}

pub fn alignment_u8(a: &Alignment) -> u8 {
  match a {
    Alignment::None => 0,
    Alignment::Left => 1,
    Alignment::Center => 2,
    Alignment::Right => 3,
  }
}

pub fn ref_type_u8(rt: &ReferenceType) -> u8 {
  match rt {
    ReferenceType::Full => 0,
    ReferenceType::Collapsed => 1,
    ReferenceType::Shortcut => 2,
  }
}

pub fn doc_style_u8(ds: &DocStyle) -> u8 {
  match ds {
    DocStyle::JSDoc => 0,
    DocStyle::JavaDoc => 1,
    DocStyle::PyDoc => 2,
    DocStyle::PyDocGoogle => 3,
    DocStyle::PyDocNumpy => 4,
  }
}
