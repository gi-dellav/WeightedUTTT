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
#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub fn set(&mut self, coord: Coord, symbol: Cell) {
        let minigrid_pos = (coord.meta_x + 3 * coord.meta_y) as usize;
        let mut minigrid = self.matrix[minigrid_pos];
        let grid_pos = (coord.x + 3 * coord.y) as usize;
        minigrid.matrix[grid_pos] = symbol;
    }
    pub fn update_grid(&mut self) {
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
    pub fn is_completed(self) -> Option<Cell> {
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

    /// Get all valid legal moves based on game state and last move
    pub fn get_legal_moves(&self, last_move: Option<Coord>) -> Vec<Coord> {
        let mut moves = Vec::new();
        
        // Determine which meta grids are playable
        let allowed_meta = match last_move {
            Some(Coord { x, y, .. }) => {
                let target_meta_idx = (x + y * 3) as usize;
                if self.completed_minigrid[target_meta_idx] == Cell::Empty {
                    vec![(x, y)]
                } else {
                    // If target meta grid is completed, player can choose any available
                    self.completed_minigrid.iter()
                        .enumerate()
                        .filter_map(|(i, &cell)| {
                            if cell == Cell::Empty {
                                Some((i as u8 % 3, i as u8 / 3))
                            } else {
                                None
                            }
                        })
                        .collect()
                }
            }
            None => {
                // First move can be anywhere
                self.completed_minigrid.iter()
                    .enumerate()
                    .filter_map(|(i, &cell)| {
                        if cell == Cell::Empty {
                            Some((i as u8 % 3, i as u8 / 3))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
        };

        // Check each allowed meta grid for available cells
        for (meta_x, meta_y) in allowed_meta {
            let minigrid_idx = (meta_x + meta_y * 3) as usize;
            let minigrid = self.matrix[minigrid_idx];
            
            for y in 0..3 {
                for x in 0..3 {
                    if minigrid.matrix[(x + y * 3) as usize] == Cell::Empty {
                        moves.push(Coord {
                            meta_x,
                            meta_y,
                            x,
                            y,
                        });
                    }
                }
            }
        }
        
        moves
    }
}

pub trait Player: Send + Sync {
    /// Run before playing a match
    fn reset(&self);
    fn select_move(&self, grid: Grid, last_move: Option<Coord>) -> Coord;
}

pub fn play_match<A: Player + Copy, B: Player + Copy>(a: A, b: B) -> MatchStats {
    a.reset();
    b.reset();

    let mut grid: Grid = Default::default();
    let mut last_move: Option<Coord> = None;
    let mut current_player = Cell::Cross;
    let mut number_turns: u8 = 0;

    loop {
        let coord = match current_player {
            Cell::Cross => a.select_move(grid, last_move),
            Cell::Circle => b.select_move(grid, last_move),
            _ => panic!("Invalid player state"),
        };
        
        grid.set(coord, current_player);
        grid.update_grid();
        last_move = Some(coord);
        
        if let Some(winner) = grid.is_completed() {
            break MatchStats {
                winner,
                number_turns,
                final_grid: grid,
            };
        }
        
        current_player = if current_player == Cell::Cross {
            Cell::Circle
        } else {
            Cell::Cross
        };
        number_turns += 1;
    }

    return MatchStats {
        winner: _last_player,
        number_turns,
        final_grid: grid,
    };
}
