//! Parsing: string ↔ Board / Grid conversion.

use crate::board::{Board, SudokuError};
use crate::grid::Grid;
use std::str::FromStr;

/// Maximum allowed grid size to prevent excessive memory usage.
pub const MAX_GRID_SIZE: usize = 25;

// ---------------------------------------------------------------------------
// Board parsing (81-char string)
// ---------------------------------------------------------------------------

impl FromStr for Board {
    type Err = SudokuError;

    /// Parses an 81-char string into a Board.
    /// Digits `1-9` are values; `0` or `'.'` are empty.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 81 {
            return Err(SudokuError::InvalidInput(
                "input must be 81 characters".into(),
            ));
        }
        let mut b = Board::new();
        for (i, ch) in s.bytes().enumerate() {
            let r = i / 9;
            let c = i % 9;
            match ch {
                b'1'..=b'9' => b.0[r][c] = ch - b'0',
                b'0' | b'.' => b.0[r][c] = 0,
                _ => {
                    return Err(SudokuError::InvalidInput(
                        "invalid character in board".into(),
                    ));
                }
            }
        }
        b.validate()?;
        Ok(b)
    }
}

// ---------------------------------------------------------------------------
// Grid parsing (size*size char string)
// ---------------------------------------------------------------------------

impl Grid {
    /// Parses a `size*size` character string into a Grid.
    /// Digits `1-9` are values; `0` or `'.'` are empty. Supports sizes up to 9.
    pub fn from_string_n(
        s: &str,
        size: usize,
        box_rows: usize,
        box_cols: usize,
    ) -> Result<Self, SudokuError> {
        if size != box_rows * box_cols {
            return Err(SudokuError::InvalidInput(format!(
                "invalid dims: size={size} box_rows={box_rows} box_cols={box_cols}"
            )));
        }
        let expected = size * size;
        if s.len() != expected {
            return Err(SudokuError::InvalidInput(format!(
                "input must be {expected} characters"
            )));
        }
        let mut g = Grid::new(size, box_rows, box_cols)?;
        for (i, ch) in s.bytes().enumerate() {
            let r = i / size;
            let c = i % size;
            match ch {
                b'1'..=b'9' => {
                    let v = (ch - b'0') as usize;
                    if v > size {
                        return Err(SudokuError::InvalidInput("digit exceeds grid size".into()));
                    }
                    g.cells[r][c] = v as u8;
                }
                b'0' | b'.' => g.cells[r][c] = 0,
                _ => {
                    return Err(SudokuError::InvalidInput(
                        "invalid character in grid".into(),
                    ));
                }
            }
        }
        g.validate()?;
        Ok(g)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string_errors() {
        // Wrong length
        assert!("123".parse::<Board>().is_err());

        // Invalid character
        let mut bad = vec![b'0'; 81];
        bad[10] = b'x';
        let s = String::from_utf8(bad).unwrap();
        assert!(s.parse::<Board>().is_err());

        // Duplicate in a row should fail via validate
        let mut dup = String::from("11");
        dup.push_str(&"0".repeat(79));
        assert!(dup.parse::<Board>().is_err());
    }

    #[test]
    fn test_board_string_roundtrip() {
        let input =
            "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
        let b: Board = input.parse().unwrap();
        let out = b.to_string();
        assert_eq!(out.len(), 81);
    }

    #[test]
    fn test_parse_and_string() {
        let input =
            "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
        let b: Board = input.parse().unwrap();
        assert_eq!(b.to_string().len(), 81);
    }

    #[test]
    fn test_from_string_n_4x4() {
        let s = "0000340000430000";
        let g = Grid::from_string_n(s, 4, 2, 2).unwrap();
        assert_eq!(g.cells.len(), 4);
        assert_eq!(g.cells[0].len(), 4);
    }

    #[test]
    fn test_from_string_n_size_mismatch() {
        assert!(Grid::from_string_n("0000", 6, 2, 3).is_err());
    }

    #[test]
    fn test_from_string_n_invalid_char() {
        assert!(Grid::from_string_n("x0", 1, 1, 1).is_err());
    }
}
