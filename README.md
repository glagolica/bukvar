<div align="center">

# Bukvar

**Ultra-fast zero-dependency markdown and documentation parser**

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![License: LGPL v3](https://img.shields.io/badge/License-LGPL_v3-blue.svg)](LICENSE.md)
[![Zero Dependencies](https://img.shields.io/badge/dependencies-0-brightgreen.svg)](#)

</div>

---

## Features

- **Blazingly fast** - Zero-copy parsing, minimal allocations
- **Zero dependencies** - Everything implemented from scratch
- **GFM Markdown** - Full GitHub Flavored Markdown support
- **Doc comments** - JSDoc, JavaDoc, and PyDoc extraction
- **Multiple outputs** - JSON and compact binary (DAST) formats
- **Validation** - Check for broken links and references
- **Source maps** - Track AST nodes back to source positions
- **Streaming** - Memory-efficient parsing for large files

## Installation

```bash
git clone https://github.com/glagolica/bukvar.git
cd bukvar
cargo install --path .
```

## Quick Start

```bash
# Parse markdown files to JSON
bukvar ./docs ./output -f json --pretty

# Parse with validation
bukvar ./src ./ast -f json --validate --verbose

# Use compact binary format
bukvar ./docs ./output -f dast
```

## CLI Reference

```
bukvar [OPTIONS] <INPUT> [OUTPUT]

OPTIONS:
    -i, --input <PATH>      Input directory
    -o, --output <PATH>     Output directory (default: ./ast_output)
    -f, --format <FMT>      Output format: dast (binary) or json
    -e, --extensions <EXT>  File extensions (comma-separated)
    --pretty                Pretty-print JSON output
    --validate              Check for broken links/references
    --sourcemap             Generate source maps
    --streaming             Streaming parser for large files
    --verbose               Show detailed progress
    -h, --help              Show help
    -v, --version           Show version
```

## Supported Files

| Extension      | Parser       | Description                  |
| -------------- | ------------ | ---------------------------- |
| .md, .markdown | GFM Markdown | GitHub Flavored Markdown     |
| .js, .ts, .tsx | JSDoc        | JavaScript/TypeScript        |
| .java          | JavaDoc      | Java documentation           |
| .py, .pyi      | PyDoc        | Google, NumPy, Sphinx styles |

## Output Formats

### JSON

Human-readable AST. Easy to inspect and process.

### DAST (Binary)

Compact binary format with string interning. ~3-5x smaller than JSON.

## Development

```bash
just test     # Run tests
just check    # Lint and format
just release  # Build release
just ci       # Full CI pipeline
```

## License

GNU Lesser General Public License v3.0 - see [LICENSE.md](LICENSE.md)
