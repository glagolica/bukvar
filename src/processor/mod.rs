//! File processor - handles directory traversal and parallel processing

mod files;
mod parse;
mod stats;
mod write;

use crate::cli::Args;
use std::fs;
use std::path::{Path, PathBuf};

pub use self::files::collect_files;
pub use self::stats::ProcessingStats;

/// Main file processor.
pub struct FileProcessor {
  args: Args,
  files: Vec<PathBuf>,
}

impl FileProcessor {
  pub fn new(args: &Args) -> Result<Self, String> {
    validate_input(args)?;
    let files = collect_files(&args.input, &args.extensions, args.recursive)?;
    validate_files(&files, args)?;
    Ok(Self {
      args: args.clone(),
      files,
    })
  }

  pub fn process_all(&self) -> Result<ProcessingStats, String> {
    fs::create_dir_all(&self.args.output)
      .map_err(|e| format!("Failed to create output directory: {}", e))?;

    if self.args.parallel && self.files.len() > 1 {
      self.process_parallel()
    } else {
      self.process_sequential()
    }
  }

  fn process_sequential(&self) -> Result<ProcessingStats, String> {
    let mut stats = ProcessingStats::default();

    for file_path in &self.files {
      match parse::process_single_file(file_path, &self.args) {
        Ok((doc_type, node_count)) => {
          stats.add_file(doc_type, node_count);
          self.log_success(file_path, node_count);
        }
        Err(e) => {
          stats.errors += 1;
          self.log_error(file_path, &e);
        }
      }
    }

    Ok(stats)
  }

  fn process_parallel(&self) -> Result<ProcessingStats, String> {
    use std::thread;

    let num_threads = thread::available_parallelism()
      .map(|n| n.get())
      .unwrap_or(4);
    let counters = ParallelCounters::new();
    let chunk_size = (self.files.len() + num_threads - 1) / num_threads;
    let mut handles = Vec::new();

    for chunk in self.files.chunks(chunk_size) {
      let chunk: Vec<PathBuf> = chunk.to_vec();
      let args = self.args.clone();
      let c = counters.clone();

      handles.push(thread::spawn(move || {
        for file_path in chunk {
          match parse::process_single_file(&file_path, &args) {
            Ok((doc_type, count)) => c.add_success(doc_type, count),
            Err(_) => c.add_error(),
          }
        }
      }));
    }

    for handle in handles {
      handle.join().map_err(|_| "Thread panicked")?;
    }

    Ok(counters.into_stats())
  }

  fn log_success(&self, path: &Path, node_count: usize) {
    if self.args.verbose {
      println!("  Processed: {} ({} nodes)", path.display(), node_count);
    }
  }

  fn log_error(&self, path: &Path, error: &str) {
    if self.args.verbose {
      eprintln!("  Error processing {}: {}", path.display(), error);
    }
  }
}

fn validate_input(args: &Args) -> Result<(), String> {
  if !args.input.exists() {
    return Err(format!(
      "Input directory does not exist: {}",
      args.input.display()
    ));
  }
  if !args.input.is_dir() {
    return Err(format!(
      "Input path is not a directory: {}",
      args.input.display()
    ));
  }
  Ok(())
}

fn validate_files(files: &[PathBuf], args: &Args) -> Result<(), String> {
  if files.is_empty() {
    return Err(format!(
      "No matching files found in {} with extensions: {:?}",
      args.input.display(),
      args.extensions
    ));
  }
  Ok(())
}

#[derive(Clone)]
struct ParallelCounters {
  markdown: std::sync::Arc<std::sync::atomic::AtomicUsize>,
  js: std::sync::Arc<std::sync::atomic::AtomicUsize>,
  java: std::sync::Arc<std::sync::atomic::AtomicUsize>,
  python: std::sync::Arc<std::sync::atomic::AtomicUsize>,
  nodes: std::sync::Arc<std::sync::atomic::AtomicUsize>,
  errors: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl ParallelCounters {
  fn new() -> Self {
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    Self {
      markdown: Arc::new(AtomicUsize::new(0)),
      js: Arc::new(AtomicUsize::new(0)),
      java: Arc::new(AtomicUsize::new(0)),
      python: Arc::new(AtomicUsize::new(0)),
      nodes: Arc::new(AtomicUsize::new(0)),
      errors: Arc::new(AtomicUsize::new(0)),
    }
  }

  fn add_success(&self, doc_type: crate::ast::DocumentType, node_count: usize) {
    use crate::ast::DocumentType;
    use std::sync::atomic::Ordering;

    match doc_type {
      DocumentType::Markdown => self.markdown.fetch_add(1, Ordering::Relaxed),
      DocumentType::JavaScript | DocumentType::TypeScript => {
        self.js.fetch_add(1, Ordering::Relaxed)
      }
      DocumentType::Java => self.java.fetch_add(1, Ordering::Relaxed),
      DocumentType::Python => self.python.fetch_add(1, Ordering::Relaxed),
    };
    self.nodes.fetch_add(node_count, Ordering::Relaxed);
  }

  fn add_error(&self) {
    use std::sync::atomic::Ordering;
    self.errors.fetch_add(1, Ordering::Relaxed);
  }

  fn into_stats(self) -> ProcessingStats {
    use std::sync::atomic::Ordering;
    ProcessingStats {
      markdown_files: self.markdown.load(Ordering::Relaxed),
      js_files: self.js.load(Ordering::Relaxed),
      java_files: self.java.load(Ordering::Relaxed),
      python_files: self.python.load(Ordering::Relaxed),
      total_nodes: self.nodes.load(Ordering::Relaxed),
      errors: self.errors.load(Ordering::Relaxed),
    }
  }
}
