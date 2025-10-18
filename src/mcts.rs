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
        let mut rng = rand::thread_rng();
        let mut sim_state = *state;
        let mut current_player = self.symbol;
        let mut last_move: Option<Coord> = None;
    
        // Play out random moves until game conclusion
        loop {
            // Check if the game is completed
            if let Some(winner) = sim_state.is_completed() {
                return match winner {
                    w if w == self.symbol => 1.0,
                    w if w != Cell::Empty => -1.0,
                    _ => 0.0,
                };
            }
            
            // Get legal moves
            let legal_moves = sim_state.get_legal_moves(last_move);
            if legal_moves.is_empty() {
                return 0.0; // Draw if no moves available
            }

            // Select random move from available options
            let random_move = legal_moves[rng.gen_range(0..legal_moves.len())];
            sim_state.set(random_move, current_player);
            sim_state.update_grid();

            // Update last move
            last_move = Some(random_move);

            // Switch players for next turn
            current_player = match current_player {
                Cell::Cross => Cell::Circle,
                Cell::Circle => Cell::Cross,
                _ => panic!("Invalid player"),
            };
        }
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
    fn select_move(&self, grid: Grid, last_move: Option<Coord>) -> Coord {
        let root = Arc::new(Node {
            state: grid,
            visits: AtomicU32::new(0),
            score: AtomicU32::new(0.0f32.to_bits()),
            children: std::sync::Mutex::new(Vec::new()),
            parent: None,
            last_move: None,
        });

        // Get initial legal moves to ensure we always have valid moves
        let initial_legal_moves = grid.get_legal_moves(last_move);
        if initial_legal_moves.is_empty() {
            panic!("No legal moves available");
        }

        // Parallel MCTS iterations
        (0..self.simulation_steps).into_par_iter().for_each(|_| {
            let mut node = root.clone();

            // Selection phase - traverse tree using UCB until leaf node
            while !node.children.lock().unwrap().is_empty() {
                node = self.select_best_child(&node);
            }

            // Always use get_legal_moves to ensure we're following game rules
            let legal_moves = node.state.get_legal_moves(node.last_move);
            
            // If there are legal moves and the node hasn't been expanded yet, expand
            if node.visits.load(Ordering::Relaxed) == 0 && !legal_moves.is_empty() {
                // Create child nodes for all legal moves
                for m in &legal_moves {
                    let mut new_state = node.state;
                    new_state.set(*m, self.symbol);
                    new_state.update_grid();

                    let child_node = Arc::new(Node {
                        state: new_state,
                        visits: AtomicU32::new(0),
                        score: AtomicU32::new(0.0f32.to_bits()),
                        children: std::sync::Mutex::new(Vec::new()),
                        parent: Some(node.clone()),
                        last_move: Some(*m),
                    });
                    
                    node.children.lock().unwrap().push(child_node);
                }
            }

            // If we have children after expansion, select one to simulate from
            // Otherwise, use the current node for simulation
            let node_to_simulate = if !node.children.lock().unwrap().is_empty() {
                // Select a child node to simulate from
                self.select_best_child(&node)
            } else {
                node.clone()
            };

            // Simulation phase - play out random game from the selected state
            let result = self.simulate(&node_to_simulate.state);

            // Backpropagation phase - update tree statistics
            Self::backpropagate(&node_to_simulate, result);
        });

        // Select the move with the highest number of visits from the root's children
        // Ensure the selected move is legal according to the initial state
        let children = root.children.lock().unwrap();
        let best_child = children.iter()
            .max_by_key(|child| child.visits.load(Ordering::Relaxed));
        
        if let Some(child) = best_child {
            if let Some(mv) = child.last_move {
                // Verify the move is in the initial legal moves
                if initial_legal_moves.contains(&mv) {
                    return mv;
                }
            }
        }
        
        // Fallback: pick the first legal move if no child found or best move is invalid
        *initial_legal_moves.first().unwrap()
    }
}
