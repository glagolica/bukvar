//! Helper write functions for DAST binary format.

use crate::ast::{ListMarker, Span};
use std::io::{self, Write};

pub fn write_span<W: Write>(span: &Span, w: &mut W) -> io::Result<()> {
  w.write_all(&(span.start as u32).to_le_bytes())?;
  w.write_all(&(span.end as u32).to_le_bytes())?;
  w.write_all(&(span.line as u32).to_le_bytes())?;
  w.write_all(&(span.column as u32).to_le_bytes())
}

pub fn write_opt_u32<W: Write>(v: &Option<u32>, w: &mut W) -> io::Result<()> {
  match v {
    Some(n) => {
      w.write_all(&[1])?;
      w.write_all(&n.to_le_bytes())
    }
    None => w.write_all(&[0]),
  }
}

pub fn write_opt_bool<W: Write>(v: &Option<bool>, w: &mut W) -> io::Result<()> {
  match v {
    Some(b) => w.write_all(&[1, *b as u8]),
    None => w.write_all(&[0]),
  }
}

pub fn write_marker<W: Write>(m: &ListMarker, w: &mut W) -> io::Result<()> {
  match m {
    ListMarker::Bullet(c) => w.write_all(&[0, *c as u8]),
    ListMarker::Ordered(c) => w.write_all(&[1, *c]),
  }
}
