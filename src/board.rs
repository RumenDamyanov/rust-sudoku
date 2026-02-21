//! Classic 9×9 Sudoku board: validation, solving, generation, and hints.

use std::fmt;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors returned by board operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SudokuError {
    #[error("invalid board")]
    InvalidBoard,
    #[error("generation failed: {0}")]
    GenerationFailed(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

// ---------------------------------------------------------------------------
// Difficulty
// ---------------------------------------------------------------------------

/// Difficulty controls the number of clues in a generated puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Target number of clues for 9×9.
    pub fn clues_for_9x9(self) -> usize {
        match self {
            Difficulty::Easy => 40,
            Difficulty::Medium => 32,
            Difficulty::Hard => 26,
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "easy"),
            Difficulty::Medium => write!(f, "medium"),
            Difficulty::Hard => write!(f, "hard"),
        }
    }
}

impl std::str::FromStr for Difficulty {
    type Err = SudokuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            _ => Err(SudokuError::InvalidInput(format!(
                "invalid difficulty: {s}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// Global RNG
// ---------------------------------------------------------------------------

std::thread_local! {
    static GLOBAL_RNG: std::cell::RefCell<StdRng> = std::cell::RefCell::new(
        StdRng::from_os_rng()
    );
}

/// Sets the seed for the library's random generator ensuring reproducible generation.
/// Call during init / setup; not concurrency-guarded beyond the thread-local.
pub fn set_rand_seed(seed: u64) {
    GLOBAL_RNG.with(|rng| {
        *rng.borrow_mut() = StdRng::seed_from_u64(seed);
    });
}

pub(crate) fn with_rng<F, R>(f: F) -> R
where
    F: FnOnce(&mut StdRng) -> R,
{
    GLOBAL_RNG.with(|rng| f(&mut rng.borrow_mut()))
}

// ---------------------------------------------------------------------------
// Board
// ---------------------------------------------------------------------------

/// A 9×9 Sudoku grid. Empty cells are 0.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board(pub [[u8; 9]; 9]);

impl Board {
    /// Creates an empty board.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates that values are in `[0,9]` and no row/col/box duplicates (ignoring zeros).
    pub fn validate(&self) -> Result<(), SudokuError> {
        for i in 0..9 {
            let mut row = [false; 10];
            let mut col = [false; 10];
            for j in 0..9 {
                let rv = self.0[i][j] as usize;
                let cv = self.0[j][i] as usize;
                if rv > 9 || cv > 9 {
                    return Err(SudokuError::InvalidBoard);
                }
                if rv != 0 {
                    if row[rv] {
                        return Err(SudokuError::InvalidBoard);
                    }
                    row[rv] = true;
                }
                if cv != 0 {
                    if col[cv] {
                        return Err(SudokuError::InvalidBoard);
                    }
                    col[cv] = true;
                }
            }
        }
        // 3×3 boxes
        for br in (0..9).step_by(3) {
            for bc in (0..9).step_by(3) {
                let mut seen = [false; 10];
                for r in br..br + 3 {
                    for c in bc..bc + 3 {
                        let v = self.0[r][c] as usize;
                        if v != 0 {
                            if seen[v] {
                                return Err(SudokuError::InvalidBoard);
                            }
                            seen[v] = true;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Solves the board using backtracking. Returns `Some(solved)` or `None`.
    pub fn solve(&self) -> Option<Board> {
        let mut work = *self;
        if backtrack_board(&mut work) {
            Some(work)
        } else {
            None
        }
    }

    /// Generates a puzzle with a unique solution.
    /// `attempts` controls how many removal passes to try (>= 1).
    pub fn generate(difficulty: Difficulty, attempts: usize) -> Result<Board, SudokuError> {
        let attempts = attempts.max(1);
        let mut last_err = None;

        for _ in 0..attempts {
            let mut b = Board::new();
            fill_diagonal_boxes_board(&mut b);
            if !backtrack_board(&mut b) {
                last_err = Some("failed to build solved board".to_string());
                continue;
            }
            let target = difficulty.clues_for_9x9();
            let mut puzzle = b;
            let mut rm_order: Vec<usize> = (0..81).collect();
            with_rng(|rng| rm_order.shuffle(rng));

            for idx in &rm_order {
                if count_clues_board(&puzzle) <= target {
                    break;
                }
                let r = idx / 9;
                let c = idx % 9;
                let old = puzzle.0[r][c];
                if old == 0 {
                    continue;
                }
                puzzle.0[r][c] = 0;
                if !has_unique_solution_board(&puzzle, 2) {
                    puzzle.0[r][c] = old;
                }
            }
            if has_unique_solution_board(&puzzle, 2) {
                return Ok(puzzle);
            }
            last_err = Some("puzzle uniqueness not achieved".to_string());
        }

        Err(SudokuError::GenerationFailed(
            last_err.unwrap_or_else(|| "generation failed".to_string()),
        ))
    }

    /// Returns a single suggested value for the board: `(row, col, value)`.
    pub fn hint(&self) -> Option<(usize, usize, u8)> {
        if self.validate().is_err() {
            return None;
        }
        if let Some(sol) = self.solve() {
            for r in 0..9 {
                for c in 0..9 {
                    if self.0[r][c] == 0 {
                        return Some((r, c, sol.0[r][c]));
                    }
                }
            }
        }
        None
    }
}

impl fmt::Display for Board {
    /// Returns 81-char representation, `'0'` for empty.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..9 {
            for c in 0..9 {
                let v = self.0[r][c];
                write!(f, "{}", v)?;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Board internals
// ---------------------------------------------------------------------------

fn backtrack_board(b: &mut Board) -> bool {
    let Some((r, c)) = find_empty_board(b) else {
        return true;
    };
    let mut vals: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    with_rng(|rng| vals.shuffle(rng));
    for v in vals {
        if is_safe_board(b, r, c, v) {
            b.0[r][c] = v;
            if backtrack_board(b) {
                return true;
            }
            b.0[r][c] = 0;
        }
    }
    false
}

fn find_empty_board(b: &Board) -> Option<(usize, usize)> {
    for r in 0..9 {
        for c in 0..9 {
            if b.0[r][c] == 0 {
                return Some((r, c));
            }
        }
    }
    None
}

fn is_safe_board(b: &Board, r: usize, c: usize, v: u8) -> bool {
    for i in 0..9 {
        if b.0[r][i] == v || b.0[i][c] == v {
            return false;
        }
    }
    let br = (r / 3) * 3;
    let bc = (c / 3) * 3;
    for i in 0..3 {
        for j in 0..3 {
            if b.0[br + i][bc + j] == v {
                return false;
            }
        }
    }
    true
}

fn count_clues_board(b: &Board) -> usize {
    b.0.iter().flatten().filter(|&&v| v != 0).count()
}

fn has_unique_solution_board(b: &Board, limit: usize) -> bool {
    let mut count = 0usize;
    let mut work = *b;

    fn dfs(cur: &mut Board, count: &mut usize, limit: usize) -> bool {
        let Some((r, c)) = find_empty_board(cur) else {
            *count += 1;
            return *count >= limit;
        };
        for v in 1..=9u8 {
            if is_safe_board(cur, r, c, v) {
                cur.0[r][c] = v;
                if dfs(cur, count, limit) {
                    return true;
                }
                cur.0[r][c] = 0;
            }
        }
        false
    }

    dfs(&mut work, &mut count, limit);
    count == 1
}

fn fill_diagonal_boxes_board(b: &mut Board) {
    for d in (0..9).step_by(3) {
        fill_box_board(b, d, d);
    }
}

fn fill_box_board(b: &mut Board, br: usize, bc: usize) {
    let mut vals: Vec<u8> = (1..=9).collect();
    with_rng(|rng| vals.shuffle(rng));
    let mut idx = 0;
    for r in 0..3 {
        for c in 0..3 {
            b.0[br + r][bc + c] = vals[idx];
            idx += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_row_duplicate() {
        let mut b = Board::new();
        b.0[0][0] = 5;
        b.0[0][1] = 5;
        assert_eq!(b.validate(), Err(SudokuError::InvalidBoard));
    }

    #[test]
    fn test_validate_col_duplicate() {
        let mut b = Board::new();
        b.0[0][0] = 7;
        b.0[1][0] = 7;
        assert_eq!(b.validate(), Err(SudokuError::InvalidBoard));
    }

    #[test]
    fn test_validate_box_duplicate() {
        let mut b = Board::new();
        b.0[0][0] = 3;
        b.0[1][1] = 3;
        assert_eq!(b.validate(), Err(SudokuError::InvalidBoard));
    }

    #[test]
    fn test_solve_simple() {
        let input =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let b: Board = input.parse().unwrap();
        let solved = b.solve().expect("should solve");
        assert!(solved.validate().is_ok());
    }

    #[test]
    fn test_solve_unsolvable() {
        // Row 0 has 1..8, col 0 has 9 → no legal move at (0,0)
        let mut b = Board::new();
        for c in 1..9 {
            b.0[0][c] = c as u8; // 1..8
        }
        b.0[1][0] = 9;
        assert!(b.solve().is_none());
    }

    #[test]
    fn test_generate() {
        set_rand_seed(42);
        let puz = Board::generate(Difficulty::Easy, 1).expect("should generate");
        assert!(puz.validate().is_ok());
        assert!(has_unique_solution_board(&puz, 2));
    }

    #[test]
    fn test_generate_all_difficulties() {
        for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            let b = Board::generate(d, 1).unwrap_or_else(|_| panic!("generate {:?}", d));
            assert!(b.validate().is_ok());
            assert!(has_unique_solution_board(&b, 2));
        }
    }

    #[test]
    fn test_hint() {
        let input =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let b: Board = input.parse().unwrap();
        let (r, c, v) = b.hint().expect("should give hint");
        assert!(r < 9 && c < 9 && (1..=9).contains(&v));
    }

    #[test]
    fn test_board_display() {
        let input =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let b: Board = input.parse().unwrap();
        let s = b.to_string();
        assert_eq!(s.len(), 81);
    }

    #[test]
    fn test_difficulty_from_str() {
        assert_eq!("easy".parse::<Difficulty>().unwrap(), Difficulty::Easy);
        assert_eq!("Medium".parse::<Difficulty>().unwrap(), Difficulty::Medium);
        assert_eq!("HARD".parse::<Difficulty>().unwrap(), Difficulty::Hard);
        assert!("unknown".parse::<Difficulty>().is_err());
    }
}
