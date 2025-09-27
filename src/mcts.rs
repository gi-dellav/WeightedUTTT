
use crate::defs::{Cell, Coord, Grid, Player};

impl Player for super::mcts::MCTSPlayer {
    fn reset(self) {
        // Reset dello stato MCTS se necessario
    }

    fn select_move(self, grid: Grid, last_move: Option<Coord>) -> Coord {
        // TODO: Implementare la logica MCTS
        todo!("Implementare la selezione della mossa per MCTSPlayer")
    }
}
