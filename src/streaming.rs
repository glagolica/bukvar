//! Streaming parser for large files.
//!
//! Processes input in chunks to handle files that don't fit in memory.

use crate::ast::Document;
use std::io::{BufRead, BufReader, Read};

/// Buffer size for streaming (64KB)
#[allow(dead_code)]
const BUFFER_SIZE: usize = 64 * 1024;

/// Streaming markdown parser.
///
/// Processes input line by line, yielding blocks as they complete.
#[allow(dead_code)]
pub struct StreamingParser<R: Read> {
  reader: BufReader<R>,
  line_buffer: String,
  current_block: Vec<String>,
  line_num: usize,
  finished: bool,
}

#[allow(dead_code)]
impl<R: Read> StreamingParser<R> {
  /// Create a new streaming parser.
  pub fn new(reader: R) -> Self {
    Self {
      reader: BufReader::with_capacity(BUFFER_SIZE, reader),
      line_buffer: String::new(),
      current_block: Vec::new(),
      line_num: 0,
      finished: false,
    }
  }

  /// Read the next block of content.
  ///
  /// Returns None when input is exhausted.
  pub fn next_block(&mut self) -> Option<String> {
    if self.finished {
      return None;
    }

    loop {
      self.line_buffer.clear();
      match self.reader.read_line(&mut self.line_buffer) {
        Ok(0) => {
          // EOF reached
          self.finished = true;
          if !self.current_block.is_empty() {
            let block = self.current_block.join("\n");
            self.current_block.clear();
            return Some(block);
          }
          return None;
        }
        Ok(_) => {
          self.line_num += 1;
          let line = self
            .line_buffer
            .trim_end_matches('\n')
            .trim_end_matches('\r');

          // Empty line marks end of block
          if line.trim().is_empty() {
            if !self.current_block.is_empty() {
              let block = self.current_block.join("\n");
              self.current_block.clear();
              return Some(block);
            }
          } else {
            self.current_block.push(line.to_string());
          }
        }
        Err(_) => {
          self.finished = true;
          return None;
        }
      }
    }
  }

  /// Get current line number.
  pub fn line_num(&self) -> usize {
    self.line_num
  }
}

/// Parse a document from a reader in streaming mode.
///
/// This reads and parses the entire input but does so efficiently
/// by using buffered I/O.
pub fn parse_streaming<R: Read>(reader: R) -> Document {
  use crate::markdown::MarkdownParser;

  let mut content = String::new();
  let mut buf_reader = BufReader::with_capacity(BUFFER_SIZE, reader);
  let _ = buf_reader.read_to_string(&mut content);

  let mut parser = MarkdownParser::new(&content);
  parser.parse()
}

/// Iterator over blocks in streaming input.
#[allow(dead_code)]
pub struct BlockIterator<R: Read> {
  parser: StreamingParser<R>,
}

#[allow(dead_code)]
impl<R: Read> BlockIterator<R> {
  pub fn new(reader: R) -> Self {
    Self {
      parser: StreamingParser::new(reader),
    }
  }
}

impl<R: Read> Iterator for BlockIterator<R> {
  type Item = String;

  fn next(&mut self) -> Option<Self::Item> {
    self.parser.next_block()
  }
}

/// Create an iterator over blocks in a reader.
#[allow(dead_code)]
pub fn blocks<R: Read>(reader: R) -> BlockIterator<R> {
  BlockIterator::new(reader)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Cursor;

  #[test]
  fn test_buffer_size() {
    // Verify BUFFER_SIZE is reasonable (64KB)
    assert_eq!(BUFFER_SIZE, 64 * 1024);
  }

  #[test]
  fn test_streaming_parser_line_num() {
    let input = "Line one.\n\nLine two.";
    let reader = Cursor::new(input);
    let mut parser = StreamingParser::new(reader);

    // Initial line num is 0
    assert_eq!(parser.line_num(), 0);

    // After first block (line 1)
    let _ = parser.next_block();
    assert!(parser.line_num() > 0);
  }

  #[test]
  fn test_streaming_blocks() {
    let input = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    let reader = Cursor::new(input);

    let blocks: Vec<String> = blocks(reader).collect();
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0], "First paragraph.");
    assert_eq!(blocks[1], "Second paragraph.");
    assert_eq!(blocks[2], "Third paragraph.");
  }

  #[test]
  fn test_streaming_empty_input() {
    let reader = Cursor::new("");
    let blocks: Vec<String> = blocks(reader).collect();
    assert_eq!(blocks.len(), 0);
  }

  #[test]
  fn test_streaming_single_block() {
    let input = "Single paragraph\nwith multiple lines.";
    let reader = Cursor::new(input);

    let blocks: Vec<String> = blocks(reader).collect();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0], "Single paragraph\nwith multiple lines.");
  }

  #[test]
  fn test_block_iterator() {
    let input = "Block 1.\n\nBlock 2.";
    let reader = Cursor::new(input);
    let iter = BlockIterator::new(reader);

    let collected: Vec<String> = iter.collect();
    assert_eq!(collected.len(), 2);
  }

  #[test]
  fn test_parse_streaming() {
    let input = "# Hello\n\nThis is a paragraph.";
    let reader = Cursor::new(input);
    let doc = parse_streaming(reader);

    assert!(doc.metadata.total_nodes > 0);
  }
}
