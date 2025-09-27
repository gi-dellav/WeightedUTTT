use crate::defs::*;
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
        // TODO: Implementare la logica per leggere le coordinate dall'utente
        clear_term();

        println!("-----------------------------");
        println!("- {:?}'s TURN -", self.symbol);
        print_grid(&grid);

        // TODO: Implementare la logica per leggere l'input dell'utente
        // TODO: Validare la mossa rispetto alle regole del gioco
        // TODO: Gestire errori di input

        println!("Select grid X: ");
        println!("Select grid Y: ");
        println!("Select minigrid X: ");
        println!("Select minigrid Y: ");

        todo!("Implement HumanPlayer")
    }
}
