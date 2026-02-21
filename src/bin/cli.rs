//! sudoku-cli — Command-line interface for sudoku generation, solving, and hints.

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use rumenx_sudoku::{Board, Difficulty, Grid};

/// Version info injected at build time (via env vars or defaults).
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "sudoku-cli", about = "Sudoku generator, solver & hint tool")]
struct Cli {
    /// Difficulty: easy|medium|hard (for generation)
    #[arg(short, long, default_value = "medium")]
    difficulty: String,

    /// Generation attempts for uniqueness (>= 1)
    #[arg(short, long, default_value_t = 3)]
    attempts: usize,

    /// When generating, also show solution
    #[arg(short, long)]
    solve: bool,

    /// Grid size (SxS), e.g. 4, 6, 9
    #[arg(long, default_value_t = 9)]
    size: usize,

    /// Sub-box dims RxC, e.g. 2x2, 2x3, 3x3
    #[arg(long, default_value = "3x3")]
    r#box: String,

    /// Print a hint for the provided board/string
    #[arg(long)]
    hint: bool,

    /// Solve: puzzle string (0 or . for empty)
    #[arg(long)]
    string: Option<String>,

    /// Solve: path to file containing puzzle string
    #[arg(long)]
    file: Option<String>,

    /// Print output as JSON
    #[arg(long)]
    json: bool,

    /// Print version and exit
    #[arg(long)]
    version: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(&cli, &mut stdout, &mut stderr);
    use std::io::Write as _;
    let _ = std::io::stdout().write_all(&stdout);
    let _ = std::io::stderr().write_all(&stderr);
    code
}

fn run_cli(cli: &Cli, mut stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode {
    if cli.version {
        let _ = writeln!(stdout, "sudoku-cli {VERSION} (commit none, built unknown)");
        return ExitCode::SUCCESS;
    }

    // If a puzzle string or file is provided, solve/hint it
    if cli.string.is_some() || cli.file.is_some() {
        let s = if let Some(ref path) = cli.file {
            match std::fs::read_to_string(path) {
                Ok(content) => content.trim().to_string(),
                Err(e) => {
                    let _ = writeln!(stderr, "error: {e}");
                    return ExitCode::FAILURE;
                }
            }
        } else {
            cli.string.as_deref().unwrap_or("").trim().to_string()
        };

        let board: Board = match s.parse() {
            Ok(b) => b,
            Err(e) => {
                let _ = writeln!(stderr, "error: {e}");
                return ExitCode::FAILURE;
            }
        };

        if cli.hint {
            match board.hint() {
                Some((r, c, v)) => {
                    if cli.json {
                        let _ = writeln!(
                            stdout,
                            "{}",
                            serde_json::json!({"row": r, "col": c, "val": v})
                        );
                    } else {
                        let _ = writeln!(stdout, "Hint: row {}, col {} = {}", r + 1, c + 1, v);
                    }
                    return ExitCode::SUCCESS;
                }
                None => {
                    let _ = writeln!(stderr, "error: no hint available");
                    return ExitCode::FAILURE;
                }
            }
        }

        match board.solve() {
            Some(solved) => {
                if cli.json {
                    let _ = writeln!(
                        stdout,
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({"solution": solved}))
                            .unwrap()
                    );
                } else {
                    let _ = writeln!(stdout, "Solution:");
                    print_board_to(&mut stdout, &solved);
                }
                ExitCode::SUCCESS
            }
            None => {
                let _ = writeln!(stderr, "error: unsolvable puzzle");
                ExitCode::FAILURE
            }
        }
    } else {
        // Generate mode
        let d: Difficulty = match cli.difficulty.parse() {
            Ok(d) => d,
            Err(_) => {
                let _ = writeln!(stderr, "error: invalid difficulty: {}", cli.difficulty);
                return ExitCode::from(2);
            }
        };

        // Parse box dims
        let parts: Vec<&str> = cli.r#box.split('x').collect();
        let (br, bc) = if parts.len() == 2 {
            match (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                (Ok(r), Ok(c)) if r > 0 && c > 0 && r * c == cli.size => (r, c),
                _ => {
                    let _ = writeln!(stderr, "error: invalid box dims; ensure size == R*C");
                    return ExitCode::from(2);
                }
            }
        } else {
            let _ = writeln!(stderr, "error: invalid box dims; ensure size == R*C");
            return ExitCode::from(2);
        };

        if cli.size == 9 && br == 3 && bc == 3 {
            // Classic 9×9
            match Board::generate(d, cli.attempts) {
                Ok(puz) => {
                    if cli.json {
                        let mut out = serde_json::json!({"puzzle": puz});
                        if cli.solve
                            && let Some(sol) = puz.solve()
                        {
                            out["solution"] = serde_json::to_value(sol).unwrap();
                        }
                        let _ = writeln!(stdout, "{}", serde_json::to_string_pretty(&out).unwrap());
                    } else {
                        let _ = writeln!(stdout, "Generated ({d}):");
                        print_board_to(&mut stdout, &puz);
                        if cli.solve
                            && let Some(sol) = puz.solve()
                        {
                            let _ = writeln!(stdout, "\nSolution:");
                            print_board_to(&mut stdout, &sol);
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    let _ = writeln!(stderr, "error: {e}");
                    ExitCode::FAILURE
                }
            }
        } else {
            // Variable size
            let g = match Grid::new(cli.size, br, bc) {
                Ok(g) => g,
                Err(e) => {
                    let _ = writeln!(stderr, "error: {e}");
                    return ExitCode::FAILURE;
                }
            };
            match g.generate(d, cli.attempts) {
                Ok(gpuz) => {
                    if cli.json {
                        let out = serde_json::json!({
                            "size": gpuz.size,
                            "boxR": gpuz.box_rows,
                            "boxC": gpuz.box_cols,
                            "board": gpuz.to_string()
                        });
                        let _ = writeln!(stdout, "{}", serde_json::to_string_pretty(&out).unwrap());
                    } else {
                        let _ = writeln!(
                            stdout,
                            "{}x{} ({}x{} boxes)",
                            gpuz.size, gpuz.size, gpuz.box_rows, gpuz.box_cols
                        );
                        for r in 0..gpuz.size {
                            for c in 0..gpuz.size {
                                let v = gpuz.cells[r][c];
                                if v == 0 {
                                    let _ = write!(stdout, ".");
                                } else {
                                    let _ = write!(stdout, "{v}");
                                }
                                if c < gpuz.size - 1 {
                                    let _ = write!(stdout, " ");
                                }
                            }
                            let _ = writeln!(stdout);
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    let _ = writeln!(stderr, "error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn print_board_to(w: &mut impl Write, b: &Board) {
    let line = "+-------+-------+-------+";
    let _ = writeln!(w, "{line}");
    for r in 0..9 {
        let _ = write!(w, "|");
        for c in 0..9 {
            let v = b.0[r][c];
            let ch = if v == 0 { '.' } else { (b'0' + v) as char };
            let sep = if (c + 1) % 3 == 0 { " |" } else { " " };
            let _ = write!(w, " {ch}{sep}");
        }
        let _ = writeln!(w);
        if (r + 1) % 3 == 0 {
            let _ = writeln!(w, "{line}");
        }
    }
}
