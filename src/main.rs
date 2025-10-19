pub mod defs;
pub mod human;
pub mod mcts;
pub mod weighted;

use crate::defs::{play_match, Cell};
use crate::human::HumanPlayer;
use crate::mcts::MCTSPlayer;

fn main() {
    //let human = HumanPlayer::new(Cell::Cross);
    let ai = MCTSPlayer::new(1.5, 100, Cell::Circle); // Parametri da principiante
    let ai2 = MCTSPlayer::new(1.5, 100, Cell::Cross);

    for _ in 0..25 {
        let stats = play_match(ai2, ai);

        if stats.winner.is_none() {
            println!("Pareggio in {} turni", stats.number_turns);
        } else {
            println!(
                "Vittoria di {:?} in {} turni",
                stats.winner.unwrap(),
                stats.number_turns
            );
        }
    }
}
