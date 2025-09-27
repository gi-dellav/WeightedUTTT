
use crate::defs::{Cell, Coord, Grid, Player, MatchStats};
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

struct Node {
    state: Grid,
    visits: AtomicU32,
    score: AtomicU32,
    children: std::sync::Mutex<Vec<Arc<Node>>>,
    parent: Option<Arc<Node>>,
    last_move: Option<Coord>,
}

impl Clone for Node {
    fn clone(&self) -> Self {
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
    pub fn new(exploration_weight: f32, simulation_steps: u32, symbol: Cell) -> Self {
        Self {
            exploration_weight,
            simulation_steps,
            symbol,
        }
    }


    fn ucb(&self, node: &Node) -> f32 {
        let visits = node.visits.load(Ordering::Relaxed);
        if visits == 0 {
            return f32::INFINITY;
        }
        let score = f32::from_bits(node.score.load(Ordering::Relaxed));
        let parent_visits = node.parent.as_ref().unwrap().visits.load(Ordering::Relaxed) as f32;
        
        score / visits as f32 + 
            self.exploration_weight * 
            (parent_visits.ln().sqrt()) / visits as f32
    }

    fn select_best_child(&self, node: &Node) -> Arc<Node> {
        node.children.lock().unwrap().par_iter()
            .max_by(|a, b| self.ucb(a).partial_cmp(&self.ucb(b)).unwrap())
            .unwrap()
            .clone()
    }

    fn simulate(&self, state: &Grid) -> f32 {
        (0..self.simulation_steps).into_par_iter().map(|_| {
            let mut rng = rand::thread_rng();
            let mut sim_state = *state;
            let mut current_player = self.symbol;
            let mut stats = MatchStats {
                winner: Cell::Empty,
                number_turns: 0,
                final_grid: sim_state,
            };

            while stats.winner == Cell::Empty && stats.number_turns < 9 {
                let legal_moves = sim_state.get_legal_moves(None); // During simulation we don't track last move
                if legal_moves.is_empty() {
                    break;
                }
                let random_move = legal_moves[rng.gen_range(0..legal_moves.len())];
                
                sim_state.set(random_move, current_player);
                sim_state.update_grid();
                sim_state.update_grid();
                
                stats.final_grid = sim_state;
                stats.winner = sim_state.is_completed().unwrap_or(Cell::Empty);
                current_player = if current_player == Cell::Cross {
                    Cell::Circle
                } else {
                    Cell::Cross
                };
                stats.number_turns += 1;
            }

            match stats.winner {
                winner if winner == self.symbol => 1.0,
                winner if winner != Cell::Empty => -1.0,
                _ => 0.0,
            }
        }).sum::<f32>() / 100.0
    }

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
    fn reset(&self) {}

    fn select_move(&self, grid: Grid, _last_move: Option<Coord>) -> Coord {
        let root = Arc::new(Node {
            state: grid,
            visits: AtomicU32::new(0),
            score: AtomicU32::new(0.0f32.to_bits()),
            children: std::sync::Mutex::new(Vec::new()),
            parent: None,
            last_move: None, 
        });

        (0..self.simulation_steps).into_par_iter().for_each(|_| {
            let mut node = root.clone();
            
            // Selection
            while !node.children.lock().unwrap().is_empty() {
                node = self.select_best_child(&node);
            }
            
            // Expansion
            if node.visits.load(Ordering::Relaxed) > 0 {
                let legal_moves = node.state.get_legal_moves(node.last_move);
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
                let next_node = { 
                    let children = node.children.lock().unwrap();
                    children[0].clone()
                };
                node = next_node;
            }
            
            // Simulation
            let result = self.simulate(&node.state);
            
            // Backpropagation
            Self::backpropagate(&node, result);
        });

        let children = root.children.lock().unwrap();
        let legal_moves = root.state.get_legal_moves(None);
        children.par_iter()
            .max_by(|a, b| a.visits.load(Ordering::Relaxed).cmp(&b.visits.load(Ordering::Relaxed)))
            .map(move |n| legal_moves
                .into_par_iter()
                .find_first(|m| {
                    let mut g = root.state;
                    g.set(*m, Cell::Cross);
                    g.update_grid();
                    g == n.state
                })
                .unwrap())
            .unwrap()
    }
}
