
#[derive(Clone, Copy, Debug)]
pub enum Cell {
    Empty,
    Cross,
    Circle
}
#[derive(Clone, Copy, Debug)]
pub struct Minigrid {
    pub matrix: [Cell; 9],
}
#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub matrix: [Minigrid; 9],
    pub completed_minigrid: [bool; 9],
}
#[derive(Clone, Copy, Debug)]
pub struct Coord {
    pub meta_x: u8,
    pub meta_y: u8,
    pub x: u8,
    pub y: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct MatchStats {
    pub winner: Cell,
    pub number_turns: u8,
    pub final_grid: Grid,
}

impl Default for Cell {
    fn default() -> Cell { Cell::Empty }
}
impl Default for Minigrid {
    fn default() -> Minigrid {
        return Minigrid{ matrix: [Cell::Empty; 9] };
     }
}
impl Default for Grid {
    fn default() -> Grid {
        return Grid {
            matrix: [Default::default(); 9],
            completed_minigrid: [false; 9],
        }
    }
}

impl Grid {
    fn set(&mut self, coord: Coord, symbol: Cell) {
        todo!();
    }
    fn update_grid(&mut self) {
        todo!();
    }
    fn is_completed(self) -> Option<Cell> {
        // If the grid is completed, returns the symbol of the winner
        todo!();
    }
}

pub trait Player {
    /// Run before playing a match
    fn reset(self);
    fn select_move(self, grid: Grid, last_move: Option<Coord>) -> Coord;
}

pub fn play_match<A: Player + Copy, B: Player + Copy>(a: A, b: B) -> MatchStats {
    a.reset();
    b.reset();

    let mut grid: Grid = Default::default();
    let mut last_move: Option<Coord> = None;
    let mut last_player: Cell = Cell::Empty;
    let mut number_turns: u8 = 0;

    while true {
        let a_coord = a.select_move(grid, last_move);
        grid.set(a_coord, Cell::Cross);
        grid.update_grid();

        let grid_completed: Option<Cell> = grid.is_completed();
        if grid_completed.is_some() {
            last_player = grid_completed.unwrap();
            break;
        }

        last_player = Cell::Cross;
        number_turns += 1;

        // ---

        let b_coord = b.select_move(grid, last_move);
        grid.set(b_coord, Cell::Circle);
        grid.update_grid();

        let grid_completed: Option<Cell> = grid.is_completed();
        if grid_completed.is_some() {
            last_player = grid_completed.unwrap();
            break;
        }

        last_player = Cell::Circle;
        number_turns += 1;
    }

    return MatchStats {
        winner: last_player,
        number_turns,
        final_grid: grid,
    }
}
