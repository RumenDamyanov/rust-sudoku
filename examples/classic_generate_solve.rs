//! Example: classic 9×9 generation and solving.

use rumenx_sudoku::{Board, Difficulty, set_rand_seed};

fn main() {
    set_rand_seed(1);
    let puz = Board::generate(Difficulty::Medium, 2).unwrap();
    let prefix = &puz.to_string()[..9];
    println!("clues: {prefix}");
    if let Some(_sol) = puz.solve() {
        println!("solvable: true");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classic_generate_solve() {
        set_rand_seed(1);
        let puz = Board::generate(Difficulty::Medium, 2).unwrap();
        assert_eq!(puz.to_string().len(), 81);
        assert!(puz.solve().is_some());
    }
}
