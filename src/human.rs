use crate::defs::{Cell, Coord, Grid, Player};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{self, Write as IoWrite, stdout};
use std::fmt::Write as FmtWrite;

pub fn clear_term() {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
}
/// Convert a Cell into a printable char
fn cell_char(c: Cell) -> char {
    match c {
        Cell::Empty => '.',
        Cell::Cross => 'X',
        Cell::Circle => 'O',
    }
}
pub fn print_grid(g: &Grid) {
    let mut out = String::new();

    for meta_y in 0..3 {
        for local_y in 0..3 {
            for meta_x in 0..3 {
                let mg_idx = (meta_y * 3 + meta_x) as usize;
                let mg = &g.matrix[mg_idx];

                for local_x in 0..3 {
                    let cell_idx = (local_y * 3 + local_x) as usize;
                    let ch = cell_char(mg.matrix[cell_idx]);
                    // space before each cell for readability
                    write!(out, " {}", ch).unwrap();
                }

                // vertical separator between minigrids (but not after last)
                if meta_x < 2 {
                    write!(out, " |").unwrap();
                }
            }
            out.push('\n');
        }

        if meta_y < 2 {
            out.push_str("-------+-------+-------\n");
        }
    }

    out.push_str("\n\n");
    out.push_str("Completed minigrids (winner per 3x3 block):\n");
    for meta_y in 0..3 {
        for meta_x in 0..3 {
            let idx = (meta_y * 3 + meta_x) as usize;
            let ch = cell_char(g.completed_minigrid[idx]);
            write!(out, " {}", ch).unwrap();
        }
        out.push('\n');
    }

    print!("{}", out);
}

pub fn input_str() -> String {
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    return input.trim().to_string();
}

#[derive(Clone, Copy)]
pub struct HumanPlayer {
    symbol: Cell,
}

impl HumanPlayer {
    pub fn new(symbol: Cell) -> Self {
        Self { symbol }
    }
}

impl Player for HumanPlayer {
    fn reset(&self) {
        // Empty function, no reset logic needed
    }

    fn select_move(&self, grid: Grid, _last_move: Option<Coord>) -> Coord {
        clear_term();
        println!("-----------------------------");
        println!("- {:?}'s TURN -", self.symbol);
        print_grid(&grid);

        // Get all legal moves first for validation
        let legal_moves = grid.get_legal_moves(_last_move);

        loop {
            println!("Enter coordinates (meta_x meta_y x y) between 0-2 separated by spaces:");
            print!("> ");
            io::stdout().flush().unwrap();

            let input = input_str();
            let parts: Vec<&str> = input.split_whitespace().collect();

            // Parse input coordinates
            let coords = match parts.as_slice() {
                [mx, my, x, y] => {
                    match (mx.parse(), my.parse(), x.parse(), y.parse()) {
                        (Ok(mx), Ok(my), Ok(x), Ok(y)) => Some(Coord {
                            meta_x: mx,
                            meta_y: my,
                            x: x,
                            y: y,
                        }),
                        _ => {
                            println!("Invalid input: Please enter numbers between 0-2");
                            continue;
                        }
                    }
                }
                _ => {
                    println!("Invalid format: Expected 4 numbers separated by spaces");
                    continue;
                }
            };

            // Validate coordinates
            if let Some(coord) = coords {
                // Check all coordinates are within 0-2 range
                if [coord.meta_x, coord.meta_y, coord.x, coord.y].iter().any(|&v| v > 2) {
                    println!("Coordinates must be between 0-2");
                    continue;
                }

                // Check if move is legal by comparing all fields
                if legal_moves.iter().any(|c| 
                    c.meta_x == coord.meta_x &&
                    c.meta_y == coord.meta_y &&
                    c.x == coord.x &&
                    c.y == coord.y
                ) {
                    return coord;
                } else {
                    println!("Invalid move: That position is not playable according to game rules");
                    println!("Legal moves are:");
                    for m in legal_moves {
                        println!("- Meta: ({},{}) Local: ({},{})", m.meta_x, m.meta_y, m.x, m.y);
                    }
                }
            }
        }
    }
}
