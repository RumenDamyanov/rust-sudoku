//! Generalised Sudoku grid of size S×S with sub-boxes `box_rows × box_cols`.

use std::fmt;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::board::{Difficulty, SudokuError};

// Re-use the thread-local RNG from board.
use crate::board::with_rng;

/// Maximum allowed grid size.
pub use crate::parse::MAX_GRID_SIZE;

/// A generalised Sudoku grid. Values in `[0..size]`, 0 = empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid {
    pub size: usize,
    pub box_rows: usize,
    pub box_cols: usize,
    pub cells: Vec<Vec<u8>>,
}

impl Grid {
    /// Creates an empty grid with given dimensions.
    pub fn new(size: usize, box_rows: usize, box_cols: usize) -> Result<Grid, SudokuError> {
        if size == 0 || box_rows == 0 || box_cols == 0 || size != box_rows * box_cols {
            return Err(SudokuError::InvalidInput(format!(
                "invalid dimensions: size={size} box_rows={box_rows} box_cols={box_cols}"
            )));
        }
        if size > MAX_GRID_SIZE {
            return Err(SudokuError::InvalidInput(format!(
                "grid size {size} exceeds maximum allowed ({MAX_GRID_SIZE})"
            )));
        }
        Ok(Grid {
            size,
            box_rows,
            box_cols,
            cells: vec![vec![0u8; size]; size],
        })
    }

    /// Deep copy.
    pub fn clone_grid(&self) -> Grid {
        self.clone()
    }

    /// Validates that values are in `[0..size]` and no row/col/box duplicates.
    pub fn validate(&self) -> Result<(), SudokuError> {
        let s = self.size;
        for i in 0..s {
            let mut row = vec![false; s + 1];
            let mut col = vec![false; s + 1];
            for j in 0..s {
                let rv = self.cells[i][j] as usize;
                let cv = self.cells[j][i] as usize;
                if rv > s || cv > s {
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
        // boxes
        for br in (0..s).step_by(self.box_rows) {
            for bc in (0..s).step_by(self.box_cols) {
                let mut seen = vec![false; s + 1];
                for r in br..br + self.box_rows {
                    for c in bc..bc + self.box_cols {
                        let v = self.cells[r][c] as usize;
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

    /// Solves the grid using backtracking. Returns `Some(solved)` or `None`.
    pub fn solve(&self) -> Option<Grid> {
        let mut work = self.clone();
        if self.backtrack(&mut work) {
            Some(work)
        } else {
            None
        }
    }

    /// Generates a puzzle with a unique solution.
    pub fn generate(&self, difficulty: Difficulty, attempts: usize) -> Result<Grid, SudokuError> {
        let attempts = attempts.max(1);
        let mut last_err = None;

        for _ in 0..attempts {
            let mut solved = self.clone();
            solved.fill_diagonal_boxes();
            if !self.backtrack(&mut solved) {
                last_err = Some("failed to build solved grid".to_string());
                continue;
            }
            let target = self.clues_for(difficulty);
            let mut puzzle = solved.clone();
            let total = self.size * self.size;
            let mut rm_order: Vec<usize> = (0..total).collect();
            with_rng(|rng| rm_order.shuffle(rng));

            for &idx in &rm_order {
                if self.count_clues(&puzzle) <= target {
                    break;
                }
                let r = idx / self.size;
                let c = idx % self.size;
                let old = puzzle.cells[r][c];
                if old == 0 {
                    continue;
                }
                puzzle.cells[r][c] = 0;
                if !self.has_unique_solution(&puzzle, 2) {
                    puzzle.cells[r][c] = old;
                }
            }
            if self.has_unique_solution(&puzzle, 2) {
                return Ok(puzzle);
            }
            last_err = Some("puzzle uniqueness not achieved".to_string());
        }

        Err(SudokuError::GenerationFailed(
            last_err.unwrap_or_else(|| "generation failed".to_string()),
        ))
    }

    /// Returns a suggested value for the grid: `(row, col, value)`.
    pub fn hint(&self) -> Option<(usize, usize, u8)> {
        if self.validate().is_err() {
            return None;
        }
        if let Some(sol) = self.solve() {
            for r in 0..self.size {
                for c in 0..self.size {
                    if self.cells[r][c] == 0 {
                        return Some((r, c, sol.cells[r][c]));
                    }
                }
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn backtrack(&self, w: &mut Grid) -> bool {
        let Some((r, c)) = self.find_empty(w) else {
            return true;
        };
        let mut vals: Vec<u8> = (1..=self.size as u8).collect();
        with_rng(|rng| vals.shuffle(rng));
        for v in vals {
            if self.is_safe(w, r, c, v) {
                w.cells[r][c] = v;
                if self.backtrack(w) {
                    return true;
                }
                w.cells[r][c] = 0;
            }
        }
        false
    }

    fn find_empty(&self, w: &Grid) -> Option<(usize, usize)> {
        for r in 0..self.size {
            for c in 0..self.size {
                if w.cells[r][c] == 0 {
                    return Some((r, c));
                }
            }
        }
        None
    }

    fn is_safe(&self, w: &Grid, r: usize, c: usize, v: u8) -> bool {
        for i in 0..self.size {
            if w.cells[r][i] == v || w.cells[i][c] == v {
                return false;
            }
        }
        let br = (r / self.box_rows) * self.box_rows;
        let bc = (c / self.box_cols) * self.box_cols;
        for i in 0..self.box_rows {
            for j in 0..self.box_cols {
                if w.cells[br + i][bc + j] == v {
                    return false;
                }
            }
        }
        true
    }

    fn clues_for(&self, d: Difficulty) -> usize {
        let base = d.clues_for_9x9();
        if self.size == 9 {
            return base;
        }
        let cells = self.size * self.size;
        (cells * base / 81).max(8)
    }

    fn count_clues(&self, w: &Grid) -> usize {
        w.cells.iter().flatten().filter(|&&v| v != 0).count()
    }

    fn has_unique_solution(&self, w: &Grid, limit: usize) -> bool {
        let mut count = 0usize;
        let mut work = w.clone();

        fn dfs(grid: &Grid, cur: &mut Grid, count: &mut usize, limit: usize) -> bool {
            let Some((r, c)) = grid.find_empty(cur) else {
                *count += 1;
                return *count >= limit;
            };
            for v in 1..=grid.size as u8 {
                if grid.is_safe(cur, r, c, v) {
                    cur.cells[r][c] = v;
                    if dfs(grid, cur, count, limit) {
                        return true;
                    }
                    cur.cells[r][c] = 0;
                }
            }
            false
        }

        dfs(self, &mut work, &mut count, limit);
        count == 1
    }

    fn fill_diagonal_boxes(&mut self) {
        let n_row_boxes = self.size / self.box_rows;
        let n_col_boxes = self.size / self.box_cols;
        let steps = n_row_boxes.min(n_col_boxes);
        for i in 0..steps {
            let br = i * self.box_rows;
            let bc = i * self.box_cols;
            self.fill_box(br, bc);
        }
    }

    fn fill_box(&mut self, br: usize, bc: usize) {
        let mut vals: Vec<u8> = (1..=self.size as u8).collect();
        with_rng(|rng| vals.shuffle(rng));
        let mut idx = 0;
        for r in 0..self.box_rows {
            for c in 0..self.box_cols {
                self.cells[br + r][bc + c] = vals[idx];
                idx += 1;
            }
        }
    }
}

impl fmt::Display for Grid {
    /// Returns compact representation (`size*size` chars, `'0'` for empty).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..self.size {
            for c in 0..self.size {
                write!(f, "{}", self.cells[r][c])?;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid_errors() {
        assert!(Grid::new(9, 2, 5).is_err()); // 2*5 != 9
    }

    #[test]
    fn test_grid_validate_and_solve_4x4() {
        let mut g = Grid::new(4, 2, 2).unwrap();
        g.cells = vec![
            vec![0, 0, 3, 4],
            vec![3, 4, 0, 0],
            vec![0, 0, 4, 3],
            vec![4, 3, 0, 0],
        ];
        assert!(g.validate().is_ok());
        let sol = g.solve().expect("should solve 4x4");
        assert!(sol.validate().is_ok());
    }

    #[test]
    fn test_grid_generate_and_hint() {
        for (size, br, bc) in [(4, 2, 2), (6, 2, 3), (9, 3, 3)] {
            let g = Grid::new(size, br, bc).unwrap();
            let mut puz = None;
            for _ in 0..3 {
                if let Ok(p) = g.generate(Difficulty::Medium, 1) {
                    puz = Some(p);
                    break;
                }
            }
            let puz = puz.unwrap_or_else(|| panic!("generate failed for size {size}"));
            assert!(puz.validate().is_ok());
            let (r, c, v) = puz
                .hint()
                .unwrap_or_else(|| panic!("hint failed for size {size}"));
            assert!(r < size && c < size && v >= 1);
        }
    }

    #[test]
    fn test_grid_display() {
        let mut g = Grid::new(4, 2, 2).unwrap();
        g.cells = vec![
            vec![1, 2, 3, 4],
            vec![3, 4, 1, 2],
            vec![2, 1, 4, 3],
            vec![4, 3, 2, 1],
        ];
        assert_eq!(g.to_string(), "1234341221434321");
    }
}
