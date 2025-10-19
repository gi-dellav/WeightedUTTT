use cached::proc_macro::cached;

use crate::defs::{Cell, Coord, Grid, Player};

const CENTER_COORD: Coord = Coord {
    meta_x: 1,
    meta_y: 1,
    x: 1,
    y: 1,
};
const LINES_3: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

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
    pub best_enemy_move: i32,    // -1.0~0.0
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
/// This function is needed as WeightedParameters store parameters as i32 even tho they are f32 values.
fn convert(v: i32) -> f32 {
    return (v * 1000000000) as f32;
}

fn is_win_in_cells(cells: &[Cell; 9], player: Cell) -> bool {
    for line in &LINES_3 {
        if cells[line[0]] == player && cells[line[1]] == player && cells[line[2]] == player {
            return true;
        }
    }
    false
}

/// Returns true if there exists a line with exactly 2 `player` and 1 `Empty`.
fn has_completable_two_in_row(cells: &[Cell; 9], player: Cell) -> bool {
    for line in &LINES_3 {
        let a = cells[line[0]];
        let b = cells[line[1]];
        let c = cells[line[2]];
        let count_player = (a == player) as usize + (b == player) as usize + (c == player) as usize;
        let count_empty =
            (a == Cell::Empty) as usize + (b == Cell::Empty) as usize + (c == Cell::Empty) as usize;
        if count_player == 2 && count_empty == 1 {
            return true;
        }
    }
    false
}

#[cached]
fn eval_board(
    params: WeightedParameters,
    grid: Grid,
    eval_move: Coord,
    player_symbol: Cell,
) -> f32 {
    let mut score: f32 = 0.0;
    let meta_x: usize = eval_move.meta_x as usize;
    let meta_y: usize = eval_move.meta_y as usize;
    let x: usize = eval_move.x as usize;
    let y: usize = eval_move.y as usize;
    let target_cell = grid.matrix[meta_x].matrix[meta_y];

    let mut hypothetical_cells = grid.matrix[meta_x].matrix;
    hypothetical_cells[x] = player_symbol;

    // Apply take_cell
    if is_win_in_cells(&hypothetical_cells, player_symbol) {
        score += convert(params.take_cell);
    }

    // Apply take_double_cell

    // Apply take_double_grid

    // Apply stop_cell

    // Apply stop_win

    // Apply stop_double_cell

    // Apply stop_double_grid

    // Apply play_corner, _sides and _center
    if eval_move == CENTER_COORD {
        score += convert(params.play_center)
    } else if (eval_move.x == 0 || eval_move.x == 2) && (eval_move.y == 0 || eval_move.y == 2) {
        score += convert(params.play_corner)
    } else {
        score += convert(params.play_sides)
    }

    // Apply giving_cell

    // Apply giving_double_cell

    // Apply giving_double_grid

    // NOTE: best_enemy_move is not applied by this function

    return score;
}

impl Player for WeightedPlayer {
    fn reset(&self) {
        // No reset logic needed
    }

    fn select_move(
        &self,
        _grid: Grid,
        _legal_moves: Vec<Coord>,
        _last_move: Option<Coord>,
    ) -> Coord {
        todo!("Implement WeightedPlayer");
    }
}
