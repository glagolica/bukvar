//! Error types.

use std::fmt;
use std::io;

/// Position in source for error reporting.
#[derive(Debug, Clone, Default)]
pub struct SourcePosition {
  pub line: usize,
  pub column: usize,
  #[allow(dead_code)]
  pub offset: usize,
}

impl SourcePosition {
  #[allow(dead_code)]
  pub fn new(line: usize, column: usize, offset: usize) -> Self {
    Self {
      line,
      column,
      offset,
    }
  }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
  Io(io::Error),
  InvalidUtf8(std::string::FromUtf8Error),
  InvalidFormat {
    message: String,
    pos: Option<SourcePosition>,
  },
  UnexpectedEof {
    pos: Option<SourcePosition>,
  },
  UnexpectedToken {
    expected: String,
    found: String,
    pos: Option<SourcePosition>,
  },
  UnclosedElement {
    element: String,
    pos: Option<SourcePosition>,
  },
}

fn fmt_pos(pos: &Option<SourcePosition>) -> String {
  match pos {
    Some(p) => format!(" at line {}, column {}", p.line, p.column),
    None => String::new(),
  }
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ParseError::Io(e) => write!(f, "IO error: {}", e),
      ParseError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
      ParseError::InvalidFormat { message, pos } => {
        write!(f, "Invalid format: {}{}", message, fmt_pos(pos))
      }
      ParseError::UnexpectedEof { pos } => {
        write!(f, "Unexpected end of file{}", fmt_pos(pos))
      }
      ParseError::UnexpectedToken {
        expected,
        found,
        pos,
      } => {
        write!(f, "Expected {}, found {}{}", expected, found, fmt_pos(pos))
      }
      ParseError::UnclosedElement { element, pos } => {
        write!(f, "Unclosed {}{}", element, fmt_pos(pos))
      }
    }
  }
}

impl std::error::Error for ParseError {}

impl From<io::Error> for ParseError {
  fn from(e: io::Error) -> Self {
    ParseError::Io(e)
  }
}

impl From<std::string::FromUtf8Error> for ParseError {
  fn from(e: std::string::FromUtf8Error) -> Self {
    ParseError::InvalidUtf8(e)
  }
}

/// Result type using ParseError.
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, ParseError>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_source_position_new() {
    let pos = SourcePosition::new(10, 5, 100);
    assert_eq!(pos.line, 10);
    assert_eq!(pos.column, 5);
    assert_eq!(pos.offset, 100);
  }

  #[test]
  fn test_source_position_default() {
    let pos = SourcePosition::default();
    assert_eq!(pos.line, 0);
    assert_eq!(pos.column, 0);
    assert_eq!(pos.offset, 0);
  }

  #[test]
  fn test_parse_error_invalid_format() {
    let err = ParseError::InvalidFormat {
      message: "bad syntax".to_string(),
      pos: Some(SourcePosition::new(5, 10, 50)),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid format"));
    assert!(msg.contains("bad syntax"));
    assert!(msg.contains("line 5"));
    assert!(msg.contains("column 10"));
  }

  #[test]
  fn test_parse_error_unexpected_eof() {
    let err = ParseError::UnexpectedEof {
      pos: Some(SourcePosition::new(100, 1, 500)),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Unexpected end of file"));
    assert!(msg.contains("line 100"));
  }

  #[test]
  fn test_parse_error_unexpected_token() {
    let err = ParseError::UnexpectedToken {
      expected: "number".to_string(),
      found: "string".to_string(),
      pos: Some(SourcePosition::new(3, 15, 30)),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Expected number"));
    assert!(msg.contains("found string"));
  }

  #[test]
  fn test_parse_error_unclosed_element() {
    let err = ParseError::UnclosedElement {
      element: "code block".to_string(),
      pos: None,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Unclosed code block"));
    // No position info since pos is None
    assert!(!msg.contains("line"));
  }

  #[test]
  fn test_parse_error_io() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = ParseError::Io(io_err);
    let msg = format!("{}", err);
    assert!(msg.contains("IO error"));
    assert!(msg.contains("file not found"));
  }

  #[test]
  fn test_result_type() {
    fn returns_ok() -> Result<i32> {
      Ok(42)
    }

    fn returns_err() -> Result<i32> {
      Err(ParseError::UnexpectedEof { pos: None })
    }

    assert!(returns_ok().is_ok());
    assert!(returns_err().is_err());
  }
}
