//! # sudoku
//!
//! Fast Sudoku generator & solver for Rust — classic 9×9 plus configurable grids —
//! with guaranteed unique puzzles, deterministic seeding, and hint helpers.
//!
//! Module: rust-sudoku
//! Author: Rumen Damyanov <contact@rumenx.com>

mod board;
mod grid;
mod parse;

pub use board::{Board, Difficulty, set_rand_seed};
pub use grid::Grid;
pub use parse::MAX_GRID_SIZE;

#[cfg(feature = "server")]
pub mod server;
