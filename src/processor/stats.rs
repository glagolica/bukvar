//! Processing statistics.

use crate::ast::DocumentType;

#[derive(Debug, Default)]
pub struct ProcessingStats {
  pub markdown_files: usize,
  pub js_files: usize,
  pub java_files: usize,
  pub python_files: usize,
  pub total_nodes: usize,
  pub errors: usize,
}

impl ProcessingStats {
  pub fn total_files(&self) -> usize {
    self.markdown_files + self.js_files + self.java_files + self.python_files
  }

  pub fn add_file(&mut self, doc_type: DocumentType, node_count: usize) {
    match doc_type {
      DocumentType::Markdown => self.markdown_files += 1,
      DocumentType::JavaScript | DocumentType::TypeScript => self.js_files += 1,
      DocumentType::Java => self.java_files += 1,
      DocumentType::Python => self.python_files += 1,
    }
    self.total_nodes += node_count;
  }
}
