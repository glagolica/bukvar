//! Documentation comment parsers for JSDoc, JavaDoc, and PyDoc

pub mod javadoc;
pub mod jsdoc;
pub mod pydoc;

pub use javadoc::JavaDocParser;
pub use jsdoc::JsDocParser;
pub use pydoc::PyDocParser;

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::{DocumentType, NodeKind};

  #[test]
  fn test_jsdoc_basic() {
    let input = r#"
/**
 * This is a description
 * @param {string} name - The name
 * @returns {void}
 */
function test() {}
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.doc_type, DocumentType::JavaScript);
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_multiple_comments() {
    let input = r#"
/** First comment */
function first() {}

/** Second comment */
function second() {}
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 2);
  }

  #[test]
  fn test_jsdoc_empty() {
    let input = "function test() {}";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_skip_normal_comments() {
    let input = r#"
/* This is not a JSDoc comment */
// Neither is this
/** But this is */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 1);
  }

  #[test]
  fn test_javadoc_basic() {
    let input = r#"
/**
 * This is a description
 * @param name The name parameter
 * @return The result
 */
public void test() {}
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.doc_type, DocumentType::Java);
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_with_throws() {
    let input = r#"
/**
 * Description
 * @param x Input value
 * @throws IllegalArgumentException if x is negative
 * @return Result
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_empty() {
    let input = "public class Test {}";
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_google_style() {
    let input = r#"
def test():
    """This is a description.

    Args:
        name: The name parameter
        value: The value

    Returns:
        The result
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.doc_type, DocumentType::Python);
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_numpy_style() {
    let input = r#"
def test():
    """
    This is a description.

    Parameters
    ----------
    name : str
        The name parameter

    Returns
    -------
    str
        The result
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_empty() {
    let input = "def test(): pass";
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_single_line() {
    let input = r#"
def test():
    """Single line docstring."""
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_parsers_doc_comment_node() {
    let input = "/** Test */";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    if !doc.nodes.is_empty() {
      assert!(matches!(doc.nodes[0].kind, NodeKind::DocComment { .. }));
    }
  }
}
