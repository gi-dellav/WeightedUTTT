pub mod defs;
pub mod human;
pub mod mcts;
pub mod weighted;

use crate::defs::{play_match, Cell};
use crate::human::HumanPlayer;
use crate::mcts::MCTSPlayer;

fn main() {
    let human = HumanPlayer::new(Cell::Cross);
    let ai = MCTSPlayer::new(1.5, 800, Cell::Circle); // Parametri da principiante
    
    let stats = play_match(human, ai);
    
    println!("Risultato finale:");
    println!("Vincitore: {:?}", stats.winner);
    println!("Numero di turni: {}", stats.number_turns);
}
