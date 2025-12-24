//! Parsed documentation item.

/// Represents a parsed documentation item (parameter, return, etc.)
pub struct DocItem {
  pub name: String,
  pub item_type: Option<String>,
  pub description: Option<String>,
}

impl DocItem {
  pub fn new(name: String, item_type: Option<String>, description: Option<String>) -> Self {
    Self {
      name,
      item_type,
      description,
    }
  }
}
