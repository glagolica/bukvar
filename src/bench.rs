//! Simple benchmarking utilities.
//!
//! Provides lightweight timing measurements without external dependencies.

use std::time::{Duration, Instant};

/// Result of a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchResult {
  /// Name of the benchmark
  pub name: String,
  /// Total time for all iterations
  pub total_time: Duration,
  /// Number of iterations
  pub iterations: usize,
  /// Average time per iteration
  pub avg_time: Duration,
  /// Throughput in operations/second
  pub ops_per_sec: f64,
}

impl BenchResult {
  /// Format result as a summary string.
  pub fn summary(&self) -> String {
    let avg_us = self.avg_time.as_secs_f64() * 1_000_000.0;
    let total_ms = self.total_time.as_secs_f64() * 1_000.0;
    format!(
      "{}: {:.2} µs/op ({:.0} ops/sec, {} iters, {:.2}ms total)",
      self.name, avg_us, self.ops_per_sec, self.iterations, total_ms
    )
  }
}

/// Run a benchmark with the given function.
///
/// Runs the function multiple times and measures timing.
pub fn bench<F>(name: &str, iterations: usize, mut f: F) -> BenchResult
where
  F: FnMut(),
{
  // Warm up
  for _ in 0..5 {
    f();
  }

  // Actual measurement
  let start = Instant::now();
  for _ in 0..iterations {
    f();
  }
  let total_time = start.elapsed();

  let avg_time = total_time / iterations as u32;
  let ops_per_sec = iterations as f64 / total_time.as_secs_f64();

  BenchResult {
    name: name.to_string(),
    total_time,
    iterations,
    avg_time,
    ops_per_sec,
  }
}

/// Run a benchmark measuring throughput (bytes/second).
pub fn bench_throughput<F>(name: &str, iterations: usize, bytes_per_iter: usize, mut f: F) -> String
where
  F: FnMut(),
{
  let result = bench(name, iterations, &mut f);

  let total_bytes = iterations * bytes_per_iter;
  let bytes_per_sec = total_bytes as f64 / result.total_time.as_secs_f64();
  let mb_per_sec = bytes_per_sec / (1024.0 * 1024.0);

  format!(
    "{}: {:.2} MB/s ({} iterations, {} bytes each)",
    name, mb_per_sec, iterations, bytes_per_iter
  )
}

/// Benchmark suite runner.
pub struct BenchSuite {
  results: Vec<BenchResult>,
}

impl BenchSuite {
  pub fn new() -> Self {
    Self {
      results: Vec::new(),
    }
  }

  /// Add a benchmark to the suite.
  pub fn add<F>(&mut self, name: &str, iterations: usize, f: F)
  where
    F: FnMut(),
  {
    let result = bench(name, iterations, f);
    self.results.push(result);
  }

  /// Print all results.
  pub fn report(&self) {
    println!("\n=== Benchmark Results ===\n");
    for result in &self.results {
      println!("{}", result.summary());
    }
    println!();
  }
}

impl Default for BenchSuite {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bench_runs() {
    let mut count = 0;
    let result = bench("test_counter", 100, || {
      count += 1;
    });
    assert_eq!(result.iterations, 100);
    assert!(count >= 100); // includes warmup
  }

  #[test]
  fn test_bench_suite() {
    let mut suite = BenchSuite::new();
    suite.add("fast_op", 1000, || {
      let _ = 1 + 1;
    });
    assert_eq!(suite.results.len(), 1);
  }
}
