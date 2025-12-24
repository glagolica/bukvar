# Contributing to Bukvar

Thank you for your interest in contributing to Bukvar! This document provides guidelines and information for contributors.

## 🚀 Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/bukvar.git
   cd bukvar
   ```
3. **Install dependencies**:

   - [Rust](https://rustup.rs/) (stable channel)
   - [just](https://github.com/casey/just) (optional, but recommended)

4. **Run the tests** to make sure everything works:
   ```bash
   just test
   # or
   cargo test
   ```

## 📝 Development Workflow

### Making Changes

1. Create a new branch for your feature/fix:

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes, following our coding standards

3. Run the full check suite:

   ```bash
   just check
   # or manually:
   cargo fmt -- --check
   cargo clippy --all-targets
   cargo test
   ```

4. Commit your changes with a clear message:

   ```bash
   git commit -m "feat: add support for X"
   ```

5. Push and create a Pull Request

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `perf:` - Performance improvements
- `chore:` - Maintenance tasks

Examples:

```
feat: add table alignment support
fix: handle empty code blocks correctly
docs: update CLI reference in README
test: add tests for frontmatter parsing
```

## 🎨 Code Style

### Rust Guidelines

- Use `rustfmt` for formatting (config in `rustfmt.toml`)
- Follow `clippy` recommendations
- Keep functions small and focused
- Prefer iterators over manual loops
- Use `#[inline]` sparingly and only where benchmarked

### File Organization

- Keep files under 200 lines when possible
- Split large modules into submodules
- Minimize nesting depth (max 2-3 levels)
- Group related functionality together

### Documentation

- Document all public APIs
- Include examples in doc comments
- Keep comments concise and useful

````rust
/// Parses a markdown document into an AST.
///
/// # Example
/// ```
/// let mut parser = MarkdownParser::new("# Hello");
/// let doc = parser.parse();
/// ```
pub fn parse(&mut self) -> Document {
    // ...
}
````

## 🧪 Testing

### Writing Tests

- Add tests for all new functionality
- Place tests in the same file using `#[cfg(test)]`
- Use descriptive test names: `test_parse_heading_with_id`
- Test edge cases and error conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_basic() {
        // Test basic functionality
    }

    #[test]
    fn test_feature_edge_case() {
        // Test edge cases
    }
}
```

### Running Tests

```bash
# All tests
just test

# Specific test
just test-one test_name

# With output
just test-verbose
```

## 🏗️ Architecture

### Key Principles

1. **Zero dependencies** - Don't add external crates
2. **Performance first** - Optimize hot paths
3. **Memory efficient** - Minimize allocations
4. **Clean abstractions** - Clear module boundaries

### Module Overview

```
src/
├── ast/        # Data structures (Node, Span, etc.)
├── markdown/   # Markdown parsing (scanner → blocks → inlines)
├── parsers/    # Doc comment parsers (JSDoc, JavaDoc, PyDoc)
├── formats/    # Output (JSON, DAST binary)
├── processor/  # File handling and parallelism
└── *.rs        # Top-level modules (cli, validate, etc.)
```

## 🐛 Reporting Issues

When reporting bugs, please include:

1. **Rust version** (`rustc --version`)
2. **OS and version**
3. **Steps to reproduce**
4. **Expected vs actual behavior**
5. **Minimal test case** if possible

## 💡 Feature Requests

For feature requests:

1. Check existing issues first
2. Describe the use case clearly
3. Explain why existing solutions don't work
4. Consider submitting a PR!

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Bukvar! 🎉
