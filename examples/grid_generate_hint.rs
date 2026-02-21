//! Example: variable-sized grid generation and hint.

use rumenx_sudoku::{Difficulty, Grid, set_rand_seed};

fn main() {
    set_rand_seed(42);
    let g = Grid::new(6, 2, 3).unwrap();
    let p = g.generate(Difficulty::Easy, 2).unwrap();
    if let Some((r, c, v)) = p.hint() {
        println!("hint-ok: true cell: {r} {c} val: {v}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_generate_hint() {
        set_rand_seed(42);
        let g = Grid::new(6, 2, 3).unwrap();
        let p = g.generate(Difficulty::Easy, 2).unwrap();
        let (r, c, v) = p.hint().expect("should give hint");
        assert!(r < 6 && c < 6 && v >= 1);
    }
}
