# CrateDocs MCP Development Guide

## Build Commands

- Build project: `cargo build`
- Run STDIN/STDOUT server: `cargo run --bin stdio-server`
- Run HTTP/SSE server: `cargo run --bin axum-docs`
- Run tests: `cargo test`
- Run specific test: `cargo test test_name`
- Format code: `cargo fmt`
- Check code: `cargo clippy`

## Code Style Guidelines

- **Naming**: Use snake_case for functions/variables, CamelCase for types/structs
- **Error Handling**: Use `Result<T, ToolError>` for functions that can fail
- **Imports**: Organize imports by external crates first, then internal modules
- **Async**: Use async/await with proper error propagation using `?` operator
- **Documentation**: Include doc comments for public functions and structs
- **Error Messages**: Be specific and descriptive in error messages
- **Caching**: Implement caching for repeated operations when appropriate
- **Responses**: Return structured responses via `Content::text()`

## Project Architecture

- Implements MCP protocol for Rust documentation tools
- `DocRouter` is the core component for handling tool requests
- Uses reqwest for API requests, tokio for async runtime
- Async handlers for all tool implementations

## how to analyze code

- the main dependency source ./rust-sdk
- use zed function
- use Brave web api if needed
