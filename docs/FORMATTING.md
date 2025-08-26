# Code Formatting Guide

This project uses automatic code formatting to ensure consistent code style across all contributors.

## Automatic Formatting Setup

### Git Hooks (Already Configured)

The project includes git hooks that automatically check code formatting:

- **Pre-commit hook**: Prevents commits with incorrectly formatted code
- **Pre-push hook**: Double-checks formatting before pushing to remote

### Development Scripts

Use the provided development scripts for common formatting tasks:

```bash
# Format all code automatically
./scripts/dev.sh fmt

# Check if code needs formatting (without making changes)
./scripts/dev.sh fmt-check

# Run all pre-commit checks (formatting + tests + linting)
./scripts/dev.sh pre-commit
```

### Available Commands

| Command | Description |
|---------|-------------|
| `./scripts/dev.sh fmt` | Auto-format all code with cargo fmt |
| `./scripts/dev.sh fmt-check` | Check if formatting is needed |
| `./scripts/dev.sh test` | Run all tests |
| `./scripts/dev.sh check` | Run cargo check for compilation |
| `./scripts/dev.sh lint` | Run clippy linting |
| `./scripts/dev.sh pre-commit` | Run all quality checks |

## Manual Formatting

You can also run formatting commands directly:

```bash
# Format all code
cargo fmt

# Check formatting without making changes
cargo fmt --check

# Format specific file
cargo fmt src/main.rs
```

## Configuration

The project uses a `rustfmt.toml` configuration file with these settings:

- **Line width**: 100 characters maximum
- **Indentation**: 4 spaces (no tabs)
- **Function parameters**: Vertical layout for readability
- **Match blocks**: Trailing commas for better diffs
- **Error handling**: Use `?` operator shorthand

## CI/CD Integration

GitHub Actions automatically checks formatting on:

- All pushes to `main` and `develop` branches
- All pull requests
- Multiple Rust versions (stable, beta, nightly)

## Tips

- **Before committing**: Run `./scripts/dev.sh pre-commit` to catch issues early
- **IDE integration**: Configure your editor to run `cargo fmt` on save
- **Bypassing hooks**: Use `git commit --no-verify` only in emergencies (not recommended)

## Troubleshooting

If formatting checks fail:

1. Run `./scripts/dev.sh fmt` to auto-fix formatting
2. Stage and commit the changes: `git add -A && git commit -m "fix: Apply code formatting"`
3. Push the changes: `git push`

The git hooks will prevent commits/pushes with formatting issues, ensuring code quality consistency.
