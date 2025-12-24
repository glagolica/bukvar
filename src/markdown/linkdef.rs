//! Link definition parsing.

use super::scanner::Scanner;

/// Link reference: `[label]: url "title"`
#[derive(Debug, Clone)]
pub struct LinkDef {
  pub label: String,
  pub url: String,
  pub title: Option<String>,
}

/// Collect all link definitions from the document.
pub fn collect_definitions(scanner: &mut Scanner) -> Vec<LinkDef> {
  let mut defs = Vec::new();
  while !scanner.is_eof() {
    let start = scanner.pos();
    scanner.skip_whitespace_inline();

    if scanner.check(b'[') {
      if let Some(def) = try_parse(scanner) {
        defs.push(def);
        continue;
      }
    }

    scanner.set_pos(start);
    scanner.skip_line();
  }
  defs
}

fn try_parse(scanner: &mut Scanner) -> Option<LinkDef> {
  if !scanner.consume(b'[') {
    return None;
  }

  let label = scanner.scan_until(b']')?;
  scanner.advance(); // ]

  if !scanner.consume(b':') {
    return None;
  }

  scanner.skip_whitespace_inline();
  scanner.consume(b'\n');
  scanner.skip_whitespace_inline();

  let url = parse_url(scanner)?;
  scanner.skip_whitespace_inline();
  let title = parse_title(scanner);

  Some(LinkDef { label, url, title })
}

fn parse_url(scanner: &mut Scanner) -> Option<String> {
  if scanner.consume(b'<') {
    let url = scanner.scan_until(b'>')?;
    scanner.advance();
    Some(url)
  } else {
    Some(scanner.scan_non_whitespace())
  }
}

fn parse_title(scanner: &mut Scanner) -> Option<String> {
  let delim = scanner.peek()?;
  if delim != b'"' && delim != b'\'' && delim != b'(' {
    return None;
  }

  let end = if delim == b'(' { b')' } else { delim };
  scanner.advance();
  let title = scanner.scan_until(end)?;
  scanner.advance();
  Some(title)
}
