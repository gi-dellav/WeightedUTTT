use crate::defs::{Cell, Coord, Grid, Player};

#[derive(Clone, Copy)]
pub struct WeightedParameters {
    pub take_cell: f32,          // 0~1.0
    pub take_double_cell: f32,   // 0~1.0
    pub take_double_grid: f32,   // 0~1.0
    pub stop_cell: f32,          // 0~1.0
    pub stop_win: f32,           // 0~1.0
    pub stop_double_cell: f32,   // 0~1.0
    pub stop_double_grid: f32,   // 0~1.0
    pub giving_cell: f32,        // -1.0~1.0
    pub giving_double_cell: f32, // -1.0~1.0
    pub giving_double_grid: f32, // -1.0~1.0
    pub play_corner: f32,        // -1.0~1.0
    pub play_sides: f32,         // -1.0~1.0
    pub play_center: f32,        // -1.0~1.0
    pub best_enemy_move: f32,    // -1.0~1.0
}

#[derive(Clone, Copy)]
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

impl Player for WeightedPlayer {
    fn reset(&self) {
        // No reset logic needed
    }

    fn select_move(&self, grid: Grid, last_move: Option<Coord>) -> Coord {
        todo!("Implement WeightedPlayer");
    }
}
