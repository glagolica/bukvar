mod ast;
mod bench;
mod cli;
mod error;
mod formats;
mod markdown;
mod parsers;
mod processor;
mod sourcemap;
mod streaming;
mod validate;

use cli::parse_args;
use processor::FileProcessor;
use std::time::Instant;

fn main() {
  let args = match parse_args() {
    Ok(args) => args,
    Err(msg) => {
      // Help or error message
      if msg.starts_with("bukvar") || msg.starts_with("Bukvar") {
        println!("{}", msg);
        std::process::exit(0);
      } else {
        eprintln!("{}", msg);
        std::process::exit(1);
      }
    }
  };

  // Run benchmarks if requested
  if args.bench {
    run_benchmarks();
    return;
  }

  println!();
  println!("\x1b[1;36mBukvar v1.0.0\x1b[0m  \x1b[90m(Glagolica Project)\x1b[0m");
  println!("\x1b[90mUltra-fast zero-dependency markdown parser\x1b[0m");
  println!();
  println!(
    "  Input:  {}",
    args.input.to_string_lossy().replace('\\', "/")
  );
  println!(
    "  Output: {}",
    args.output.to_string_lossy().replace('\\', "/")
  );
  println!("  Format: {:?}", args.format);
  println!();

  let start = Instant::now();

  let processor = match FileProcessor::new(&args) {
    Ok(p) => p,
    Err(e) => {
      eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
      std::process::exit(1);
    }
  };

  let stats = match processor.process_all() {
    Ok(s) => s,
    Err(e) => {
      eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
      std::process::exit(1);
    }
  };

  let elapsed = start.elapsed();
  let total = stats.total_files();

  // Success output
  println!();
  println!("\x1b[32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
  println!("\x1b[1;32m  ✓ SUCCESS\x1b[0m");
  println!("\x1b[32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
  println!();
  println!("\x1b[1m  Files Processed\x1b[0m");
  println!(
    "    Markdown     \x1b[36m{:>5}\x1b[0m",
    stats.markdown_files
  );
  println!("    JavaScript   \x1b[36m{:>5}\x1b[0m", stats.js_files);
  println!("    Java         \x1b[36m{:>5}\x1b[0m", stats.java_files);
  println!("    Python       \x1b[36m{:>5}\x1b[0m", stats.python_files);
  println!();
  println!("\x1b[1m  AST Generated\x1b[0m");
  println!("    Total nodes  \x1b[33m{:>5}\x1b[0m", stats.total_nodes);

  if stats.errors > 0 {
    println!("    Errors       \x1b[31m{:>5}\x1b[0m", stats.errors);
  }

  println!();
  println!("\x1b[1m  Performance\x1b[0m");
  println!("    Time         \x1b[32m{:.2?}\x1b[0m", elapsed);

  if elapsed.as_secs_f64() > 0.0 {
    let throughput = total as f64 / elapsed.as_secs_f64();
    println!(
      "    Throughput   \x1b[32m{:.0} files/sec\x1b[0m",
      throughput
    );
  }

  println!("\x1b[32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
  println!();
}

/// Run internal benchmarks.
fn run_benchmarks() {
  use bench::{bench_throughput, BenchSuite};
  use markdown::MarkdownParser;

  println!("\n\x1b[1;36mBukvar Benchmarks\x1b[0m  \x1b[90m(Glagolica Project)\x1b[0m\n");

  let mut suite = BenchSuite::new();

  // Simple paragraph
  let simple = "Hello, this is a simple paragraph.";
  suite.add("simple_paragraph", 10000, || {
    let mut p = MarkdownParser::new(simple);
    let _ = p.parse();
  });

  // Headings
  let headings = "# Heading 1\n\nSome text.\n\n## Heading 2\n\nMore text.";
  suite.add("headings", 10000, || {
    let mut p = MarkdownParser::new(headings);
    let _ = p.parse();
  });

  // Emphasis and strong
  let emphasis = "This is *emphasized* and **strong** text with `code`.";
  suite.add("inline_emphasis", 10000, || {
    let mut p = MarkdownParser::new(emphasis);
    let _ = p.parse();
  });

  // Links
  let links = "Check [this link](https://example.com) and ![image](img.png).";
  suite.add("links", 10000, || {
    let mut p = MarkdownParser::new(links);
    let _ = p.parse();
  });

  // Code block
  let code = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
  suite.add("code_block", 10000, || {
    let mut p = MarkdownParser::new(code);
    let _ = p.parse();
  });

  // Lists
  let list = "- Item 1\n- Item 2\n  - Nested\n- Item 3";
  suite.add("list", 10000, || {
    let mut p = MarkdownParser::new(list);
    let _ = p.parse();
  });

  // Complex document
  let complex = r#"# Title

Introduction paragraph with *emphasis* and **strong**.

## Features

- Feature 1
- Feature 2
- Feature 3

```rust
fn example() {
    println!("code");
}
```

Check [link](https://example.com) for more info.
"#;
  suite.add("complex_doc", 5000, || {
    let mut p = MarkdownParser::new(complex);
    let _ = p.parse();
  });

  suite.report();

  // Throughput benchmarks - show MB/s parsing speed
  println!("=== Throughput Benchmarks ===\n");

  // Large document throughput test
  let large_doc = complex.repeat(100); // ~23KB document
  let throughput = bench_throughput("large_doc_throughput", 1000, large_doc.len(), || {
    let mut p = MarkdownParser::new(&large_doc);
    let _ = p.parse();
  });
  println!("{}", throughput);

  // Simple text throughput
  let bulk_simple = simple.repeat(1000); // ~34KB of simple text
  let simple_throughput =
    bench_throughput("bulk_simple_throughput", 500, bulk_simple.len(), || {
      let mut p = MarkdownParser::new(&bulk_simple);
      let _ = p.parse();
    });
  println!("{}", simple_throughput);

  println!();
}
