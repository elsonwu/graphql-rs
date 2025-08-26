# Contributing to GraphQL-RS

Thank you for your interest in contributing to GraphQL-RS! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Architecture Guidelines](#architecture-guidelines)
- [Testing Guidelines](#testing-guidelines)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Please be respectful and considerate in all interactions.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your feature or bugfix
4. Make your changes
5. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Local Setup

```bash
# Clone the repository
git clone https://github.com/elsonwu/graphql-rs.git
cd graphql-rs

# Install dependencies
cargo build

# Run tests
cargo test

# Run linting
cargo clippy

# Format code
cargo fmt
```

## Making Changes

### Branch Naming

Use descriptive branch names with prefixes:

- `feat/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/modifications
- `chore/` - Maintenance tasks

Examples:
- `feat/query-execution-engine`
- `fix/schema-validation-error`
- `docs/api-documentation`

### Domain-Driven Design

This project follows Domain-Driven Design principles:

- **Domain Layer**: Core business logic and domain models
- **Application Layer**: Use cases and application services
- **Infrastructure Layer**: External dependencies and technical implementation
- **Presentation Layer**: API endpoints and request/response handling

Place new code in the appropriate layer and follow the existing patterns.

## Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools

### Examples

```
feat(parser): add support for custom scalar types

fix(validation): resolve schema validation edge case

docs(readme): update installation instructions

refactor(domain): extract common traits into separate module
```

## Pull Request Process

1. **Update Documentation**: Ensure README, CHANGELOG, and code documentation are updated
2. **Add Tests**: Include unit tests and integration tests for new functionality
3. **Follow Code Style**: Run `cargo fmt` and `cargo clippy`
4. **Update CHANGELOG**: Add your changes to the Unreleased section
5. **Create PR**: Use a descriptive title and detailed description
6. **Review Process**: Address feedback and maintain CI passing status

### PR Template

When creating a PR, include:

- **Description**: What changes are being made and why
- **Type of Change**: Bug fix, new feature, breaking change, etc.
- **Testing**: How the changes have been tested
- **Checklist**: Ensure all requirements are met

## Architecture Guidelines

### Code Organization

```
src/
├── domain/           # Domain layer (business logic)
│   ├── entities/     # Domain entities
│   ├── value_objects/# Value objects
│   ├── services/     # Domain services
│   └── repositories/ # Repository traits
├── application/      # Application layer (use cases)
│   ├── services/     # Application services
│   ├── use_cases/    # Use case implementations
│   └── dto/          # Data transfer objects
├── infrastructure/   # Infrastructure layer
│   ├── persistence/  # Database implementations
│   ├── web/          # Web server setup
│   └── config/       # Configuration
└── presentation/     # Presentation layer
    ├── handlers/     # Request handlers
    ├── middleware/   # HTTP middleware
    └── schema/       # GraphQL schema definitions
```

### Design Principles

- **Single Responsibility**: Each module should have one reason to change
- **Dependency Inversion**: Depend on abstractions, not concretions
- **Open/Closed**: Open for extension, closed for modification
- **Interface Segregation**: Many specific interfaces are better than one general-purpose interface

## Testing Guidelines

### Test Structure

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test component interactions
- **End-to-End Tests**: Test complete user scenarios

### Test Naming

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_query_when_valid_syntax() {
        // Given
        let query = "{ user { name } }";
        
        // When
        let result = parse_query(query);
        
        // Then
        assert!(result.is_ok());
    }
}
```

### Test Coverage

- Aim for high test coverage, especially for critical business logic
- Use `cargo tarpaulin` to measure coverage
- Include both positive and negative test cases

## Questions or Help?

If you have questions or need help:

1. Check existing issues and discussions
2. Create a new issue with the `question` label
3. Reach out to maintainers

Thank you for contributing to GraphQL-RS!
