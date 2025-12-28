//! Supporting types for AST nodes
//!
//! These types are used by [`NodeKind`](super::NodeKind) variants
//! to represent list markers, table alignment, link references, etc.

use std::fmt;

/// List marker type for ordered and unordered lists.
///
/// # Examples
/// - Bullet: `-`, `*`, `+`
/// - Ordered: `1.`, `2)`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMarker {
  /// Bullet list marker: -, *, +
  Bullet(char),
  /// Ordered list marker delimiter: ), .
  #[allow(dead_code)] // Part of public API
  Ordered(u8),
}

/// Table cell alignment (GFM tables).
///
/// Determined by colons in the separator row:
/// - `---` = None (default)
/// - `:--` = Left
/// - `:-:` = Center
/// - `--:` = Right
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Variants are part of public API
pub enum Alignment {
  #[default]
  None,
  Left,
  Center,
  Right,
}

/// Link reference type for reference-style links.
///
/// # Reference Styles
/// - Full: `[text][label]`
/// - Collapsed: `[label][]`
/// - Shortcut: `[label]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
  /// Full reference: `[text][label]`
  Full,
  /// Collapsed reference: `[label][]`
  #[allow(dead_code)]
  Collapsed,
  /// Shortcut reference: `[label]`
  Shortcut,
}

/// Documentation comment style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocStyle {
  JSDoc,
  JavaDoc,
  PyDoc,
  PyDocGoogle,
  PyDocNumpy,
}

impl fmt::Display for DocStyle {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::JSDoc => write!(f, "JSDoc"),
      Self::JavaDoc => write!(f, "JavaDoc"),
      Self::PyDoc => write!(f, "PyDoc"),
      Self::PyDocGoogle => write!(f, "PyDoc (Google)"),
      Self::PyDocNumpy => write!(f, "PyDoc (NumPy)"),
    }
  }
}

/// Alert type for GitHub-style blockquote callouts
///
/// Used with `> [!TYPE]` syntax in blockquotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertType {
  /// Highlights information users should take into account
  Note,
  /// Optional information to help users be more successful
  Tip,
  /// Crucial information necessary for users to succeed
  Important,
  /// Critical content demanding immediate user attention
  Warning,
  /// Negative potential consequences of an action
  Caution,
}

impl fmt::Display for AlertType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Note => write!(f, "NOTE"),
      Self::Tip => write!(f, "TIP"),
      Self::Important => write!(f, "IMPORTANT"),
      Self::Warning => write!(f, "WARNING"),
      Self::Caution => write!(f, "CAUTION"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_list_marker() {
    let bullet = ListMarker::Bullet('-');
    assert!(matches!(bullet, ListMarker::Bullet('-')));

    let ordered = ListMarker::Ordered(b'.');
    assert!(matches!(ordered, ListMarker::Ordered(b'.')));
  }

  #[test]
  fn test_alignment_default() {
    assert_eq!(Alignment::default(), Alignment::None);
  }

  #[test]
  fn test_doc_style_display() {
    assert_eq!(format!("{}", DocStyle::JSDoc), "JSDoc");
    assert_eq!(format!("{}", DocStyle::PyDocGoogle), "PyDoc (Google)");
  }

  #[test]
  fn test_alert_type_display() {
    assert_eq!(format!("{}", AlertType::Note), "NOTE");
    assert_eq!(format!("{}", AlertType::Tip), "TIP");
    assert_eq!(format!("{}", AlertType::Important), "IMPORTANT");
    assert_eq!(format!("{}", AlertType::Warning), "WARNING");
    assert_eq!(format!("{}", AlertType::Caution), "CAUTION");
  }
}
