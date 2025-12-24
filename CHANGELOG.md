# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2025-12-24

### Added

- Initial release of Bukvar
- Full GFM (GitHub Flavored Markdown) parsing support
- JSDoc parser for JavaScript/TypeScript files
- JavaDoc parser for Java files
- PyDoc parser (Google, NumPy, Sphinx styles) for Python files
- JSON output format with pretty-print option
- DAST (compact binary) output format with string interning
- Parallel file processing for improved performance
- Streaming parser for memory-efficient large file handling
- Source map generation for AST-to-source mapping
- Document validation (broken links, missing references)
- Frontmatter support (YAML/TOML)
- Math block and inline math support
- Footnote support
- Definition list support
- Task list (checkbox) support
- Comprehensive CLI with all configuration options
- 138 unit tests covering all major functionality
- Zero external dependencies

### Performance

- LTO (Link Time Optimization) enabled for release builds
- Single codegen unit for maximum optimization
- Panic=abort for smaller binary size
- String interning in DAST format for ~3-5x size reduction vs JSON

---

## Version History

- **1.0.0** - Initial release with full markdown and doc-comment parsing
