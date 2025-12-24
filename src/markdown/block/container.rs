//! Container block elements: blockquotes, lists.

use super::BlockParser;
use crate::ast::{ListMarker, Node, NodeKind, Span};

impl<'a, 'b> BlockParser<'a, 'b> {
  pub fn parse_blockquote(&mut self, line: usize, col: usize) -> Node {
    let start = self.scanner.pos();
    let content = self.collect_blockquote_content();

    let mut inner = super::super::MarkdownParser::new(&content);
    let inner_doc = inner.parse();

    Node::with_children(
      NodeKind::BlockQuote,
      Span::new(start, self.scanner.pos(), line, col),
      inner_doc.nodes,
    )
  }

  fn collect_blockquote_content(&mut self) -> String {
    let mut content = String::new();

    while !self.scanner.is_eof() && self.scanner.consume(b'>') {
      self.scanner.consume(b' ');
      self.append_line_to(&mut content);
      content.push('\n');
      self.scanner.consume(b'\n');
    }

    content
  }

  fn append_line_to(&mut self, content: &mut String) {
    while !self.scanner.is_eof() && !self.scanner.check(b'\n') {
      if let Some(ch) = self
        .scanner
        .slice(self.scanner.pos(), self.scanner.pos() + 1)
        .chars()
        .next()
      {
        content.push(ch);
      }
      self.scanner.advance();
    }
  }

  pub fn parse_list(&mut self, ordered: bool) -> Node {
    let start = self.scanner.pos();
    let items = self.collect_list_items();

    Node::with_children(
      NodeKind::List {
        ordered,
        start: None,
        tight: true,
      },
      Span::new(start, self.scanner.pos(), 0, 0),
      items,
    )
  }

  fn collect_list_items(&mut self) -> Vec<Node> {
    let mut items = Vec::new();

    while !self.scanner.is_eof() {
      if !self.is_list_marker() {
        break;
      }

      self.scanner.advance(); // skip marker
      self.scanner.consume(b' ');

      items.push(self.parse_list_item());
    }

    items
  }

  fn is_list_marker(&self) -> bool {
    matches!(self.scanner.peek(), Some(b'-' | b'*' | b'+'))
  }

  fn parse_list_item(&mut self) -> Node {
    let item_start = self.scanner.pos();
    let content = self.scan_line_content();
    self.scanner.consume(b'\n');

    let inline = self.parse_inline(&content);

    Node::with_children(
      NodeKind::ListItem {
        marker: ListMarker::Bullet('-'),
        checked: None,
      },
      Span::new(item_start, self.scanner.pos(), 0, 0),
      vec![Node::with_children(
        NodeKind::Paragraph,
        Span::empty(),
        inline,
      )],
    )
  }
}
