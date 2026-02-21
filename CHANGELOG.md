# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - TBD

### Added

- Core library with `Board` (classic 9×9) and `Grid` (variable-size NxN) types
- Backtracking solver with shuffled candidates for variety
- Puzzle generator with unique-solution guarantee and difficulty levels (easy/medium/hard)
- Hint system for both `Board` and `Grid`
- Deterministic seeding via `set_rand_seed()` for reproducible generation
- String parsing (`FromStr` for `Board`, `from_string_n` for `Grid`)
- Display trait implementations for compact string output
- Serde serialization/deserialization support
- CLI binary (`sudoku-cli`) with clap — generate, solve, hint, variable sizes, JSON output
- REST server binary (`sudoku-server`) with axum — `/healthz`, `/generate`, `/solve` endpoints
- Security headers (Content-Type, Cache-Control, X-Content-Type-Options)
- Makefile with targets: `all`, `fmt`, `check`, `test`, `build`, `run`, `cli`, `clean`, `docker-*`
- Multi-stage Dockerfile with distroless runtime image
- Comprehensive test suite (28 tests across lib, server, and examples)
- CI workflow (fmt, clippy, test with coverage, security audit, Docker build + smoke test)
- Publish workflow for crates.io release on GitHub release creation
- Community files: CODE_OF_CONDUCT, CONTRIBUTING, SECURITY, FUNDING, CHANGELOG

[Unreleased]: https://github.com/rumendamyanov/rust-sudoku/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/rumendamyanov/rust-sudoku/releases/tag/v1.0.0
