//! Type decoding for DAST binary format.

use crate::ast::*;

pub fn u8_to_doc_type(v: u8) -> DocumentType {
  match v {
    0 => DocumentType::Markdown,
    1 => DocumentType::JavaScript,
    2 => DocumentType::TypeScript,
    3 => DocumentType::Java,
    _ => DocumentType::Python,
  }
}

pub fn u8_to_alignment(v: u8) -> Alignment {
  match v {
    0 => Alignment::None,
    1 => Alignment::Left,
    2 => Alignment::Center,
    _ => Alignment::Right,
  }
}

pub fn u8_to_ref_type(v: u8) -> ReferenceType {
  match v {
    0 => ReferenceType::Full,
    1 => ReferenceType::Collapsed,
    _ => ReferenceType::Shortcut,
  }
}

pub fn u8_to_doc_style(v: u8) -> DocStyle {
  match v {
    0 => DocStyle::JSDoc,
    1 => DocStyle::JavaDoc,
    2 => DocStyle::PyDoc,
    3 => DocStyle::PyDocGoogle,
    _ => DocStyle::PyDocNumpy,
  }
}
