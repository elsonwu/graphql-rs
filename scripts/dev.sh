#!/bin/bash
#
# Development scripts for graphql-rs project
# Usage: ./scripts/dev.sh <command>

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

case "${1:-help}" in
    "fmt" | "format")
        echo "🔧 Running cargo fmt..."
        cargo fmt --all
        echo "✅ Formatting complete!"
        ;;
    
    "fmt-check" | "format-check")
        echo "🔍 Checking formatting..."
        if cargo fmt --all --check; then
            echo "✅ All files are properly formatted!"
        else
            echo "❌ Some files need formatting. Run './scripts/dev.sh fmt' to fix."
            exit 1
        fi
        ;;
    
    "test")
        echo "🧪 Running tests..."
        cargo test
        ;;
    
    "check")
        echo "🔍 Running cargo check..."
        cargo check --all-targets
        ;;
    
    "lint")
        echo "🔍 Running clippy..."
        cargo clippy --all-targets -- -D warnings
        ;;
    
    "pre-commit")
        echo "🚀 Running pre-commit checks..."
        ./scripts/dev.sh fmt-check
        ./scripts/dev.sh check
        ./scripts/dev.sh test
        echo "✅ All pre-commit checks passed!"
        ;;
    
    "help" | *)
        echo "GraphQL-RS Development Scripts"
        echo ""
        echo "Usage: ./scripts/dev.sh <command>"
        echo ""
        echo "Available commands:"
        echo "  fmt           - Format all code with cargo fmt"
        echo "  fmt-check     - Check if code is properly formatted"
        echo "  test          - Run all tests"
        echo "  check         - Run cargo check"
        echo "  lint          - Run clippy linter"
        echo "  pre-commit    - Run all pre-commit checks"
        echo "  help          - Show this help message"
        echo ""
        ;;
esac
