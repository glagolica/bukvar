//! Helper read functions for DAST binary format.

use crate::ast::{ListMarker, Span};
use std::io::{self, Read};

pub fn read_u8<R: Read>(r: &mut R) -> io::Result<u8> {
  let mut b = [0u8; 1];
  r.read_exact(&mut b)?;
  Ok(b[0])
}

pub fn read_u32<R: Read>(r: &mut R) -> io::Result<u32> {
  let mut b = [0u8; 4];
  r.read_exact(&mut b)?;
  Ok(u32::from_le_bytes(b))
}

pub fn read_span<R: Read>(r: &mut R) -> io::Result<Span> {
  Ok(Span::new(
    read_u32(r)? as usize,
    read_u32(r)? as usize,
    read_u32(r)? as usize,
    read_u32(r)? as usize,
  ))
}

pub fn read_opt_u32<R: Read>(r: &mut R) -> io::Result<Option<u32>> {
  Ok(match read_u8(r)? {
    0 => None,
    _ => Some(read_u32(r)?),
  })
}

pub fn read_opt_bool<R: Read>(r: &mut R) -> io::Result<Option<bool>> {
  Ok(match read_u8(r)? {
    0 => None,
    _ => Some(read_u8(r)? != 0),
  })
}

pub fn read_marker<R: Read>(r: &mut R) -> io::Result<ListMarker> {
  let t = read_u8(r)?;
  let c = read_u8(r)?;
  Ok(match t {
    0 => ListMarker::Bullet(c as char),
    _ => ListMarker::Ordered(c),
  })
}
