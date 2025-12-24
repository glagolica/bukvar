//! NodeKind JSON serialization.

use super::esc;
use crate::ast::*;

pub fn write_kind(out: &mut String, kind: &NodeKind) {
  out.push('{');
  match kind {
    NodeKind::Document => out.push_str("\"type\":\"Document\""),
    NodeKind::Heading { level, id } => {
      out.push_str(&format!("\"type\":\"Heading\",\"level\":{}", level));
      if let Some(id) = id.as_ref() {
        out.push_str(&format!(",\"id\":\"{}\"", esc(id)));
      }
    }
    NodeKind::Paragraph => out.push_str("\"type\":\"Paragraph\""),
    NodeKind::BlockQuote => out.push_str("\"type\":\"BlockQuote\""),
    NodeKind::CodeBlock { language, info } | NodeKind::FencedCodeBlock { language, info } => {
      out.push_str("\"type\":\"CodeBlock\"");
      if let Some(l) = language.as_ref() {
        out.push_str(&format!(",\"language\":\"{}\"", esc(l)));
      }
      if let Some(i) = info.as_ref() {
        out.push_str(&format!(",\"info\":\"{}\"", esc(i)));
      }
    }
    NodeKind::IndentedCodeBlock => out.push_str("\"type\":\"IndentedCodeBlock\""),
    NodeKind::HtmlBlock { block_type } => {
      out.push_str(&format!(
        "\"type\":\"HtmlBlock\",\"block_type\":{}",
        block_type
      ));
    }
    NodeKind::ThematicBreak => out.push_str("\"type\":\"ThematicBreak\""),
    NodeKind::List {
      ordered,
      start,
      tight,
    } => {
      out.push_str(&format!(
        "\"type\":\"List\",\"ordered\":{},\"tight\":{}",
        ordered, tight
      ));
      if let Some(s) = start {
        out.push_str(&format!(",\"start\":{}", s));
      }
    }
    NodeKind::ListItem { marker, checked } => {
      out.push_str(&format!(
        "\"type\":\"ListItem\",\"marker\":\"{:?}\"",
        marker
      ));
      if let Some(c) = checked {
        out.push_str(&format!(",\"checked\":{}", c));
      }
    }
    NodeKind::Table => out.push_str("\"type\":\"Table\""),
    NodeKind::TableHead => out.push_str("\"type\":\"TableHead\""),
    NodeKind::TableBody => out.push_str("\"type\":\"TableBody\""),
    NodeKind::TableRow => out.push_str("\"type\":\"TableRow\""),
    NodeKind::TableCell {
      alignment,
      is_header,
    } => {
      out.push_str(&format!(
        "\"type\":\"TableCell\",\"alignment\":\"{:?}\",\"is_header\":{}",
        alignment, is_header
      ));
    }
    NodeKind::Text { content } => out.push_str(&format!(
      "\"type\":\"Text\",\"content\":\"{}\"",
      esc(content)
    )),
    NodeKind::Emphasis => out.push_str("\"type\":\"Emphasis\""),
    NodeKind::Strong => out.push_str("\"type\":\"Strong\""),
    NodeKind::Strikethrough => out.push_str("\"type\":\"Strikethrough\""),
    NodeKind::Code { content } | NodeKind::CodeSpan { content } => {
      out.push_str(&format!(
        "\"type\":\"Code\",\"content\":\"{}\"",
        esc(content)
      ));
    }
    NodeKind::Link {
      url,
      title,
      ref_type,
    } => {
      out.push_str(&format!("\"type\":\"Link\",\"url\":\"{}\"", esc(url)));
      if let Some(t) = title.as_ref() {
        out.push_str(&format!(",\"title\":\"{}\"", esc(t)));
      }
      out.push_str(&format!(",\"ref_type\":\"{:?}\"", ref_type));
    }
    NodeKind::Image { url, alt, title } => {
      out.push_str(&format!(
        "\"type\":\"Image\",\"url\":\"{}\",\"alt\":\"{}\"",
        esc(url),
        esc(alt)
      ));
      if let Some(t) = title.as_ref() {
        out.push_str(&format!(",\"title\":\"{}\"", esc(t)));
      }
    }
    NodeKind::AutoLink { url } => {
      out.push_str(&format!("\"type\":\"AutoLink\",\"url\":\"{}\"", esc(url)))
    }
    NodeKind::HardBreak => out.push_str("\"type\":\"HardBreak\""),
    NodeKind::SoftBreak => out.push_str("\"type\":\"SoftBreak\""),
    NodeKind::HtmlInline { content } => {
      out.push_str(&format!(
        "\"type\":\"HtmlInline\",\"content\":\"{}\"",
        esc(content)
      ));
    }
    NodeKind::DocComment { style } => out.push_str(&format!(
      "\"type\":\"DocComment\",\"style\":\"{:?}\"",
      style
    )),
    NodeKind::DocTag { name, content } => {
      out.push_str(&format!("\"type\":\"DocTag\",\"name\":\"{}\"", esc(name)));
      if let Some(c) = content.as_ref() {
        out.push_str(&format!(",\"content\":\"{}\"", esc(c)));
      }
    }
    NodeKind::DocParam {
      name,
      param_type,
      description,
    } => {
      out.push_str(&format!("\"type\":\"DocParam\",\"name\":\"{}\"", esc(name)));
      if let Some(t) = param_type.as_ref() {
        out.push_str(&format!(",\"param_type\":\"{}\"", esc(t)));
      }
      if let Some(d) = description.as_ref() {
        out.push_str(&format!(",\"description\":\"{}\"", esc(d)));
      }
    }
    NodeKind::Frontmatter { format, content } => {
      out.push_str(&format!(
        "\"type\":\"Frontmatter\",\"format\":\"{:?}\",\"content\":\"{}\"",
        format,
        esc(content)
      ));
    }
    NodeKind::MathInline { content } => out.push_str(&format!(
      "\"type\":\"MathInline\",\"content\":\"{}\"",
      esc(content)
    )),
    NodeKind::MathBlock { content } => out.push_str(&format!(
      "\"type\":\"MathBlock\",\"content\":\"{}\"",
      esc(content)
    )),
    NodeKind::Footnote { label } => out.push_str(&format!(
      "\"type\":\"Footnote\",\"label\":\"{}\"",
      esc(label)
    )),
    NodeKind::DefinitionList => out.push_str("\"type\":\"DefinitionList\""),
    NodeKind::DefinitionTerm => out.push_str("\"type\":\"DefinitionTerm\""),
    NodeKind::DefinitionDescription => out.push_str("\"type\":\"DefinitionDescription\""),
    NodeKind::AutoUrl { url } => {
      out.push_str(&format!("\"type\":\"AutoUrl\",\"url\":\"{}\"", esc(url)))
    }
    _ => out.push_str(&format!("\"type\":\"{:?}\"", std::mem::discriminant(kind))),
  }
  out.push('}');
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_write_document() {
    let mut out = String::new();
    write_kind(&mut out, &NodeKind::Document);
    assert_eq!(out, "{\"type\":\"Document\"}");
  }

  #[test]
  fn test_write_heading() {
    let mut out = String::new();
    write_kind(&mut out, &NodeKind::Heading { level: 2, id: None });
    assert!(out.contains("\"type\":\"Heading\""));
    assert!(out.contains("\"level\":2"));
  }

  #[test]
  fn test_write_heading_with_id() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::Heading {
        level: 1,
        id: Some("intro".to_string()),
      },
    );
    assert!(out.contains("\"id\":\"intro\""));
  }

  #[test]
  fn test_write_paragraph() {
    let mut out = String::new();
    write_kind(&mut out, &NodeKind::Paragraph);
    assert_eq!(out, "{\"type\":\"Paragraph\"}");
  }

  #[test]
  fn test_write_code_block() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::CodeBlock {
        language: Some("rust".to_string()),
        info: Some("example".to_string()),
      },
    );
    assert!(out.contains("\"type\":\"CodeBlock\""));
    assert!(out.contains("\"language\":\"rust\""));
    assert!(out.contains("\"info\":\"example\""));
  }

  #[test]
  fn test_write_list() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::List {
        ordered: true,
        start: Some(5),
        tight: false,
      },
    );
    assert!(out.contains("\"ordered\":true"));
    assert!(out.contains("\"tight\":false"));
    assert!(out.contains("\"start\":5"));
  }

  #[test]
  fn test_write_text() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::Text {
        content: "hello".to_string(),
      },
    );
    assert!(out.contains("\"type\":\"Text\""));
    assert!(out.contains("\"content\":\"hello\""));
  }

  #[test]
  fn test_write_link() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::Link {
        url: "https://example.com".to_string(),
        title: Some("Example".to_string()),
        ref_type: ReferenceType::Full,
      },
    );
    assert!(out.contains("\"url\":\"https://example.com\""));
    assert!(out.contains("\"title\":\"Example\""));
  }

  #[test]
  fn test_write_image() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::Image {
        url: "img.png".to_string(),
        alt: "Alt text".to_string(),
        title: None,
      },
    );
    assert!(out.contains("\"url\":\"img.png\""));
    assert!(out.contains("\"alt\":\"Alt text\""));
  }

  #[test]
  fn test_write_emphasis() {
    let mut out = String::new();
    write_kind(&mut out, &NodeKind::Emphasis);
    assert_eq!(out, "{\"type\":\"Emphasis\"}");
  }

  #[test]
  fn test_write_strong() {
    let mut out = String::new();
    write_kind(&mut out, &NodeKind::Strong);
    assert_eq!(out, "{\"type\":\"Strong\"}");
  }

  #[test]
  fn test_write_strikethrough() {
    let mut out = String::new();
    write_kind(&mut out, &NodeKind::Strikethrough);
    assert_eq!(out, "{\"type\":\"Strikethrough\"}");
  }

  #[test]
  fn test_write_table_cell() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::TableCell {
        alignment: Alignment::Center,
        is_header: true,
      },
    );
    assert!(out.contains("\"alignment\":\"Center\""));
    assert!(out.contains("\"is_header\":true"));
  }

  #[test]
  fn test_write_frontmatter() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::Frontmatter {
        format: FrontmatterFormat::Yaml,
        content: "title: Test".to_string(),
      },
    );
    assert!(out.contains("\"type\":\"Frontmatter\""));
    assert!(out.contains("\"format\":\"Yaml\""));
  }

  #[test]
  fn test_write_math_inline() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::MathInline {
        content: "x^2".to_string(),
      },
    );
    assert!(out.contains("\"type\":\"MathInline\""));
    assert!(out.contains("\"content\":\"x^2\""));
  }

  #[test]
  fn test_write_math_block() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::MathBlock {
        content: "\\sum".to_string(),
      },
    );
    assert!(out.contains("\"type\":\"MathBlock\""));
  }

  #[test]
  fn test_write_doc_param() {
    let mut out = String::new();
    write_kind(
      &mut out,
      &NodeKind::DocParam {
        name: "x".to_string(),
        param_type: Some("int".to_string()),
        description: Some("The value".to_string()),
      },
    );
    assert!(out.contains("\"name\":\"x\""));
    assert!(out.contains("\"param_type\":\"int\""));
    assert!(out.contains("\"description\":\"The value\""));
  }
}
