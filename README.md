# rust-sudoku

[![CI](https://github.com/rumendamyanov/rust-sudoku/actions/workflows/ci.yml/badge.svg)](https://github.com/rumendamyanov/rust-sudoku/actions/workflows/ci.yml)
[![CodeQL](https://github.com/RumenDamyanov/rust-sudoku/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/RumenDamyanov/rust-sudoku/actions/workflows/github-code-scanning/codeql)
[![Dependabot](https://github.com/RumenDamyanov/rust-sudoku/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/RumenDamyanov/rust-sudoku/actions/workflows/dependabot/dependabot-updates)
[![codecov](https://codecov.io/gh/RumenDamyanov/rust-sudoku/graph/badge.svg)](https://codecov.io/gh/RumenDamyanov/rust-sudoku)
[![crates.io](https://img.shields.io/crates/v/rumenx-sudoku.svg)](https://crates.io/crates/rumenx-sudoku)
[![docs.rs](https://docs.rs/rumenx-sudoku/badge.svg)](https://docs.rs/rumenx-sudoku)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/rumendamyanov/rust-sudoku/blob/master/LICENSE.md)

Fast, dependency-light Sudoku generator & solver for Rust — classic 9×9 plus configurable smaller grids — with guaranteed unique puzzles, deterministic seeding, CLI, and REST server.

> Rust adaptation of [go-sudoku](https://github.com/RumenDamyanov/go-sudoku).

## Highlights

- ✅ Minimal core dependencies (`rand`, `thiserror`, `serde`)
- 🎯 Deterministic, validated backtracking solver
- 🧩 Generator guarantees a unique solution & target clue counts (difficulty aware)
- 🧪 Comprehensive test suite
- 🌐 Minimal REST server (generate / solve) via `axum`
- 💻 CLI (generate / solve / hint / variable sizes) via `clap`
- 🧱 General `Grid` API for 4×4, 6×6, 9×9 (and other ≤9 sizes with box layout)
- 🔍 Hint helpers for both classic `Board` and general `Grid`

## Install

### As a library

```toml
[dependencies]
rumenx-sudoku = "1"
```

Or from git:

```toml
[dependencies]
rumenx-sudoku = { git = "https://github.com/RumenDamyanov/rust-sudoku" }
```

### As a CLI

```sh
cargo install rumenx-sudoku --features cli --bin sudoku-cli
```

## Library Usage

```rust
use rumenx_sudoku::{Board, Difficulty, Grid, set_rand_seed};

fn main() {
    // Classic 9×9
    set_rand_seed(42);
    let puz = Board::generate(Difficulty::Medium, 1).unwrap();
    println!("Puzzle: {puz}");
    if let Some(sol) = puz.solve() {
        println!("Solved: {sol}");
    }

    // Variable size (6×6 with 2×3 boxes)
    let g = Grid::new(6, 2, 3).unwrap();
    let gpuz = g.generate(Difficulty::Easy, 2).unwrap();
    if let Some((r, c, v)) = gpuz.hint() {
        println!("Hint: row {}, col {} = {}", r + 1, c + 1, v);
    }
}
```

## Difficulty Levels

| Level  | 9×9 Clues | Description          |
|--------|-----------|----------------------|
| Easy   | ~40       | Lots of clues        |
| Medium | ~32       | Balanced             |
| Hard   | ~26       | Fewer clues          |

Clue counts are proportionally scaled for other grid sizes.

## Variable-Size Grids

Supported configurations (any `size = box_rows × box_cols`):

| Grid | Box  | Example                  |
|------|------|--------------------------|
| 4×4  | 2×2  | Kids / quick puzzles     |
| 6×6  | 2×3  | Intermediate             |
| 9×9  | 3×3  | Classic Sudoku           |

## CLI

```sh
# Generate a medium 9×9 puzzle
cargo run --features cli --bin sudoku-cli

# Generate easy 6×6 as JSON
cargo run --features cli --bin sudoku-cli -- --difficulty easy --size 6 --box 2x3 --json

# Solve a puzzle string
cargo run --features cli --bin sudoku-cli -- --string '530070000600195000098000060800060003400803001700020006060000280000419005000080079'

# Get a hint
cargo run --features cli --bin sudoku-cli -- --string '530070000600195000098000060800060003400803001700020006060000280000419005000080079' --hint

# Print version
cargo run --features cli --bin sudoku-cli -- --version
```

### CLI Flags

| Flag           | Default  | Description                          |
|----------------|----------|--------------------------------------|
| `--difficulty` | medium   | easy \| medium \| hard               |
| `--attempts`   | 3        | Generation retries                   |
| `--solve`      | false    | Also show solution when generating   |
| `--size`       | 9        | Grid size (S×S)                      |
| `--box`        | 3x3      | Sub-box dimensions R×C               |
| `--hint`       | false    | Show hint for puzzle                 |
| `--string`     | —        | Puzzle string to solve               |
| `--file`       | —        | File containing puzzle string        |
| `--json`       | false    | Output as JSON                       |
| `--version`    | false    | Print version and exit               |

## REST Server

```sh
cargo run --features server --bin sudoku-server
```

### Endpoints

| Method | Path       | Description                |
|--------|-----------|----------------------------|
| GET    | /healthz  | Health check + version     |
| GET    | /health   | Alias for /healthz         |
| POST   | /generate | Generate a puzzle          |
| POST   | /solve    | Solve a puzzle             |

### Examples

```sh
# Health check
curl -s localhost:8080/healthz | jq .

# Generate medium puzzle
curl -s -X POST localhost:8080/generate \
  -H 'Content-Type: application/json' \
  -d '{"difficulty":"medium"}' | jq .

# Generate with solution
curl -s -X POST localhost:8080/generate \
  -H 'Content-Type: application/json' \
  -d '{"difficulty":"easy","includeSolution":true}' | jq .

# Solve
curl -s -X POST localhost:8080/solve \
  -H 'Content-Type: application/json' \
  -d '{"string":"530070000600195000098000060800060003400803001700020006060000280000419005000080079"}' | jq .
```

## Docker

```sh
# Build
docker build -t rust-sudoku .

# Run
docker run --rm -p 8080:8080 rust-sudoku
```

## Build

```sh
make all        # fmt + check + test
make build      # release binaries in bin/
make run        # run server
make cli        # run CLI
make clean      # clean up
```

## License

MIT — see [LICENSE.md](LICENSE.md)
