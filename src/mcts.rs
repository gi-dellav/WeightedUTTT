
use crate::defs::{Coord, Grid, Player};

pub struct MCTSPlayer;

impl MCTSPlayer {
    pub fn new() -> Self {
        Self
    }
}

impl Player for MCTSPlayer {
    fn reset(&self) {}

    fn select_move(&self, _grid: Grid, _last_move: Option<Coord>) -> Coord {
        todo!("Implement MCTS logic");
    }
}
