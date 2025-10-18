use cached::proc_macro::cached;

use crate::defs::{Cell, Coord, Grid, Player};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct WeightedParameters {
    pub take_cell: i32,          // 0~1.0    (*10**9 for integer rappresentation)
    pub take_double_cell: i32,   // 0~1.0
    pub take_double_grid: i32,   // 0~1.0
    pub stop_cell: i32,          // 0~1.0
    pub stop_win: i32,           // 0~1.0
    pub stop_double_cell: i32,   // 0~1.0
    pub stop_double_grid: i32,   // 0~1.0
    pub giving_cell: i32,        // -1.0~1.0
    pub giving_double_cell: i32, // -1.0~1.0
    pub giving_double_grid: i32, // -1.0~1.0
    pub play_corner: i32,        // -1.0~1.0
    pub play_sides: i32,         // -1.0~1.0
    pub play_center: i32,        // -1.0~1.0
    pub best_enemy_move: i32,    // -1.0~1.0
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct WeightedPlayer {
    pub weighted_params: WeightedParameters,
    pub apply_softsign: bool,
    pub ignore_giving: bool,

    pub symbol: Cell,
}

impl WeightedPlayer {
    pub fn new(params: WeightedParameters, symbol: Cell) -> Self {
        Self {
            weighted_params: params,
            symbol,
            apply_softsign: false,
            ignore_giving: false,
        }
    }
}

#[cached]
fn eval_board(params: WeightedParameters, grid: Grid) -> f32 {
    todo!("Implement eval_board");
}

impl Player for WeightedPlayer {
    fn reset(&self) {
        // No reset logic needed
    }

    fn select_move(&self, grid: Grid, last_move: Option<Coord>) -> Coord {
        todo!("Implement WeightedPlayer");
    }
}
