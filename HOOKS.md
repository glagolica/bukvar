# Git Hooks Setup

This project uses Git hooks for code quality.

## Installation

### Unix/macOS

```bash
chmod +x hooks/pre-commit
ln -sf ../../hooks/pre-commit .git/hooks/pre-commit
```

### Windows (PowerShell)

```powershell
Copy-Item -Path hooks\pre-commit -Destination .git\hooks\pre-commit
```

Or configure Git to use the hooks directory:

```bash
git config core.hooksPath hooks
```

## Hooks

### pre-commit

- Runs `cargo fmt --check` to verify formatting
- Runs `cargo clippy` to catch common issues

## Manual Checks

Run these commands before committing:

```bash
cargo fmt        # Format code
cargo clippy     # Lint code
cargo test       # Run tests
```
