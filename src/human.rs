use crate::defs::*;
use std::io::{self, Write as IoWrite};
pub fn input_str() -> String {
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
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

    fn select_move(&self, grid: Grid, legal_moves: Vec<Coord>, _last_move: Option<Coord>) -> Coord {
        clear_term();
        println!("-----------------------------");
        println!("- {:?}'s TURN -", self.symbol);
        print_grid(&grid);

        loop {
            println!("Enter coordinates (meta_x meta_y x y) between 0-2 separated by spaces:");
            print!("> ");
            io::stdout().flush().unwrap();

            let input = input_str();
            let parts: Vec<&str> = input.split_whitespace().collect();

            // Parse input coordinates
            let coords = match parts.as_slice() {
                [mx, my, x, y] => match (mx.parse(), my.parse(), x.parse(), y.parse()) {
                    (Ok(mx), Ok(my), Ok(x), Ok(y)) => Some(Coord {
                        meta_x: mx,
                        meta_y: my,
                        x,
                        y,
                    }),
                    _ => {
                        println!("Invalid input: Please enter numbers between 0-2");
                        continue;
                    }
                },
                _ => {
                    println!("Invalid format: Expected 4 numbers separated by spaces");
                    continue;
                }
            };

            // Validate coordinates
            if let Some(coord) = coords {
                // Check all coordinates are within 0-2 range
                if [coord.meta_x, coord.meta_y, coord.x, coord.y]
                    .iter()
                    .any(|&v| v > 2)
                {
                    println!("Coordinates must be between 0-2");
                    continue;
                }

                // Check if move is legal by comparing all fields
                if legal_moves.iter().any(|c| {
                    c.meta_x == coord.meta_x
                        && c.meta_y == coord.meta_y
                        && c.x == coord.x
                        && c.y == coord.y
                }) {
                    // Create a temporary grid with the move applied
                    let mut temp_grid = grid;
                    temp_grid.set(coord, self.symbol);
                    return coord;
                } else {
                    println!("Invalid move: That position is not playable according to game rules");
                    println!("Legal moves are:");
                    for m in &legal_moves {
                        println!(
                            "- Meta: ({},{}) Local: ({},{})",
                            m.meta_x, m.meta_y, m.x, m.y
                        );
                    }
                }
            }
        }
    }
}
