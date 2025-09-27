#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Cross,
    Circle,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Minigrid {
    pub matrix: [Cell; 9],
}
#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub matrix: [Minigrid; 9],
    pub completed_minigrid: [Cell; 9],
}
#[derive(Clone, Copy, Debug, PartialEq)]
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
    fn default() -> Cell {
        Cell::Empty
    }
}
impl Default for Minigrid {
    fn default() -> Minigrid {
        return Minigrid {
            matrix: [Cell::Empty; 9],
        };
    }
}
impl Default for Grid {
    fn default() -> Grid {
        return Grid {
            matrix: [Default::default(); 9],
            completed_minigrid: [Cell::Empty; 9],
        };
    }
}
impl Minigrid {
    fn check(self, v1: usize, v2: usize, v3: usize) -> Option<Cell> {
        // All values must be non-empty and equal
        if self.matrix[v1] != Cell::Empty {
            if self.matrix[v1] == self.matrix[v2] && self.matrix[v2] == self.matrix[v3] {
                return Some(self.matrix[v1]);
            }
        }
        None
    }
}
impl Grid {
    fn set(&mut self, coord: Coord, symbol: Cell) {
        let minigrid_pos = (coord.meta_x + 3 * coord.meta_y) as usize;
        let mut minigrid = self.matrix[minigrid_pos];
        let grid_pos = (coord.x + 3 * coord.y) as usize;
        minigrid.matrix[grid_pos] = symbol;
    }
    fn update_grid(&mut self) {
        let mut index: usize = 0;
        for minigrid in self.matrix {
            let mut results = Vec::new();

            // Check rows
            results.push(minigrid.check(0, 1, 2));
            results.push(minigrid.check(3, 4, 5));
            results.push(minigrid.check(6, 7, 8));

            // Check columns
            results.push(minigrid.check(0, 3, 6));
            results.push(minigrid.check(1, 4, 7));
            results.push(minigrid.check(2, 5, 8));

            // Check diagonals
            results.push(minigrid.check(0, 4, 8));
            results.push(minigrid.check(2, 4, 6));

            for r in results {
                if r.is_some() {
                    let winner_cell: Cell = r.unwrap();
                    self.completed_minigrid[index] = winner_cell;
                    break;
                }
            }
            index += 1;
        }
    }
    fn check_completed(self, v1: usize, v2: usize, v3: usize) -> Option<Cell> {
        // All values must be non-empty and equal
        if self.completed_minigrid[v1] != Cell::Empty {
            if self.completed_minigrid[v1] == self.completed_minigrid[v2]
                && self.completed_minigrid[v2] == self.completed_minigrid[v3]
            {
                return Some(self.completed_minigrid[v1]);
            }
        }
        None
    }
    fn is_completed(self) -> Option<Cell> {
        // If the grid is completed, returns the symbol of the winner
        let mut results = Vec::new();

        // Check rows
        results.push(self.check_completed(0, 1, 2));
        results.push(self.check_completed(3, 4, 5));
        results.push(self.check_completed(6, 7, 8));

        // Check columns
        results.push(self.check_completed(0, 3, 6));
        results.push(self.check_completed(1, 4, 7));
        results.push(self.check_completed(2, 5, 8));

        // Check diagonals
        results.push(self.check_completed(0, 4, 8));
        results.push(self.check_completed(2, 4, 6));

        for r in results {
            if r.is_some() {
                let winner_cell: Cell = r.unwrap();
                return Some(winner_cell);
            }
        }
        return None;
    }
}

pub trait Player {
    /// Run before playing a match
    fn reset(&self);
    fn select_move(&self, grid: Grid, last_move: Option<Coord>) -> Coord;
}

pub fn play_match<A: Player + Copy, B: Player + Copy>(a: A, b: B) -> MatchStats {
    a.reset();
    b.reset();

    let mut grid: Grid = Default::default();
    let last_move: Option<Coord> = None;
    let mut last_player: Cell = Cell::Empty;
    let mut number_turns: u8 = 0;

    loop {
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
    };
}
