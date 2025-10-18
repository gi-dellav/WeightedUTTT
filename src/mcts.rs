use crate::defs::{Cell, Coord, Grid, MatchStats, Player};
use rand::Rng;
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct MCTSPlayer {
    exploration_weight: f32,
    simulation_steps: u32,
    symbol: Cell,
}

use std::sync::atomic::{AtomicU32, Ordering};

/// Represents a node in the Monte Carlo Tree Search
struct Node {
    state: Grid,
    visits: AtomicU32, // Number of times node was visited
    score: AtomicU32,  // Accumulated score (stored as f32 bits)
    children: std::sync::Mutex<Vec<Arc<Node>>>,
    parent: Option<Arc<Node>>,
    last_move: Option<Coord>, // Move that led to this node
}

impl Clone for Node {
    fn clone(&self) -> Self {
        // Atomic values are manually cloned since they don't implement Clone
        Node {
            state: self.state,
            visits: AtomicU32::new(self.visits.load(Ordering::Relaxed)),
            score: AtomicU32::new(self.score.load(Ordering::Relaxed)),
            children: std::sync::Mutex::new(self.children.lock().unwrap().clone()),
            parent: self.parent.clone(),
            last_move: self.last_move,
        }
    }
}

impl Node {
    fn get_visits(&self) -> u32 {
        self.visits.load(Ordering::Relaxed)
    }

    fn get_score(&self) -> f32 {
        f32::from_bits(self.score.load(Ordering::Relaxed))
    }
}

impl MCTSPlayer {
    /// Creates a new MCTS player with specified parameters:
    /// - `exploration_weight`: Balance between exploration/exploitation (typically 1.0-2.0)
    /// - `simulation_steps`: Number of MCTS iterations per move
    /// - `symbol`: Which player symbol (Cross/Circle) this AI represents
    pub fn new(exploration_weight: f32, simulation_steps: u32, symbol: Cell) -> Self {
        Self {
            exploration_weight,
            simulation_steps,
            symbol,
        }
    }

    /// Calculate Upper Confidence Bound (UCB) for node selection
    fn ucb(&self, node: &Node) -> f32 {
        let visits = node.visits.load(Ordering::Relaxed);
        if visits == 0 {
            return f32::INFINITY; // Prioritize unvisited nodes
        }

        let score = f32::from_bits(node.score.load(Ordering::Relaxed));
        let parent_visits = node.parent.as_ref().unwrap().visits.load(Ordering::Relaxed) as f32;

        // UCB formula: exploitation term + exploration term
        (score / visits as f32)
            + self.exploration_weight * (parent_visits.ln() / visits as f32).sqrt()
    }

    /// Select child node with highest UCB score using parallel iteration
    fn select_best_child(&self, node: &Node) -> Arc<Node> {
        let children = node.children.lock().unwrap();
        if children.is_empty() {
            return Arc::new(node.clone()); // Return self if no children
        }

        children
            .par_iter()
            .max_by(|a, b| {
                self.ucb(a)
                    .partial_cmp(&self.ucb(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|n| n.clone())
            .unwrap_or_else(|| Arc::new(node.clone())) // Fallback to current node
    }

    /// Run Monte Carlo simulation from current state to terminal game state
    fn simulate(&self, state: &Grid) -> f32 {
        (0..self.simulation_steps)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();
                let mut sim_state = *state;
                let mut current_player = self.symbol;
                let mut stats = MatchStats {
                    winner: Cell::Empty,
                    number_turns: 0,
                    final_grid: sim_state,
                };

                let mut last_move: Option<Coord> = None;
            
                // Play out random moves until game conclusion
                while stats.winner == Cell::Empty && stats.number_turns < 9 {
                    let legal_moves = sim_state.get_legal_moves(last_move);
                    if legal_moves.is_empty() {
                        break; // No valid moves remaining
                    }

                    // Select random move from available options
                    let random_move = legal_moves[rng.gen_range(0..legal_moves.len())];
                    sim_state.set(random_move, current_player);
                    sim_state.update_grid();

                    // Update last move to current move's LOCAL position for next turn constraints
                    last_move = Some(Coord {
                        meta_x: random_move.meta_x,
                        meta_y: random_move.meta_y,
                        x: random_move.x,
                        y: random_move.y,
                    });

                    // Check for game completion after each move
                    stats.winner = sim_state.is_completed().unwrap_or(Cell::Empty);
                    stats.final_grid = sim_state;
                    stats.number_turns += 1;

                    // Switch players for next turn (maintain symbol until turn ends)
                    current_player = match current_player {
                        Cell::Cross => Cell::Circle,
                        _ => Cell::Cross,
                    };
                }

                // Return simulation result relative to our player
                match stats.winner {
                    winner if winner == self.symbol => 1.0,  // Win
                    winner if winner != Cell::Empty => -1.0, // Loss
                    _ => 0.0,                                // Draw
                }
            })
            .sum::<f32>()
            / self.simulation_steps as f32
    }

    /// Backpropagate simulation results through the tree
    fn backpropagate(node: &Arc<Node>, result: f32) {
        let mut current = node.clone();
        while let Some(parent) = &current.parent {
            parent.visits.fetch_add(1, Ordering::Relaxed);
            parent.score.fetch_add(result.to_bits(), Ordering::Relaxed);
            current = parent.clone();
        }
    }
}

impl Player for MCTSPlayer {
    fn reset(&self) {
        // No state needs to be reset between matches
    }

    /// Select best move using MCTS algorithm
    fn select_move(&self, grid: Grid, _last_move: Option<Coord>) -> Coord {
        let root = Arc::new(Node {
            state: grid,
            visits: AtomicU32::new(0),
            score: AtomicU32::new(0.0f32.to_bits()),
            children: std::sync::Mutex::new(Vec::new()),
            parent: None,
            last_move: None,
        });

        // Parallel MCTS iterations
        (0..self.simulation_steps).into_par_iter().for_each(|_| {
            let mut node = root.clone();

            // Selection phase - traverse tree using UCB until leaf node
            while !node.children.lock().unwrap().is_empty() {
                node = self.select_best_child(&node);
            }

            // Expansion phase - create child nodes for legal moves
            if node.visits.load(Ordering::Relaxed) == 0 {
                let legal_moves = node.state.get_legal_moves(node.last_move);

                // Parallel node creation for all legal moves
                legal_moves.par_iter().for_each(|m| {
                    let mut new_state = node.state;
                    new_state.set(*m, self.symbol);
                    new_state.update_grid();

                    node.children.lock().unwrap().push(Arc::new(Node {
                        state: new_state,
                        visits: AtomicU32::new(0),
                        score: AtomicU32::new(0.0f32.to_bits()),
                        children: std::sync::Mutex::new(Vec::new()),
                        parent: Some(node.clone()),
                        last_move: Some(*m),
                    }));
                });

                // After expansion, continue selection from current node
            }

            // Simulation phase - play out random game from current state
            let result = self.simulate(&node.state);

            // Backpropagation phase - update tree statistics
            Self::backpropagate(&node, result);
        });

        let children = root.children.lock().unwrap();
        children.par_iter()
            .max_by(|a, b| a.visits.load(Ordering::Relaxed).cmp(&b.visits.load(Ordering::Relaxed)))
            .and_then(|n| n.last_move)
            .expect("No valid moves found despite legal moves existing - This error should never be reached")
    }
}
